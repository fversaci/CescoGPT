use chatgpt::client::ChatGPT;
use chatgpt::converse::Conversation;
use chatgpt::err::Error;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Default, Display, Debug, Clone, EnumIter, EnumString)]
pub enum Lang {
    #[default]
    German,
    English,
    French,
}

#[derive(Default, Display, Debug, Clone, EnumIter, EnumString)]
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
    client: ChatGPT,
    lang: &Lang,
    level: &LangLevel,
) -> Result<Conversation, Error> {
    let sys_msg = "You are CescoGPT, an AI to practice conversation in \
    foreign languages. You always reply by first correcting \
    the previous message you received and you always end your \
    response with a question. You are concise and reply shortly.";
    let msg = format!("We'll talk in {level} level {lang}. I'll start the conversation.");

    let mut conv = client.new_conversation_directed(sys_msg);
    conv.send_message(msg).await?;
    Ok(conv)
}
