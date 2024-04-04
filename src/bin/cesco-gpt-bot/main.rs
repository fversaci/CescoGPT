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
use async_openai::{config::OpenAIConfig, types::CreateRunRequest, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

mod telegram;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyBotConfig {
    id_whitelist: HashSet<ChatId>,
}

#[derive(Clone)]
pub struct ChatConv {
    chat_client: Client<OpenAIConfig>,
    thread_id: String,
    run_request: CreateRunRequest,
    presuff: (String, String),
}

#[derive(Clone)]
pub struct MyState {
    my_conf: MyBotConfig,
    chat_client: Client<OpenAIConfig>,
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
    pretty_env_logger::init_timed();
    log::info!("Starting bot...");
    let bot = Bot::from_env();
    let my_conf = get_conf();
    log::debug!("{my_conf:?}");
    let chat_client = Client::new();
    let my_state = MyState {
        my_conf,
        chat_client,
    };
    Dispatcher::builder(bot, telegram::schema(my_state))
        .dependencies(dptree::deps![InMemStorage::<telegram::State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
