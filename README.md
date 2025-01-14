# CescoGPT: a multipurpose CLI and Telegram bot, powered by OpenAI API

## Overview

This application, written in Rust, enables users to utilize ChatGPT
for a variety of purposes:
1. for plain, standard queries;
2. to practice conversation in a foreign language;
3. to correct and improve texts;
4. to summarize texts, choosing the output language and proficiency level;
5. to translate SRT movie subtitles into any desired language;
6. to transcribe audio files into text;
7. to generate images via DALL-E 3.

## Configuration

###  Setting the API key

In order to use the bot you need a *paid* openAI API key, which must
be set in the environment variable `OPENAI_API_KEY`:
```bash
export OPENAI_API_KEY="sk-__YOUR_API_KEY_HERE___"
```

### Setting up the assistants

To get the bot up and running, you'll have to create five
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

#### Translate Subtitles

````
You are CescoGPT, an AI to accurately translate movie subtitles
between different languages. The subtitles are given as values of a
dictionary, codified as a JSON. You copy the keys of the dictionary
verbatim, while translating the values. You try to translate each
single string of the json on its own. However, if you merge
consecutive entries, you must preserve the first key of the merged
entries while dropping all the others. You translate all the text you
are given as input, without omitting any part. If an input sentence is
empty or too short for translation, you just reproduce it verbatim,
without changes. Your output is in JSON format, like the input.

Here's an example, translating into Italian. Input:
```
{"000obxmO": "Hi, how are", "001Lfyqd": "you?", "002aC3nE": "Fine, thanks."}
```
Desired output:
```
{"000obxmO": "Ciao, come va?", "002aC3nE": "Bene, grazie."}
```
````

**Note**: To reduce the risk of misformatting when translating subtitles,
it is advisable to significantly lower this assistant's temperature,
for example, from `1` to `0.01`.

### Filtering the user access

The configuration file
[conf/defaults.toml](conf/defaults.toml.template) must contain an
`id_whitelist` field, which can be filled with a list of allowed
Telegram user_ids:
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

## Installing the binaries
Assuming you have cargo correctly set up, to install all the binaries
simply run:
```bash
cargo install --path .
```

## Running the bot

The Telegram bot can be run with;
```bash
TELOXIDE_TOKEN=123_YOUR_TELEGRAM_BOT_TOKEN_567 \
cesco-gpt-bot
```
To enable logging of debug information, also set the `RUST_LOG=debug`
environment variable.

## Running the CLI

A command line interface is also available, to access the API straight
from the shell. Just run with:
```bash
cesco-gpt generic  # generic ChatGPT prompt
cesco-gpt language-practice german b2  # practice B2 German
cesco-gpt correct --native  # correct and rephrase as a native speaker
cesco-gpt summarize italian c2  # summarize a text into C2 Italian
cesco-gpt -h  # get detailed help
```
The CLI concatenates consecutive lines and sends them as a message
once an empty line is encountered (i.e., *press enter twice to send
the message*). An empty message ends the conversation.

#### Logging the conversation

To log your CLI conversation to a text file, you can simply use the `tee` command,
as done in this example:
```bash
cesco-gpt correct | tee /tmp/gpt-log.txt
```

### Image generation via DALL-E 3

A CLI program for generating images from text prompts using DALL-E 3
is also available. To view the syntax, use the following command:
```
dalle-create -h
```

### Subtitle tranlation

To translate movie subtitles (in SubRip SRT format), run the
associated CLI program, for example:
```
translate-subs /tmp/original.deu.srt /tmp/translated.eng.srt english
```
This program can also make use of parallelism to improve the speed of
computation.  For details, run:
```
translate-subs -h
```

### Speech to text

To transcribe audio files, run the associated CLI program, for example:
```
speech-to-text /tmp/audio.m4a /tmp/script.txt
```
The program also supports [custom
prompts](https://platform.openai.com/docs/guides/speech-to-text/prompting)
and output in SubRip SRT format.  For details, run:
```
speech-to-text -h
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
