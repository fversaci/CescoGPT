use crate::talks::TalkStart;
use chatgpt::client::ChatGPT;
use chatgpt::err::Error;

pub async fn get_conv(client: &ChatGPT) -> Result<TalkStart, Error> {
    let conv = client.new_conversation();
    let msg = Some("Ask away, my friend.".to_string());
    let ts = TalkStart { conv, msg };
    Ok(ts)
}
