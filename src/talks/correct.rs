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
use async_openai::{config::OpenAIConfig, Client};

pub async fn get_conv(
    client: &Client<OpenAIConfig>,
    name: &str,
    native: &bool,
) -> Result<TalkStart, Error> {
    let refine = if *native {
        Some("Rephrase the text as a fluent native speaker.")
    } else {
        None
    };
    let (asst, thread) = get_asst_thread(client, name, refine).await?;
    let presuff = ("<correct_me>\n".to_string(), "\n</correct_me>".to_string());
    let msg = Some("Paste the text and I'll correct it.".to_string());
    let ts = TalkStart {
        thread,
        asst,
        msg,
        presuff,
    };
    Ok(ts)
}
