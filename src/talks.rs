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
    AssistantObject, CreateMessageRequestArgs, CreateThreadRequestArgs, MessageContent, RunObject,
    RunStatus, ThreadObject,
};
use async_openai::{config::OpenAIConfig, Client};
use strum_macros::{Display, EnumIter, EnumString};
mod basic;
mod correct;
pub mod lang_practice;
mod summarize;
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
                        .role("user")
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
    run: &RunObject,
    thread_id: &str,
) -> Result<String> {
    loop {
        let run = client.threads().runs(thread_id).retrieve(&run.id).await?;
        if let RunStatus::Completed = run.status {
            let query = [("limit", "5")];
            let response = client.threads().messages(thread_id).list(&query).await?;
            // println!("{:?}", response);
            let content = response.data.get(0).unwrap().content.get(0).unwrap();
            if let MessageContent::Text(text) = content {
                return Ok(text.text.value.clone());
            }
        } else {
            std::thread::sleep(std::time::Duration::from_secs(1));
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
        }
    }
}
