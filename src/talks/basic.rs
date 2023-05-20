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

pub async fn get_conv(client: &ChatGPT) -> Result<TalkStart, Error> {
    let conv = client.new_conversation();
    let msg = Some("Ask away, my friend.".to_string());
    let ts = TalkStart { conv, msg };
    Ok(ts)
}
