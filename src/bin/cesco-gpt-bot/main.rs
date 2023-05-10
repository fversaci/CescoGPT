use anyhow::Result;
use chatgpt::client::ChatGPT;
use chatgpt::converse::Conversation;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

mod telegram;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Insert your openAI API key
    api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyBotConfig {
    id_whitelist: HashSet<ChatId>,
}

pub struct ChatConv {
    chat_client: ChatGPT,
    conv: Option<Conversation>,
}

impl Clone for ChatConv {
    fn clone(&self) -> Self {
        match &self.conv {
            Some(conv) => {
                let conv =
                    Conversation::new_with_history(self.chat_client.clone(), conv.history.clone());
                ChatConv {
                    chat_client: self.chat_client.clone(),
                    conv: Some(conv),
                }
            }
            None => self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MyState {
    my_conf: MyBotConfig,
    chat_conv: ChatConv,
}

fn get_conf() -> MyBotConfig {
    let fname = "conf/defaults.toml";
    let conf_txt = fs::read_to_string(fname)
        .unwrap_or_else(|_| panic!("Cannot find configuration file: {}", fname));
    let my_conf: MyBotConfig = toml::from_str(&conf_txt)
        .unwrap_or_else(|err| panic!("Unable to parse configuration file {}: {}", fname, err));
    my_conf
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    log::info!("Starting bot...");
    let bot = Bot::from_env();
    let my_conf = get_conf();
    log::debug!("{my_conf:?}");
    let args = Args::parse();
    let key = args.api_key;
    let chat_client = ChatGPT::new(key)?;
    let my_state = Arc::new(MyState {
        my_conf,
        chat_conv: ChatConv {
            chat_client,
            conv: None,
        },
    });
    Dispatcher::builder(bot, telegram::schema(my_state))
        .dependencies(dptree::deps![InMemStorage::<telegram::State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
