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
use crate::{ChatConv, HashSet, MyState};
use anyhow::{Error, Result};
use cesco_gpt::talks::lang_practice::{Lang, LangLevel};
use cesco_gpt::talks::Talk;
use chatgpt::prelude::*;
use chrono::prelude::*;
use chrono::Duration;
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
    SetNative {
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
        )
        .branch(
            case![State::SetNative {
                my_state,
                prev,
                talk
            }]
            .endpoint(set_native),
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

async fn keyb_query(
    bot: &Bot,
    dialogue: &MyDialogue,
    txt_msg: String,
    keyb: InlineKeyboardMarkup,
) -> Result<MessageId> {
    let chat_id = dialogue.chat_id();
    let sent = bot
        .send_message(chat_id, txt_msg)
        .reply_markup(keyb)
        .await?;
    Ok(sent.id)
}

async fn select_talk(bot: Bot, dialogue: MyDialogue, my_state: Arc<MyState>) -> HandlerResult {
    let talks_per_row = 2;
    let talks: Vec<Talk> = Talk::iter().collect();
    let talks = talks.chunks(talks_per_row).map(|row| {
        row.iter()
            .map(|talk| talk.to_string())
            .map(|talk_cmd| InlineKeyboardButton::callback(talk_cmd.clone(), talk_cmd))
    });
    let txt_msg = "Choose the conversation:".to_string();
    let keyb = InlineKeyboardMarkup::new(talks);
    let prev = Some(keyb_query(&bot, &dialogue, txt_msg, keyb).await?);
    dialogue.update(State::InitTalk { my_state, prev }).await?;
    Ok(())
}

async fn clean_buttons(bot: Bot, chat_id: ChatId, m_id: Option<MessageId>) -> Result<()> {
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
        Talk::Generic => start_talk(bot, dialogue, talk, my_state).await,
        Talk::Correct { .. } => choose_native(bot, dialogue, talk, my_state).await,
        Talk::LanguagePractice { .. } | Talk::Summarize { .. } => {
            choose_lang(bot, dialogue, talk, my_state).await
        }
    }
}

async fn choose_native(
    bot: Bot,
    dialogue: MyDialogue,
    talk: Talk,
    my_state: Arc<MyState>,
) -> HandlerResult {
    let yes_no = vec![vec![
        InlineKeyboardButton::callback("Yes", "true"),
        InlineKeyboardButton::callback("No", "false"),
    ]];
    let txt_msg = "Rephrase as a native speaker?".to_string();
    let keyb = InlineKeyboardMarkup::new(yes_no);
    let prev = Some(keyb_query(&bot, &dialogue, txt_msg, keyb).await?);
    dialogue
        .update(State::SetNative {
            my_state,
            prev,
            talk,
        })
        .await?;
    Ok(())
}

async fn set_native(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    tup_state: (Arc<MyState>, Option<MessageId>, Talk),
) -> HandlerResult {
    let (my_state, prev, mut talk) = tup_state;
    let chat_id = dialogue.chat_id();
    clean_buttons(bot.clone(), chat_id, prev).await?;
    let new_nat = q.data.unwrap_or_default();
    let new_nat = bool::from_str(&new_nat).unwrap_or_default();
    if let Talk::Correct { ref mut native, .. } = talk {
        *native = new_nat;
    }
    start_talk(bot, dialogue, talk, my_state).await
}

