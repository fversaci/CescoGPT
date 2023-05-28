/**************************************************************************
  Copyright 2023 Francesco Versaci (https://github.com/fversaci/)

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
use cesco_gpt::talks::Talk;
use chatgpt::prelude::*;
use clap::Parser;
use futures_util::Stream;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{stdout, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Choose which conversation to start
    #[command(subcommand)]
    talk: Talk,
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

fn read_msg(presuff: &(String, String)) -> Option<String> {
    let mut rl = rustyline::DefaultEditor::new().ok()?;
    let (pre, suff) = presuff; // initial and final delimiters
    let mut msg = pre.clone();
    let zero_sz = msg.len();
    while let Ok(line) = rl.readline("") {
        if line.is_empty() {
            break;
        }
        // add line to message
        msg.push('\n');
        msg.push_str(&line);
    }
    if msg.len() == zero_sz {
        None
    } else {
        // add final delimiter
        msg.push('\n');
        msg.push_str(suff);
        Some(msg)
    }
}

async fn print_stream(
    mut stream: impl Stream<Item = ResponseChunk> + std::marker::Unpin,
) -> Option<ChatMessage> {
    let mut output: Vec<ResponseChunk> = Vec::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            ResponseChunk::Content {
                delta,
                response_index,
            } => {
                print!("{delta}");
                stdout().lock().flush().unwrap();
                output.push(ResponseChunk::Content {
                    delta,
                    response_index,
                });
            }
            other => output.push(other),
        }
    }
    println!("\n");
    let msgs = ChatMessage::from_response_chunks(output);
    if msgs.is_empty() {
        None
    } else {
        Some(msgs[0].to_owned())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let my_conf = get_conf();
    log::debug!("{my_conf:?}");
    let key = &my_conf.openai_api_key;
    let client = ChatGPT::new(key)?;
    let talk = args.talk;
    let ts = talk.get_conv(&client).await?;
    let mut conv = ts.conv;
    let presuff = ts.presuff;
    if let Some(msg) = ts.msg {
        println!("{}\n", msg);
    }
    while let Some(msg) = read_msg(&presuff) {
        let stream = conv.send_message_streaming(msg).await?;
        let msg = print_stream(stream).await;
        if let Some(msg) = msg {
            conv.history.push(msg);
        }
    }

    Ok(())
}
