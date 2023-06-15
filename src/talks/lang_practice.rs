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
use crate::talks::TalkStart;
use chatgpt::client::ChatGPT;
use chatgpt::err::Error;
use clap::ValueEnum;
use strum_macros::{Display, EnumIter, EnumString};

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
    client: &ChatGPT,
    lang: &Lang,
    level: &LangLevel,
) -> Result<TalkStart, Error> {
    let sys_msg = "You are CescoGPT, an AI to practice conversation in \
    foreign languages. You always reply in the current foreign language, by \
    1. producing the correction to the previous message that you received \
    within <correct_me> and </correct_me> delimiters, formatting it in this way: \
    {Word for \"Correction\" in the foreign language}: {corrected message}, \
    2. replying to the message and 3. you always end your \
    response with a related question.";
    let msg = format!("We'll talk in {level} level {lang}. I'll start the conversation.");

    let mut conv = client.new_conversation_directed(sys_msg);
    let response = conv.send_message(msg).await?;
    let msg = &response.message().content;
    let presuff = ("<correct_me>\n".to_string(), "\n</correct_me>".to_string());
    let ts = TalkStart {
        conv,
        msg: Some(msg.to_string()),
        presuff,
    };
    Ok(ts)
}
