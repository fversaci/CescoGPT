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

use anyhow::Result;
use async_openai::types::{
    ChatCompletionResponseStream, CreateMessageRequestArgs, CreateRunRequestArgs,
};
use async_openai::Client;
use cesco_gpt::talks::{get_response, Talk};
use clap::Parser;
use std::io::{stdout, Write};
use tokio_stream::{Stream, StreamExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Choose which conversation to start
    #[command(subcommand)]
    talk: Talk,
}

fn read_msg(presuff: &(String, String)) -> Option<String> {
    let (pre, suff) = presuff; // initial and final delimiters
    let mut msg = pre.clone();
    let zero_sz = msg.len();
    let mut rl = rustyline::DefaultEditor::new().ok()?;
    while let Ok(line) = rl.readline("") {
        if line.is_empty() {
            break;
        }
        // add line to message
        msg.push_str(&line);
        msg.push('\n');
    }
    if msg.len() == zero_sz {
        None
    } else {
        // add final delimiter
        msg.push_str(suff);
        Some(msg)
    }
}

async fn print_stream(mut stream: ChatCompletionResponseStream) -> Result<()> {
    let mut lock = stdout().lock();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                response.choices.iter().for_each(|chat_choice| {
                    if let Some(ref content) = chat_choice.delta.content {
                        write!(lock, "{}", content).unwrap();
                    }
                });
            }
            Err(err) => {
                writeln!(lock, "error: {err}").unwrap();
            }
        }
        lock.flush()?;
    }
    writeln!(lock).unwrap();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new();
    let talk = args.talk;
    let ts = talk.get_conv(&client).await?;
    let thread = ts.thread;
    let asst = ts.asst;
    let presuff = ts.presuff;
    if let Some(msg) = ts.msg {
        println!("{}\n", msg);
    }
    let run_request = CreateRunRequestArgs::default()
        .assistant_id(&asst.id)
        .build()?;

    while let Some(msg) = read_msg(&presuff) {
        let message = CreateMessageRequestArgs::default()
            .role("user")
            .content(msg)
            .build()?;
        let _message_obj = client
            .threads()
            .messages(&thread.id)
            .create(message)
            .await?;
        let run = client
            .threads()
            .runs(&thread.id)
            .create(run_request.clone())
            .await?;
        let resp = get_response(&client, &run.id, &thread.id).await?;
        println!("{resp}\n");
    }

    Ok(())
}
