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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use subtp::srt::{SrtSubtitle, SubRip};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input subtitle file, must be SRT
    in_srt: PathBuf,
    /// Output subtitle SRT file
    out_srt: PathBuf,
    /// Language to translate to
    lang: Lang,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyCLIConfig {
    openai_api_key: String,
}

fn get_conf() -> MyCLIConfig {
    let fname = "conf/defaults.toml";
    let conf_txt = fs::read_to_string(fname)
        .unwrap_or_else(|_| panic!("Cannot find configuration file: {}", fname));
    let my_conf: MyCLIConfig = toml::from_str(&conf_txt)
        .unwrap_or_else(|err| panic!("Unable to parse configuration file {}: {}", fname, err));
    my_conf
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

async fn translate_str(
    msg: &str,
    client: &Client<OpenAIConfig>,
    thread_id: &str,
    run_request: &CreateRunRequest,
) -> Result<String> {
    let message = CreateMessageRequestArgs::default()
        .role("user")
        .content(msg)
        .build()?;
    let _message_obj = client.threads().messages(thread_id).create(message).await?;
    let run = client
        .threads()
        .runs(thread_id)
        .create(run_request.clone())
        .await?;
    let resp = get_response(client, &run.id, thread_id).await?;
    Ok(resp)
}

fn chunk_to_json(chunk: &[SrtSubtitle], lang: &Lang) -> Result<String> {
    let chunk_text: Vec<Vec<String>> = chunk.iter().map(|sub| sub.text.clone()).collect();
    let chunk_dict: HashMap<usize, Vec<String>> = chunk_text.into_iter().enumerate().collect();
    let json_str = serde_json::to_string(&chunk_dict)?;
    let cmd = format!(
        "Translate these subtitles into {} language. Your output is exactly in the same json format as the input.\n",
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

async fn translate_chunk(
    chunk: &[SrtSubtitle],
    client: &Client<OpenAIConfig>,
    thread_id: &str,
    run_request: &CreateRunRequest,
    lang: &Lang,
) -> Result<Vec<SrtSubtitle>> {
    let json_str = chunk_to_json(chunk, lang)?;
    let trans_json_str = translate_str(&json_str, client, thread_id, run_request).await?;
    let ret = json_to_chunk(&trans_json_str, chunk);
    match ret {
        Ok(..) => ret,
        _ => Ok(chunk.to_vec()),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let my_conf = get_conf();
    log::debug!("{my_conf:?}");
    let key = &my_conf.openai_api_key;
    let config = OpenAIConfig::new().with_api_key(key);
    let client = Client::with_config(config);
    // start assistant
    let lang = args.lang.clone();
    let talk = TranslateSubs { lang: lang.clone() };
    let ts = talk.get_conv(&client).await?;
    let thread = ts.thread;
    let asst = ts.asst;
    let run_request = CreateRunRequestArgs::default()
        .assistant_id(&asst.id)
        .build()?;
    // read and translate
    let srt = get_parser(args.in_srt)?;
    let mut out_file = File::create(args.out_srt)?;
    let mut out_blocks = Vec::new();
    let chunk_size = 64;
    for chunk in srt.subtitles.chunks(chunk_size) {
        let translated_chunk =
            translate_chunk(chunk, &client, &thread.id, &run_request, &lang).await?;
        for block in translated_chunk {
            writeln!(out_file, "{}", block)?;
            out_blocks.push(block);
        }
    }
    Ok(())
}
