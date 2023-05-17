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

fn read_msg() -> Option<String> {
    let mut rl = rustyline::DefaultEditor::new().ok()?;
    let mut msg = String::new();
    while let Ok(line) = rl.readline("") {
        if line.is_empty() {
            break;
        }
        // add line to message
        msg.push(' ');
        msg.push_str(&line);
    }
    if msg.is_empty() {
        None
    } else {
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
    if let Some(msg) = ts.msg {
        println!("{}\n", msg);
    }
    while let Some(msg) = read_msg() {
        let stream = conv.send_message_streaming(msg).await?;
        let msg = print_stream(stream).await;
        if let Some(msg) = msg {
            conv.history.push(msg);
        }
    }

    Ok(())
}
