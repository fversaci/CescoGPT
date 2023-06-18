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
use chatgpt::client::ChatGPT;
use chatgpt::converse::Conversation;
use chatgpt::err::Error;
use strum_macros::{Display, EnumIter, EnumString};
mod basic;
mod correct;
pub mod lang_practice;
use clap::Subcommand;
use lang_practice::{Lang, LangLevel};

pub struct TalkStart {
    pub conv: Conversation,
    pub msg: Option<String>,
    pub presuff: (String, String),
}

#[derive(Default, Display, Debug, Clone, EnumIter, EnumString, Subcommand)]
pub enum Talk {
    /// Generic Chat-GPT prompt
    #[default]
    #[strum(serialize = "Generic ChatGPT")]
    Generic,
    /// Practice conversation in chosen language
    #[strum(serialize = "Language Practice")]
    LanguagePractice {
        #[arg(value_enum)]
        lang: Lang,
        #[arg(value_enum)]
        level: LangLevel,
    },
    /// Correct and improve text, as a native speaker
    #[strum(serialize = "Correct text")]
    Correct {
        #[arg(short, long)]
        native: bool,
    },
}

impl Talk {
    pub async fn get_conv(&self, client: &ChatGPT) -> Result<TalkStart, Error> {
        match self {
            Talk::Generic => basic::get_conv(client).await,
            Talk::LanguagePractice { lang, level } => {
                lang_practice::get_conv(client, lang, level).await
            }
            Talk::Correct { native } => correct::get_conv(client, native).await,
        }
    }
}
