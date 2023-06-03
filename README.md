# CescoGPT: a Telegram bot for language practice, powered by ChatGPT API (and Rust)

## Overview

This bot, written in Rust, allows users to either access ChatGPT for
plain, standard queries, or via a predefined prompt, tailored to practice
conversation in a foreign language. The prompt used can be seen
[here](src/talks/lang_practice.rs#L51).

## Configuration

##  Setting the API key

In order to use the bot you need a *paid* openAI API key, which must be
filled in the configuration file [conf/defaults.toml](conf/defaults.toml):
```toml
openai_api_key = "sk-__YOUR_API_KEY_HERE___"
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

Note: *If the list is left empty, filtering is not performed* (i.e.,
all users will be able to use the bot).

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
once an empty line is encountered (i.e., *press enter twice to send
the message*). An empty message ends the conversation.

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

Sometimes messages to Telegram might get lost and only three dots are shown
as a response. In that case just copy and paste your latest message and carry
on with the conversation.

## Author

CescoGPT is developed by
  * Francesco Versaci <francesco.versaci@gmail.com>

## License

CescoGPT is licensed under the under the Apache License, Version
2.0. See LICENSE for further details.