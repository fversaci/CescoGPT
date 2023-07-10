# CescoGPT: a Telegram bot for language practice, powered by ChatGPT API (and Rust)

## Overview

This bot, written in Rust, allows users to access ChatGPT
1. for plain, standard queries;
2. to practice conversation in a foreign language, via a [predefined
prompt](src/talks/lang_practice.rs#L51);
3. to correct and improve texts, via another [predefined
prompt](src/talks/correct.rs#L21);
4. to summarize texts, choosing the output language and its level, via
a [predefined prompt](src/talks/summarize.rs#L26).

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

## Running the CLI

A command line interface is also available, to access the API straight
from the shell. Just run with:
```bash
cargo run --bin cesco-gpt -- generic  # generic ChatGPT prompt
cargo run --bin cesco-gpt -- language-practice german b2  # practice B2 German
cargo run --bin cesco-gpt -- correct --native  # correct and rephrase as a native speaker
cargo run --bin cesco-gpt -- -h  # get detailed help
```
The CLI concatenates consecutive lines and sends them as a message
once an empty line is encountered (i.e., *press enter twice to send
the message*). An empty message ends the conversation.

### Logging the conversation

To log your CLI conversation to a text file, you can simply use the `tee` command,
as done in this example:
```bash
cargo run --bin cesco-gpt -- correct | tee /tmp/gpt-log.txt
```

## Customization

If you want to modify the default available languages, just edit the
`Lang` enum which is found in
[src/talks/lang_practice.rs](src/talks/lang_practice.rs).

## Known problems

Sometimes the OpenAI API may hang and then only three dots are shown
as a response. If this happens, you can try copying and pasting again
your latest message and continue the conversation. If that doesn't
work, you can use the command `/restart` to restart the conversation.

## Author

CescoGPT is developed by
  * Francesco Versaci <francesco.versaci@gmail.com>

## License

CescoGPT is licensed under the under the Apache License, Version
2.0. See LICENSE for further details.
