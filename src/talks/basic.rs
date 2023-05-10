use chatgpt::client::ChatGPT;
use chatgpt::converse::Conversation;
use chatgpt::err::Error;

pub async fn get_conv(client: &ChatGPT) -> Result<Conversation, Error> {
    let conv = client.new_conversation();
    Ok(conv)
}
