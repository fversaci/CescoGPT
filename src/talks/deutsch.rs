use chatgpt::client::ChatGPT;
use chatgpt::converse::Conversation;

pub async fn get_conv(client: ChatGPT) -> Conversation {
    client.new_conversation()
}
