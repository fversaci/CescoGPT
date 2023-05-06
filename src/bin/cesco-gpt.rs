use cesco_gpt::talks::Talk;
use chatgpt::prelude::*;
use clap::Parser;
use std::io::{self, BufRead};

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let key = args.api_key;
    let client = ChatGPT::new(key)?;
    let talk = Talk::Deutsch;
    let mut conv = talk.get_conv(client).await;
    println!("Ask away my friend.\n");
    while let Some(msg) = read_msg() {
        let response = conv.send_message(msg).await?;
        println!("\n{}\n", response.message().content);
    }

    Ok(())
}
