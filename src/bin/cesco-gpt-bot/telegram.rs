use crate::{ChatConv, HashSet, MyState};
use anyhow::Result;
use cesco_gpt::talks::lang_practice::{Lang, LangLevel};
use cesco_gpt::talks::Talk;
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
    SelectTalk {
        my_state: Arc<MyState>,
        prev: Option<MessageId>,
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
        .branch(case![State::SelectTalk { my_state, prev }].endpoint(start_talk));

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
    let txt_msg = "Choose the talk:".to_string();
    let sent = bot
        .send_message(chat_id, txt_msg)
        .reply_markup(InlineKeyboardMarkup::new(talks))
        .await?;
    let prev = Some(sent.id);
    dialogue
        .update(State::SelectTalk { my_state, prev })
        .await?;
    Ok(())
}

async fn clean_buttons(bot: Bot, chat_id: ChatId, m_id: Option<MessageId>) -> HandlerResult {
    // clean old buttons?
    if let Some(m_id) = m_id {
        bot.delete_message(chat_id, m_id).await?;
    }
    Ok(())
}

async fn start_talk(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    tup_state: (Arc<MyState>, Option<MessageId>),
) -> HandlerResult {
    let talk = q.data.unwrap_or_default();
    let talk = Talk::from_str(&talk).unwrap_or_default();
    let (my_state, m_id) = tup_state;
    let chat_id = dialogue.chat_id();
    clean_buttons(bot.clone(), chat_id, m_id).await?;
    // let talk = Talk::LangPractice {
    //     lang: Lang::German,
    //     level: LangLevel::B2,
    // };
    let talk = Talk::Basic;
    let chat_client = my_state.chat_conv.chat_client.clone();
    let conv = Some(talk.get_conv(&chat_client).await?);
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
    let (_my_state, chat_conv) = tup_state;
    let chat_id = msg.chat.id;
    let msg = msg.text().unwrap().to_string();
    let mut conv = chat_conv.conv.unwrap();
    let response = conv.send_message(msg);
    let response = response.await?;
    // send reply as markdown
    let msg = &response.message().content;
    let md = payloads::SendMessage::new(chat_id, msg);
    type Sender = JsonRequest<payloads::SendMessage>;
    let sent = Sender::new(bot.clone(), md.clone().parse_mode(ParseMode::Markdown)).await;
    // If markdown cannot be parsed, send it as raw text
    if sent.is_err() {
        Sender::new(bot.clone(), md.clone()).await?;
        println!("FIX ME!");
    };

    Ok(())
}
