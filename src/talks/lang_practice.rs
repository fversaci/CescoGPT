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
    foreign languages. You always reply, using the foreign language, by \
    1. producing the correction to the previous message you received, \
    formatting it in this way: \
    Correction: `{corrected message}`, \
    2. replying to the message and 3. you always end your \
    response with a related question.";
    let msg = format!("We'll talk in {level} level {lang}. I'll start the conversation.");

    let mut conv = client.new_conversation_directed(sys_msg);
    let response = conv.send_message(msg).await?;
    let msg = &response.message().content;
    let ts = TalkStart {
        conv,
        msg: Some(msg.to_string()),
    };
    Ok(ts)
}
