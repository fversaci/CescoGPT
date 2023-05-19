use crate::{ChatConv, HashSet, MyState};
use anyhow::Result;
use cesco_gpt::talks::lang_practice::{Lang, LangLevel};
use cesco_gpt::talks::Talk;
use chatgpt::prelude::*;
use futures_util::Stream;
use futures_util::StreamExt;
use std::str::FromStr;
use std::sync::Arc;
use strum::IntoEnumIterator;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    payloads,
    prelude::*,
    requests::JsonRequest,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId, ParseMode},
    utils::command::BotCommands,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Default, Clone)]
pub enum State {
    #[default]
    Bouncer,
    Start {
        my_state: Arc<MyState>,
    },
    InitTalk {
        my_state: Arc<MyState>,
        prev: Option<MessageId>,
    },
    ChooseLevel {
        my_state: Arc<MyState>,
        prev: Option<MessageId>,
        talk: Talk,
    },
    SetLevel {
        my_state: Arc<MyState>,
        prev: Option<MessageId>,
        talk: Talk,
    },
    DoTalk {
        my_state: Arc<MyState>,
        chat_conv: ChatConv,
    },
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Show available commands.")]
    Help,
    #[command(description = "(Re)start the menu.")]
    Start,
}

pub fn schema(
    my_state: Arc<MyState>,
) -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let run_bouncer = move |bot: Bot, dialogue: MyDialogue, msg: Message| {
        bouncer(bot, dialogue, msg, my_state.clone())
    };

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Start].endpoint(run_bouncer));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(
            case![State::DoTalk {
                my_state,
                chat_conv
            }]
            .endpoint(do_talk),
        )
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![State::InitTalk { my_state, prev }].endpoint(init_talk))
        .branch(
            case![State::ChooseLevel {
                my_state,
                prev,
                talk
            }]
            .endpoint(choose_level),
        )
        .branch(
            case![State::SetLevel {
                my_state,
                prev,
                talk
            }]
            .endpoint(set_level),
        );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /help to see the usage.",
    )
    .await?;
    Ok(())
}

fn allowed(chat_id: &ChatId, whitelist: &HashSet<ChatId>) -> bool {
    whitelist.is_empty() | whitelist.contains(chat_id)
}

async fn bouncer(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    my_state: Arc<MyState>,
) -> HandlerResult {
    bot.set_my_commands(Command::bot_commands()).await?;
    // whitelist check
    let chat_id = msg.chat.id;
    let wl = &my_state.my_conf.id_whitelist;
    if !allowed(&chat_id, wl) {
        bot.send_message(chat_id, "Sorry dude, you're not in the whitelist.")
            .await?;
        return Ok(());
    }
    // set initial state
    dialogue
        .update(State::Start {
            my_state: my_state.clone(),
        })
        .await?;
    select_talk(bot, dialogue, my_state).await
}

async fn select_talk(bot: Bot, dialogue: MyDialogue, my_state: Arc<MyState>) -> HandlerResult {
    let talks_per_row = 3;
    let chat_id = dialogue.chat_id();
    let talks: Vec<Talk> = Talk::iter().collect();
    let talks = talks.chunks(talks_per_row).map(|row| {
        row.iter()
            .map(|talk| talk.to_string())
            .map(|talk_cmd| InlineKeyboardButton::callback(talk_cmd.clone(), talk_cmd))
    });
    let txt_msg = "Choose the conversation:".to_string();
    let sent = bot
        .send_message(chat_id, txt_msg)
        .reply_markup(InlineKeyboardMarkup::new(talks))
        .await?;
    let prev = Some(sent.id);
    dialogue.update(State::InitTalk { my_state, prev }).await?;
    Ok(())
}

async fn clean_buttons(bot: Bot, chat_id: ChatId, m_id: Option<MessageId>) -> HandlerResult {
    // clean old buttons?
    if let Some(m_id) = m_id {
        bot.delete_message(chat_id, m_id).await?;
    }
    Ok(())
}

async fn init_talk(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    tup_state: (Arc<MyState>, Option<MessageId>),
) -> HandlerResult {
    let (my_state, prev) = tup_state;
    let chat_id = dialogue.chat_id();
    clean_buttons(bot.clone(), chat_id, prev).await?;
    let talk = q.data.unwrap_or_default();
    let talk = Talk::from_str(&talk).unwrap_or_default();
    match talk {
        Talk::LanguagePractice { .. } => choose_lang(bot, dialogue, talk, my_state).await,
        Talk::Generic => start_talk(bot, dialogue, talk, my_state).await,
    }
}

async fn choose_lang(
    bot: Bot,
    dialogue: MyDialogue,
    talk: Talk,
    my_state: Arc<MyState>,
) -> HandlerResult {
    let langs_per_row = 3;
    let chat_id = dialogue.chat_id();
    let langs: Vec<Lang> = Lang::iter().collect();
    let langs = langs.chunks(langs_per_row).map(|row| {
        row.iter()
            .map(|lang| lang.to_string())
            .map(|lang_cmd| InlineKeyboardButton::callback(lang_cmd.clone(), lang_cmd))
    });
    let txt_msg = "Choose the language:".to_string();
    let sent = bot
        .send_message(chat_id, txt_msg)
        .reply_markup(InlineKeyboardMarkup::new(langs))
        .await?;
    let prev = Some(sent.id);
    dialogue
        .update(State::ChooseLevel {
            my_state,
            prev,
            talk,
        })
        .await?;
    Ok(())
}

