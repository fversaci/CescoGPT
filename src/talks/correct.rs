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

pub async fn get_conv(client: &ChatGPT, native: &bool) -> Result<TalkStart, Error> {
    let mut sys_msg = "You are CescoGPT, an AI to correct and improve texts. \
    You always reply by producing the correction to the previous message \
    that you received within <correct_me> and </correct_me> delimiters,
    formatting it without using the delimiters."
        .to_string();
    if *native {
        sys_msg += "Rephrase the text as a fluent native speaker.";
    }
    let conv = client.new_conversation_directed(sys_msg);
    let presuff = ("<correct_me>\n".to_string(), "\n</correct_me>".to_string());
    let msg = Some("Paste the text and I'll correct it.".to_string());
    let ts = TalkStart { conv, msg, presuff };
    Ok(ts)
}
