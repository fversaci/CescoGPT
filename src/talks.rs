use chatgpt::client::ChatGPT;
use chatgpt::converse::Conversation;
use strum_macros::{Display, EnumIter, EnumString};
mod deutsch;

#[derive(Default, Display, Debug, Clone, EnumIter, EnumString)]
pub enum Talk {
    #[default]
    Deutsch,
}

impl Talk {
    pub async fn get_conv(&self, client: ChatGPT) -> Conversation {
        match self {
            Talk::Deutsch => deutsch::get_conv(client).await,
        }
    }
}
