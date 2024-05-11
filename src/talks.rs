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

use anyhow::{anyhow, Error, Result};
use async_openai::types::{
    AssistantObject, CreateMessageRequestArgs, CreateThreadRequestArgs, MessageContent,
    MessageRole, RunStatus, ThreadObject,
};
use async_openai::{config::OpenAIConfig, Client};
use strum_macros::{Display, EnumIter, EnumString};
mod basic;
mod correct;
pub mod lang_practice;
mod summarize;
mod translate_subs;
use clap::Subcommand;
use lang_practice::{Lang, LangLevel};

pub struct TalkStart {
    pub thread: ThreadObject,
    pub asst: AssistantObject,
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
    #[strum(serialize = "Correct Text")]
    Correct {
        #[arg(short, long)]
        native: bool,
    },
    /// Summarize text to 10% of original length
    #[strum(serialize = "Summarize Text")]
    Summarize {
        #[arg(value_enum)]
        lang: Lang,
        #[arg(value_enum)]
        level: LangLevel,
    },
    /// Translate subtitles into chosen language
    #[strum(serialize = "Translate Subtitles")]
    TranslateSubs {
        #[arg(value_enum)]
        lang: Lang,
    },
}

async fn get_asst_thread(
    client: &Client<OpenAIConfig>,
    name: &str,
    refine: Option<&str>,
) -> Result<(AssistantObject, ThreadObject)> {
    let mut last_id = "".to_string();
    loop {
        let query = [("limit", "100"), ("after", &last_id)];
        let asst_list = client.assistants().list(&query).await?;
        if let Some(l_id) = asst_list.last_id {
            last_id = l_id;
        }
        let data = asst_list.data;
        for asst in data {
            if asst.name.clone().is_some_and(|x| x == name) {
                let thread_request = CreateThreadRequestArgs::default().build()?;
                let thread = client.threads().create(thread_request.clone()).await?;
                if let Some(refine) = refine {
                    let ref_msg = CreateMessageRequestArgs::default()
                        .role(MessageRole::User)
                        .content(refine)
                        .build()?;
                    let _ref_obj = client
                        .threads()
                        .messages(&thread.id)
                        .create(ref_msg)
                        .await?;
                }
                return Ok((asst, thread));
            }
        }
        if !asst_list.has_more {
            return Err(anyhow!("No assistant found with name {name}."));
        }
    }
}

pub async fn get_response(
    client: &Client<OpenAIConfig>,
    run_id: &str,
    thread_id: &str,
) -> Result<String> {
    loop {
        let run = client.threads().runs(thread_id).retrieve(run_id).await?;
        match run.status {
            RunStatus::Completed => {
                let query = [("limit", "5")];
                let response = client.threads().messages(thread_id).list(&query).await?;
                let content = response.data.first().unwrap().content.first().unwrap();
                if let MessageContent::Text(text) = content {
                    return Ok(text.text.value.clone());
                }
            }
            RunStatus::InProgress | RunStatus::Queued => {
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            _ => {
                return Err(anyhow!("{:?}: {:?}", run.status, run.last_error));
            }
        }
    }
}

impl Talk {
    pub async fn get_conv(&self, client: &Client<OpenAIConfig>) -> Result<TalkStart, Error> {
        match self {
            Talk::Generic => basic::get_conv(client, &self.to_string()).await,
            Talk::LanguagePractice { lang, level } => {
                lang_practice::get_conv(client, &self.to_string(), lang, level).await
            }
            Talk::Correct { native } => correct::get_conv(client, &self.to_string(), native).await,
            Talk::Summarize { lang, level } => {
                summarize::get_conv(client, &self.to_string(), lang, level).await
            }
            Talk::TranslateSubs { lang } => {
                translate_subs::get_conv(client, &self.to_string(), lang).await
            }
        }
    }
    pub fn runs_on_bot(&self) -> bool {
        match self {
            Talk::Generic => true,
            Talk::LanguagePractice { .. } => true,
            Talk::Correct { .. } => true,
            Talk::Summarize { .. } => false,
            Talk::TranslateSubs { .. } => false,
        }
    }
}
