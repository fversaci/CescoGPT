/**************************************************************************
  Copyright 2024 Francesco Versaci (https://github.com/fversaci/)

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
use crate::talks::lang_practice::Lang;
use crate::talks::{get_asst_thread, TalkStart};
use anyhow::{Error, Result};
use async_openai::{config::OpenAIConfig, Client};

pub async fn get_conv(
    client: &Client<OpenAIConfig>,
    name: &str,
    lang: &Lang,
) -> Result<TalkStart, Error> {
    let refine = format!("You will always translate the subtitles into {lang} language.");
    let (asst, thread) = get_asst_thread(client, name, Some(&refine)).await?;
    let presuff = ("".to_string(), "".to_string());
    let msg = Some("Enter the subtitles and I'll translate them for you.".to_string());
    let ts = TalkStart {
        thread,
        asst,
        msg,
        presuff,
    };
    Ok(ts)
}
