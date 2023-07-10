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
use crate::talks::lang_practice::{Lang, LangLevel};
use crate::talks::TalkStart;
use chatgpt::client::ChatGPT;
use chatgpt::err::Error;

pub async fn get_conv(
    client: &ChatGPT,
    lang: &Lang,
    level: &LangLevel,
) -> Result<TalkStart, Error> {
    let sys_msg = format!(
        "You are CescoGPT, an AI designed to summarize texts. \
         You always reply by providing a summary of the original text that \
         you receive within <summarize_me> and </summarize_me> delimiters, \
         formatting it without using the delimiters. Your summaries are written \
         exclusively in {level} level {lang}, and the length of the summary is \
         approximately 10% of the length of the original text."
    );
    let conv = client.new_conversation_directed(sys_msg);
    let presuff = (
        "<summarize_me>\n".to_string(),
        "\n</summarize_me>".to_string(),
    );
    let msg = Some("Paste the text and I'll summarize it for you.".to_string());
    let ts = TalkStart { conv, msg, presuff };
    Ok(ts)
}
