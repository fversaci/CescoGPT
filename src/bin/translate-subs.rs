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
use std::collections::HashMap;
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
    #[arg(long, default_value_t = 64)]
    chunk: usize,
    /// Number of parallel translators
    #[arg(long, default_value_t = 1)]
    num: usize,
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
        let json_str = chunk_to_json(chunk, &self.lang)?;
        let trans_json_str = self.translate_str(&json_str).await?;
        let ret = json_to_chunk(&trans_json_str, chunk);
        if ret.is_ok() {
            return ret;
        }
        // Something went wrong, replace with a new translator
        let new_trans = Translator::new(self.client.clone(), self.lang.clone()).await?;
        *self = new_trans;
        // Couldn't translate even a single block, use the original
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
            anyhow::bail!("Error: pool must have at least 1 translator");
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

fn chunk_to_json(chunk: &[SrtSubtitle], lang: &Lang) -> Result<String> {
    let chunk_text: Vec<Vec<String>> = chunk.iter().map(|sub| sub.text.clone()).collect();
    let chunk_dict: HashMap<usize, Vec<String>> = chunk_text.into_iter().enumerate().collect();
    let json_str = serde_json::to_string(&chunk_dict)?;
    let cmd = format!(
        "Translate these subtitles into {} language. Your output must have exactly the same json format as the input.\n",
        lang
    );
    let cmd_json_str = format!("{}{}", cmd, json_str);
    Ok(cmd_json_str)
}

fn json_to_chunk(json_str: &str, in_chunk: &[SrtSubtitle]) -> Result<Vec<SrtSubtitle>> {
    let chunk_dict: HashMap<usize, Vec<String>> = serde_json::from_str(json_str)?;
    let mut sorted_values: Vec<(usize, Vec<String>)> = chunk_dict.into_iter().collect();
    sorted_values.sort_by_key(|&(key, _)| key);
    let chunk_text: Vec<Vec<String>> = sorted_values.into_iter().map(|(_, strs)| strs).collect();
    let mut out_chunk = Vec::new();
    for (in_block, text) in in_chunk.iter().zip(chunk_text) {
        let trans_block = SrtSubtitle { text, ..*in_block };
        out_chunk.push(trans_block);
    }
    Ok(out_chunk)
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
    for chunk in srt.subtitles.chunks(args.chunk) {
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