async fn choose_lang(
    bot: Bot,
    dialogue: MyDialogue,
    talk: Talk,
    my_state: Arc<MyState>,
) -> HandlerResult {
    let langs_per_row = 3;
    let langs: Vec<Lang> = Lang::iter().collect();
    let langs = langs.chunks(langs_per_row).map(|row| {
        row.iter()
            .map(|lang| lang.to_string())
            .map(|lang_cmd| InlineKeyboardButton::callback(lang_cmd.clone(), lang_cmd))
    });
    let txt_msg = "Choose the language:".to_string();
    let keyb = InlineKeyboardMarkup::new(langs);
    let prev = Some(keyb_query(&bot, &dialogue, txt_msg, keyb).await?);
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
    match talk {
        Talk::LanguagePractice { ref mut lang, .. } => *lang = new_lang,
        Talk::Summarize { ref mut lang, .. } => *lang = new_lang,
        _ => (),
    }
    let levs_per_row = 2;
    let levs: Vec<LangLevel> = LangLevel::iter().collect();
    let levs = levs.chunks(levs_per_row).map(|row| {
        row.iter()
            .map(|lev| lev.to_string())
            .map(|lev_cmd| InlineKeyboardButton::callback(lev_cmd.clone(), lev_cmd))
    });
    let txt_msg = "Choose your level:".to_string();
    let keyb = InlineKeyboardMarkup::new(levs);
    let prev = Some(keyb_query(&bot, &dialogue, txt_msg, keyb).await?);
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
    match talk {
        Talk::LanguagePractice { ref mut level, .. } => *level = new_lev,
        Talk::Summarize { ref mut level, .. } => *level = new_lev,
        _ => (),
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
    let chat_client = my_state.chat_client.clone();
    let ts = talk.get_conv(&chat_client).await?;
    let conv = Some(ts.conv);
    let presuff = ts.presuff;
    if let Some(msg) = ts.msg {
        send_markdown(bot, chat_id, &msg).await?;
    }
    let chat_conv = ChatConv {
        chat_client,
        conv,
        presuff,
    };
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
    let (pre, suff) = chat_conv.presuff.clone();
    let mut msg_out = pre;
    msg_out.push_str(msg.text().ok_or(Error::msg("## Error in message! ##"))?);
    msg_out.push_str(&suff);
    let mut conv = chat_conv.conv.ok_or(Error::msg("## No conversation! ##"))?;
    let stream = conv.send_message_streaming(msg_out).await?;
    let resp = send_stream(bot, chat_id, stream).await?;
    // save reply in chat history
    if let Some(resp) = resp {
        conv.history.push(resp);
    }
    let chat_conv = ChatConv {
        conv: Some(conv),
        ..chat_conv
    };
    dialogue
        .update(State::DoTalk {
            my_state,
            chat_conv,
        })
        .await?;

    Ok(())
}

async fn send_markdown(bot: Bot, chat_id: ChatId, msg: &str) -> Result<()> {
    let md = payloads::SendMessage::new(chat_id, msg);
    type Sender = JsonRequest<payloads::SendMessage>;
    let sent = Sender::new(bot.clone(), md.clone().parse_mode(ParseMode::Markdown)).await;
    // If markdown cannot be parsed, send it as raw text
    if let Err(e) = sent {
        Sender::new(bot, md).await?;
        log::debug!("Cannot parse markdown: {}", e);
    }

    Ok(())
}

async fn update_markdown(bot: Bot, chat_id: ChatId, m_id: MessageId, msg: &str) -> Result<()> {
    let md = payloads::EditMessageText::new(chat_id, m_id, msg);
    type Sender = JsonRequest<payloads::EditMessageText>;
    let sent = Sender::new(bot.clone(), md.clone().parse_mode(ParseMode::Markdown)).await;
    // If markdown cannot be parsed, send it as raw text
    if let Err(e) = sent {
        Sender::new(bot, md).await?;
        log::debug!("Cannot parse markdown: {}", e);
    }

    Ok(())
}

async fn send_stream(
    bot: Bot,
    chat_id: ChatId,
    mut stream: impl Stream<Item = ResponseChunk> + std::marker::Unpin,
) -> Result<Option<ChatMessage>> {
    // send message zero
    let zero = bot.send_message(chat_id, "...").await?;
    let m_id = zero.id;
    // send updates
    let mut output: Vec<ResponseChunk> = Vec::new();
    let mut msg = String::new();
    let mut oldtime = Utc::now();
    let mintime = Duration::milliseconds(2500);
    while let Some(chunk) = stream.next().await {
        match chunk {
            ResponseChunk::Content {
                delta,
                response_index,
            } => {
                msg.push_str(&delta);
                let msg_len = msg.len(); // save length
                msg.push_str("\n...");
                output.push(ResponseChunk::Content {
                    delta,
                    response_index,
                });
                // send/update msg every block token
                let now = Utc::now();
                if now - oldtime > mintime {
                    update_markdown(bot.clone(), chat_id, m_id, &msg).await?;
                    oldtime = now;
                }
                // restore message without trailing dots
                msg.truncate(msg_len);
            }
            other => output.push(other),
        }
    }
    // send/update final msg
    if msg.is_empty() {
        msg.push_str("-- ␃ --");  // end of text
    }
    update_markdown(bot, chat_id, m_id, &msg).await?;
    let msgs = ChatMessage::from_response_chunks(output);
    Ok(msgs.first().cloned())
}
