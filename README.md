# CescoGPT: a Telegram bot for language practice, powered by ChatGPT API (and Rust)

## Overview

This bot, written in Rust, allows users to either access ChatGPT for
for plain, standard queries, or via a predefined prompt, tailored to practice
conversation in a foreign language. The prompt used is this:
```rust
let sys_msg = "You are CescoGPT, an AI to practice conversation in \
foreign languages. You always reply, using the foreign language, by \
1. producing the correction to the previous message you received, \
formatting it in this way: \
Correction: `{corrected message}`, \
2. replying to the message and 3. you always end your \
response with a related question.";
let msg = format!("We'll talk in {level} level {lang}. I'll start the conversation.");
```

## Configuration

##  Setting the API key

In order to use the bot you need a **paid** openAI API key, which must be
filled in the configuration file [conf/defaults.toml](conf/defaults.toml):
```toml
openai_api_key = "sk-__YOUR_API_KEY_HERE___"
```

### Filtering the user access

The configuration file [conf/defaults.toml](conf/defaults.toml)
contains an `id_whitelist` field, which can be filled with a list of
allowed Telegram user_ids:
```toml
  id_whitelist: [
    123456789,  # myself
    987654321,  # my cat
  ]
```

**If the list is left empty, filtering is not performed** (i.e., all
users will be able to use the bot).

## Running the bot

Assuming you have cargo correctly set up, just run:
```bash
TELOXIDE_TOKEN=123_YOUR_TELEGRAM_BOT_TOKEN_567 \
cargo run --bin cesco-gpt-bot
```

## Running the CLI

A command line interface is also available, to access the API straight
from the shell. Just run with:
```bash
cargo run --bin cesco-gpt -- generic  # generic ChatGPT prompt
cargo run --bin cesco-gpt -- language-practice german b2  # practice B2 German
cargo run --bin cesco-gpt -- -h  # get detailed help
```
The CLI concatenates consecutive lines and sends them as a message
once an empty line is encountered (i.e., **press enter twice to send
the message**). An empty message ends the conversation.

## Customization

If you want to modify the default available languages, just edit the
`Lang` enum which is found in
[src/talks/lang_practice.rs](src/talks/lang_practice.rs).

## Known problems

The behaviour for language practice is sometimes erratic, e.g., after
some time it might forget to output your text correction, especially
if you ask questions in your messages. If you have better prompts for
language practice, feel free to suggest them via email or by opening
an issue.

## Author

CescoGPT is developed by
  * Francesco Versaci <francesco.versaci@gmail.com>

## License

CescoGPT is licensed under the under the Apache License, Version
2.0. See LICENSE for further details.
