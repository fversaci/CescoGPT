use chatgpt::client::ChatGPT;
use chatgpt::converse::Conversation;
use chatgpt::err::Error;
use strum_macros::{Display, EnumIter, EnumString};
mod basic;
pub mod lang_practice;
use lang_practice::{Lang, LangLevel};

#[derive(Default, Display, Debug, Clone, EnumIter, EnumString)]
pub enum Talk {
    #[default]
    Basic,
    LangPractice {
        lang: Lang,
        level: LangLevel,
    },
}

impl Talk {
    pub async fn get_conv(&self, client: &ChatGPT) -> Result<Conversation, Error> {
        match self {
            Talk::Basic => basic::get_conv(client).await,
            Talk::LangPractice { lang, level } => {
                lang_practice::get_conv(client, lang, level).await
            }
        }
    }
}
