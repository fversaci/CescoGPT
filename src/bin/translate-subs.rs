/**************************************************************************
  Copyright 2024 Francesco Versaci (https://github.com/fversaci/)

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at

      http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
**************************************************************************/

use anyhow::{anyhow, Result};
use async_openai::types::{CreateMessageRequestArgs, CreateRunRequest, CreateRunRequestArgs};
use async_openai::{config::OpenAIConfig, Client};
use cesco_gpt::talks::get_response;
use cesco_gpt::talks::lang_practice::Lang;
use cesco_gpt::talks::Talk::TranslateSubs;
use clap::Parser;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;
use subtp::srt::{SrtSubtitle, SubRip};
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input subtitle file, must be SRT
    in_srt: PathBuf,
    /// Output subtitle SRT file
    out_srt: PathBuf,
    /// Language to translate to
    lang: Lang,
    /// Number of blocks per query
    #[arg(long, default_value_t = 32)]
    chunk: usize,
    /// Number of parallel translators
    #[arg(long, default_value_t = 1)]
    num: usize,
}

struct RandLabel {
    rng: ThreadRng,
}

impl RandLabel {
    fn new() -> Self {
        let rng = rand::thread_rng();
        RandLabel { rng }
    }
    fn get_label(&mut self) -> String {
        let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

        let random_string: String = (0..5)
            .map(|_| {
                let idx = self.rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect();

        random_string
    }
}

struct Translator {
    client: Client<OpenAIConfig>,
    thread_id: String,
    run_request: CreateRunRequest,
    lang: Lang,
}

impl Translator {
    async fn new(client: Client<OpenAIConfig>, lang: Lang) -> Result<Self> {
        let talk = TranslateSubs { lang: lang.clone() };
        let ts = talk.get_conv(&client).await?;
        let thread = ts.thread;
        let asst = ts.asst;
        let run_request = CreateRunRequestArgs::default()
            .assistant_id(&asst.id)
            .build()?;

        Ok(Self {
            client,
            thread_id: thread.id,
            run_request,
            lang,
        })
    }
    async fn translate_str(&self, msg: &str) -> Result<String> {
        let message = CreateMessageRequestArgs::default()
            .role("user")
            .content(msg)
            .build()?;
        let _message_obj = self
            .client
            .threads()
            .messages(&self.thread_id)
            .create(message)
            .await?;
        let run = self
            .client
            .threads()
            .runs(&self.thread_id)
            .create(self.run_request.clone())
            .await?;
        let resp = get_response(&self.client, &run.id, &self.thread_id).await?;
        Ok(resp)
    }
    async fn translate_chunk(&mut self, chunk: &[SrtSubtitle]) -> Result<Vec<SrtSubtitle>> {
        // try and translate it
        let rand = RandLabel::new();
        let (in_labs, json_str) = chunk_to_json(rand, chunk, &self.lang)?;
        let trans_json_str = self.translate_str(&json_str).await?;
        let ret = json_to_chunk(&trans_json_str, in_labs, chunk);
        if ret.is_ok() {
            return ret;
        }
        // Something went wrong, replace with a new translator
        let new_trans = Translator::new(self.client.clone(), self.lang.clone()).await?;
        *self = new_trans;
        // Couldn't translate even a single block, give up and use the original text
        if chunk.len() == 1 {
            println!("Copying verbatim block {}", chunk.first().unwrap().sequence);
            return Ok(chunk.to_vec());
        }
        // More lines, try divide et impera
        println!(
            "Dividing chunk {}-{}",
            chunk.first().unwrap().sequence,
            chunk.last().unwrap().sequence
        );
        let (chunk_up, chunk_down) = chunk.split_at(chunk.len() / 2);
        let trans_up = Box::pin(self.translate_chunk(chunk_up)).await?;
        let trans_down = Box::pin(self.translate_chunk(chunk_down)).await?;
        let merged = trans_up.iter().chain(trans_down.iter()).cloned().collect();
        Ok(merged)
    }
}

struct TranslatorPool {
    translators: Vec<Arc<Mutex<Translator>>>,
    curr: usize,
    num: usize,
}

impl TranslatorPool {
    async fn new(num: usize, client: Client<OpenAIConfig>, lang: Lang) -> Result<Self> {
        if num == 0 {
            return Err(anyhow!("Error: pool must have at least 1 translator"));
        }
        let mut translators = Vec::new();
        for _ in 0..num {
            let translator = Translator::new(client.clone(), lang.clone()).await?;
            translators.push(Arc::new(Mutex::new(translator)));
        }
        let ret = Self {
            translators,
            curr: 0,
            num,
        };
        Ok(ret)
    }
    fn get_translator(&mut self) -> Arc<Mutex<Translator>> {
        let translator = self.translators[self.curr].clone();
        self.curr = (self.curr + 1) % self.num; // Move to the next translator
        translator
    }
}

fn get_parser(subs_fn: PathBuf) -> Result<SubRip> {
    let ext = subs_fn.extension().unwrap_or_default();
    if ext != "srt" {
        return Err(anyhow!("Subtitles filename must end in srt."));
    }
    // read subs file
    let subs_f = File::open(subs_fn)?;
    let mut subs = String::new();
    let mut file_reader = BufReader::new(subs_f);
    file_reader.read_to_string(&mut subs)?;
    // parse text
    Ok(SubRip::parse(&subs)?)
}

fn chunk_to_json(
    mut rand: RandLabel,
    chunk: &[SrtSubtitle],
    lang: &Lang,
) -> Result<(Vec<String>, String)> {
    let chunk_text: Vec<Vec<String>> = chunk.iter().map(|sub| sub.text.clone()).collect();
    let chunk_dict: BTreeMap<String, Vec<String>> = chunk_text
        .into_iter()
        .enumerate()
        .map(|(a, b)| (format!("{:03}{}", a, rand.get_label()), b))
        .collect();
    let chunk_labs: Vec<String> = chunk_dict.keys().cloned().collect();
    let json_str = serde_json::to_string_pretty(&chunk_dict)?;
    let cmd = format!(
        "Translate these JSON subtitles into {} language. Your output must also be in JSON format.\n",        
        lang
    );
    let cmd_json_str = format!("{}{}", cmd, json_str);
    Ok((chunk_labs, cmd_json_str))
}

fn split_into_frames(trans_text: &[String], num_frames: usize) -> Vec<Vec<String>> {
    // single frame, return text as vector
    if num_frames == 1 {
        return vec![trans_text.to_owned()];
    }
    // multiple frames, enough lines, split them
    println!("Spreading text: {:?} to {} frames", trans_text, num_frames);
    let num_lines = trans_text.len();
    if num_lines >= num_frames {
        let mut result = Vec::new();
        for i in 0..num_frames {
            let start = (i * num_lines + num_frames - 1) / num_frames;
            let end = ((i + 1) * num_lines + num_frames - 1) / num_frames;
            let slice = &trans_text[start..end];
            result.push(slice.to_vec());
        }
        return result;
    }
    // not enough lines, split words
    let joined: String = trans_text.join(" ");
    let words: Vec<&str> = joined.split_whitespace().collect();
    let num_words = words.len();
    if num_words >= num_frames {
        let mut result = Vec::new();
        for i in 0..num_frames {
            let start = (i * num_words + num_frames - 1) / num_frames;
            let end = ((i + 1) * num_words + num_frames - 1) / num_frames;
            let slice = &words[start..end];
            let slice = slice.join(" ");
            result.push(vec![slice]);
        }
        return result;
    }
    // not enough words, repeat text in each frame
    let mut result = Vec::new();
    for _ in 0..num_frames {
        result.push(trans_text.to_vec());
    }
    result
}

fn assemble_blocks(in_blocks: &[SrtSubtitle], trans_text: &[String]) -> Vec<SrtSubtitle> {
    let frames = split_into_frames(trans_text, in_blocks.len());
    assert!(frames.len() == in_blocks.len());
    let mut out_blocks = Vec::new();
    for (in_block, text) in in_blocks.iter().zip(frames) {
        let trans_block = SrtSubtitle { text, ..*in_block };
        out_blocks.push(trans_block);
    }
    out_blocks
}

fn json_to_chunk(
    json_str: &str,
    in_labs: Vec<String>,
    in_chunk: &[SrtSubtitle],
) -> Result<Vec<SrtSubtitle>> {
    let trans_dict: BTreeMap<String, Vec<String>> = serde_json::from_str(json_str)?;
    let mut in_curr = 0;
    let mut trans_iter = trans_dict.into_iter().peekable();
    let mut out_chunk = Vec::new();
    while let Some((trans_label, trans_data)) = trans_iter.next() {
        let in_label = in_labs.get(in_curr);
        if in_label.is_none() {
            return Err(anyhow!("Exhausted input"));
        }
        let in_label = in_label.unwrap();
        if *in_label != trans_label {
            return Err(anyhow!("Missing label in translated chunk"));
        }
        let trans_blocks;
        if trans_iter.peek().is_none() {
            trans_blocks = assemble_blocks(&in_chunk[in_curr..], &trans_data);
        } else {
            let next_label = &trans_iter.peek().unwrap().0;
            let mut num_blocks = 0;
            let mut found = false;
            for in_lab in &in_labs[in_curr..] {
                if in_lab == next_label {
                    found = true;
                    break;
                }
                num_blocks += 1;
            }
            if found {
                trans_blocks =
                    assemble_blocks(&in_chunk[in_curr..in_curr + num_blocks], &trans_data);
                in_curr += num_blocks;
            } else {
                return Err(anyhow!("Label not found"));
            }
        }
        out_chunk.extend(trans_blocks);
    }
    Ok(out_chunk)
}

fn is_end_of_sentence(character: &char) -> bool {
    let sentence_terminators = &[
        '.', '!', '?', ';', ':', '؟', '。', '？', '！', '।', '♪', '*', '"',
    ];
    sentence_terminators.contains(character)
}

/// divides in chunks, trying to split at end-of-sentence characters
fn chunker(subs: &[SrtSubtitle], chunk: usize) -> impl Iterator<Item = &[SrtSubtitle]> {
    let win = 5; // window size to look for eos
    let mut ret = Vec::new();
    let mut back = 0usize;
    for (i, c) in subs.chunks(chunk).enumerate() {
        let chunk_beg = i * chunk;
        let chunk_end = chunk_beg + c.len();
        let beg = i * chunk - back;
        let mut end = beg + c.len();
        let win_start = Ord::max(end - win, beg);
        let win_end = chunk_end;
        let mut bad = true;
        for j in win_start..win_end {
            let eos = subs[j]
                .text
                .last()
                .and_then(|c| c.trim_end().chars().last());
            if let Some(eos) = eos {
                if is_end_of_sentence(&eos) {
                    end = j + 1;
                    bad = false;
                }
            }
        }
        back = chunk_end - end;
        ret.push(&subs[beg..end]);
        if bad {
            println!(
                "Cannot split at end-of-sentence: {}-{} maps to {}-{}",
                chunk_beg, chunk_end, beg, end
            );
        }
    }
    ret.into_iter()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new();
    // start assistants and translate subs
    let mut pool = TranslatorPool::new(args.num, client, args.lang).await?;
    let srt = get_parser(args.in_srt)?;
    let mut out_file = File::create(args.out_srt)?;
    let mut jobs = Vec::new();
    // for chunk in srt.subtitles.chunks(args.chunk) {
    for chunk in chunker(&srt.subtitles, args.chunk) {
        // Translate each chunk concurrently using the pool
        let chunk = chunk.to_vec();
        let t = pool.get_translator();
        let task = tokio::spawn(async move {
            let mut t = t.lock().await;
            t.translate_chunk(&chunk).await
        });
        jobs.push(task);
    }
    // Collect and write the translated blocks to the output file
    for job in jobs {
        let translated_chunk = job.await?;
        for block in translated_chunk? {
            writeln!(out_file, "{}", block)?;
        }
    }

    Ok(())
}
