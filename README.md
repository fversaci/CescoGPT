# CescoGPT: a Telegram bot and CLI for language practice and more, powered by ChatGPT Assistants (and Rust)

## Overview

This bot, written in Rust, allows users to access ChatGPT
1. for plain, standard queries;
2. to practice conversation in a foreign language;
3. to correct and improve texts;
4. to summarize texts, choosing the output language and its level.

## Configuration

###  Setting the API key

In order to use the bot you need a *paid* openAI API key, which must be
filled in the configuration file [conf/defaults.toml](conf/defaults.toml):
```toml
openai_api_key = "sk-__YOUR_API_KEY_HERE___"
```

### Setting up the assistants

To get the bot up and running, you'll have to create four
[assistants](https://platform.openai.com/assistants) and assign them
the following names and instructions.

#### Generic ChatGPT

```
Let's chat.
```

#### Language Practice

```
You are CescoGPT, an AI to practice conversation in foreign
languages. You always reply in the current foreign language,
by 1. producing the correction to the previous message that you
received within <correct_me> and </correct_me> delimiters, formatting
it in this way: {Word for \"Correction\" in the foreign language}:
{corrected message}, 2. replying to the message and 3. you always end
your response with a related question.
```

### Correct Text

```
You are CescoGPT, an AI to correct and improve texts.  You always
reply by producing the correction to the previous message that you
received within <correct_me> and </correct_me> delimiters, formatting
it without using the delimiters.
```

#### Summarize Text

```
You are CescoGPT, an AI designed to summarize texts. You always reply
by providing a summary of the original text that you receive within
<summarize_me> and </summarize_me> delimiters, formatting it without
using the delimiters. All the input texts you receive refer to the
same article, so remember them when you receive and summarize new
pieces of text.
```

### Filtering the user access

The configuration file [conf/defaults.toml](conf/defaults.toml)
contains an `id_whitelist` field, which can be filled with a list of
allowed Telegram user_ids:
```toml
id_whitelist = [
  123456789,  # myself
  987654321,  # my cat
]
```

Note: *If the list is left empty, no filtering will be applied* (i.e.,
all users will be able to use the bot). Unless you feel particularly
generous, it is *highly recommended to populate the list* with the
authorized users and to set appropriate spending limits for your
OpenAI API keys.

## Running the bot

Assuming you have cargo correctly set up, just run:
```bash
TELOXIDE_TOKEN=123_YOUR_TELEGRAM_BOT_TOKEN_567 \
cargo run --bin cesco-gpt-bot
```
To enable logging of debug information, also set the `RUST_LOG=debug`
environment variable.

## Running the CLI

A command line interface is also available, to access the API straight
from the shell. Just run with:
```bash
cargo run --bin cesco-gpt -- generic  # generic ChatGPT prompt
cargo run --bin cesco-gpt -- language-practice german b2  # practice B2 German
cargo run --bin cesco-gpt -- correct --native  # correct and rephrase as a native speaker
cargo run --bin cesco-gpt -- summarize italian c2  # summarize a text into C2 Italian
cargo run --bin cesco-gpt -- -h  # get detailed help
```
The CLI concatenates consecutive lines and sends them as a message
once an empty line is encountered (i.e., *press enter twice to send
the message*). An empty message ends the conversation.

### Image generation via DALL-E 3

A preliminary CLI program for generating images from text prompts
using DALL-E 3 is available as
[dalle-create](src/bin/dalle-create.rs). To view the syntax, use the
following command:
```
cargo run --bin DALL-E -- -h
```

### Logging the conversation

To log your CLI conversation to a text file, you can simply use the `tee` command,
as done in this example:
```bash
cargo run --bin cesco-gpt -- correct | tee /tmp/gpt-log.txt
```

## Language customization

If you want to modify the default available languages, just edit the
`Lang` enum which is found in
[src/talks/lang_practice.rs](src/talks/lang_practice.rs).

## Known problems

- Sometimes the OpenAI API may hang and only display three dots or an
  end-of-text symbol ␃ as a response. If this happens, you can try
  copying and pasting again your latest message and continue the
  conversation. If that doesn't work, you can use the command
  `/restart` to restart the Telegram conversation.
- Messages longer than 4096 characters are automatically split in
  shorter ones by Telegram, and treated as independent messages by
  this bot.
- Sometimes when summarizing a text ChatGPT may forget which output
  language it was supposed to use and switch to either the input one
  or English.

## Author

CescoGPT is developed by
  * Francesco Versaci <francesco.versaci@gmail.com>

## License

CescoGPT is licensed under the under the Apache License, Version
2.0. See LICENSE for further details.