async fn choose_level(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    tup_state: (Arc<MyState>, Option<MessageId>, Talk),
) -> HandlerResult {
    let (my_state, prev, mut talk) = tup_state;
    let chat_id = dialogue.chat_id();
    clean_buttons(bot.clone(), chat_id, prev).await?;
    let new_lang = q.data.unwrap_or_default();
    let new_lang = Lang::from_str(&new_lang).unwrap_or_default();
    if let Talk::LanguagePractice { ref mut lang, .. } = talk {
        *lang = new_lang;
    }
    let levs_per_row = 2;
    let chat_id = dialogue.chat_id();
    let levs: Vec<LangLevel> = LangLevel::iter().collect();
    let levs = levs.chunks(levs_per_row).map(|row| {
        row.iter()
            .map(|lev| lev.to_string())
            .map(|lev_cmd| InlineKeyboardButton::callback(lev_cmd.clone(), lev_cmd))
    });
    let txt_msg = "Choose your level:".to_string();
    let sent = bot
        .send_message(chat_id, txt_msg)
        .reply_markup(InlineKeyboardMarkup::new(levs))
        .await?;
    let prev = Some(sent.id);
    dialogue
        .update(State::SetLevel {
            my_state,
            prev,
            talk,
        })
        .await?;
    Ok(())
}

async fn set_level(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    tup_state: (Arc<MyState>, Option<MessageId>, Talk),
) -> HandlerResult {
    let (my_state, prev, mut talk) = tup_state;
    let chat_id = dialogue.chat_id();
    clean_buttons(bot.clone(), chat_id, prev).await?;
    let new_lev = q.data.unwrap_or_default();
    let new_lev = LangLevel::from_str(&new_lev).unwrap_or_default();
    if let Talk::LanguagePractice { ref mut level, .. } = talk {
        *level = new_lev;
    }
    start_talk(bot, dialogue, talk, my_state).await
}

async fn start_talk(
    bot: Bot,
    dialogue: MyDialogue,
    talk: Talk,
    my_state: Arc<MyState>,
) -> HandlerResult {
    let chat_id = dialogue.chat_id();
    let chat_client = my_state.chat_conv.chat_client.clone();
    let ts = talk.get_conv(&chat_client).await?;
    let conv = Some(ts.conv);
    if let Some(msg) = ts.msg {
        send_markdown(bot, chat_id, &msg).await?;
    }
    let chat_conv = ChatConv { chat_client, conv };
    dialogue
        .update(State::DoTalk {
            my_state,
            chat_conv,
        })
        .await?;
    Ok(())
}

async fn do_talk(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    tup_state: (Arc<MyState>, ChatConv),
) -> HandlerResult {
    let (my_state, chat_conv) = tup_state;
    let chat_id = msg.chat.id;
    let msg = msg.text().unwrap().to_string();
    let mut conv = chat_conv.conv.unwrap();
    let stream = conv.send_message_streaming(msg).await?;
    let resp = send_stream(bot, chat_id, stream).await;
    // save reply in chat history
    if let Some(resp) = resp {
        conv.history.push(resp);
    }
    let chat_client = my_state.chat_conv.chat_client.clone();
    let chat_conv = ChatConv {
        chat_client,
        conv: Some(conv),
    };
    dialogue
        .update(State::DoTalk {
            my_state,
            chat_conv,
        })
        .await?;

    Ok(())
}

async fn send_markdown(bot: Bot, chat_id: ChatId, msg: &str) -> HandlerResult {
    let md = payloads::SendMessage::new(chat_id, msg);
    type Sender = JsonRequest<payloads::SendMessage>;
    let sent = Sender::new(bot.clone(), md.clone().parse_mode(ParseMode::Markdown)).await;
    // If markdown cannot be parsed, send it as raw text
    if sent.is_err() {
        Sender::new(bot.clone(), md.clone()).await?;
        log::debug!("Cannot parse markdown: {}", sent.err().unwrap());
    };

    Ok(())
}

async fn update_markdown(bot: Bot, chat_id: ChatId, m_id: MessageId, msg: &str) -> HandlerResult {
    let md = payloads::EditMessageText::new(chat_id, m_id, msg);
    type Sender = JsonRequest<payloads::EditMessageText>;
    let sent = Sender::new(bot.clone(), md.clone().parse_mode(ParseMode::Markdown)).await;
    // If markdown cannot be parsed, send it as raw text
    if sent.is_err() {
        Sender::new(bot.clone(), md.clone()).await?;
        log::debug!("Cannot parse markdown: {}", sent.err().unwrap());
    };

    Ok(())
}

async fn send_stream(
    bot: Bot,
    chat_id: ChatId,
    mut stream: impl Stream<Item = ResponseChunk> + std::marker::Unpin,
) -> Option<ChatMessage> {
    // send message zero
    let zero = bot.send_message(chat_id, "...").await.ok()?;
    let m_id = zero.id;
    // send updates
    let mut output: Vec<ResponseChunk> = Vec::new();
    let mut msg = String::new();
    let mut cow: u64 = 0;
    let block = 32;
    while let Some(chunk) = stream.next().await {
        match chunk {
            ResponseChunk::Content {
                delta,
                response_index,
            } => {
                msg.push_str(&delta);
                output.push(ResponseChunk::Content {
                    delta,
                    response_index,
                });
                // send/update msg every block token
                cow += 1;
                if cow % block == 0 {
                    update_markdown(bot.clone(), chat_id, m_id, &msg)
                        .await
                        .ok()?;
                }
            }
            other => output.push(other),
        }
    }
    // send/update final msg
    update_markdown(bot.clone(), chat_id, m_id, &msg)
        .await
        .ok()?;
    let msgs = ChatMessage::from_response_chunks(output);
    if msgs.is_empty() {
        None
    } else {
        Some(msgs[0].to_owned())
    }
}
