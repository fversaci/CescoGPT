use cesco_gpt::talks::Talk;

use chatgpt::prelude::*;
use clap::Parser;
use futures_util::Stream;
use futures_util::StreamExt;
use std::io::{self, stdout, BufRead, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Which service do you want to show?
    api_key: String,
}

fn read_msg() -> Option<String> {
    let lines = io::stdin().lock().lines();
    let mut msg = String::new();
    for line in lines {
        let line = line.expect("Error while reading line from stdin.");
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

async fn print_stream(stream: impl Stream<Item = ResponseChunk>) {
    println!("\n");
    stream
        .for_each(|each| async move {
            if let ResponseChunk::Content {
                delta,
                response_index: _,
            } = each
            {
                // Printing part of response without the newline
                print!("{delta}");
                stdout().lock().flush().unwrap();
            }
        })
        .await;
    println!("\n");
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let key = args.api_key;
    let client = ChatGPT::new(key)?;
    let talk = Talk::Deutsch;
    let mut conv = talk.get_conv(client).await;
    println!("Ask away my friend.\n");
    while let Some(msg) = read_msg() {
        let stream = conv.send_message_streaming(msg).await?;
        print_stream(stream).await;
    }

    Ok(())
}
