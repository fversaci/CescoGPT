use chatgpt::client::ChatGPT;
use chatgpt::converse::Conversation;
use chatgpt::err::Error;
use strum_macros::{Display, EnumIter, EnumString};
mod basic;
pub mod lang_practice;
use clap::Subcommand;
use lang_practice::{Lang, LangLevel};

pub struct TalkStart {
    pub conv: Conversation,
    pub msg: Option<String>,
}

#[derive(Default, Display, Debug, Clone, EnumIter, EnumString, Subcommand)]
pub enum Talk {
    /// Generic Chat-GPT prompt
    #[default]
    Generic,
    /// Practice conversation in chosen language
    LanguagePractice {
        #[arg(value_enum)]
        lang: Lang,
        #[arg(value_enum)]
        level: LangLevel,
    },
}

impl Talk {
    pub async fn get_conv(&self, client: &ChatGPT) -> Result<TalkStart, Error> {
        match self {
            Talk::Generic => basic::get_conv(client).await,
            Talk::LanguagePractice { lang, level } => {
                lang_practice::get_conv(client, lang, level).await
            }
        }
    }
}
