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
use crate::talks::{get_asst_thread, TalkStart};
use anyhow::{Error, Result};
use async_openai::types::CreateRunRequestArgs;
use async_openai::{config::OpenAIConfig, Client};
use clap::ValueEnum;
use strum_macros::{Display, EnumIter, EnumString};

use super::get_response;

#[derive(Default, Display, Debug, Clone, EnumIter, EnumString, ValueEnum)]
pub enum Lang {
    #[default]
    English,
    German,
    French,
    Spanish,
    Catalan,
    Latin,
    Italian,
    Interlingua,
}

#[derive(Default, Display, Debug, Clone, EnumIter, EnumString, ValueEnum)]
pub enum LangLevel {
    #[default]
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
}

pub async fn get_conv(
    client: &Client<OpenAIConfig>,
    name: &str,
    lang: &Lang,
    level: &LangLevel,
) -> Result<TalkStart, Error> {
    let refine = format!("We'll talk in {level} level {lang}. I'll start the conversation.");
    let (asst, thread) = get_asst_thread(client, name, Some(&refine)).await?;
    let run_request = CreateRunRequestArgs::default()
        .assistant_id(&asst.id)
        .build()?;
    let run = client
        .threads()
        .runs(&thread.id)
        .create(run_request.clone())
        .await?;
    let resp = get_response(client, &run.id, &thread.id).await?;
    let presuff = ("<correct_me>\n".to_string(), "\n</correct_me>".to_string());
    let ts = TalkStart {
        thread,
        asst,
        msg: Some(resp),
        presuff,
    };
    Ok(ts)
}
