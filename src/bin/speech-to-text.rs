/**************************************************************************
  Copyright 2024 Francesco Versaci (https://github.com/fversaci/)

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

use anyhow::Result;
use async_openai::types::{
    AudioResponseFormat, CreateTranscriptionRequestArgs, CreateTranslationRequestArgs,
};
use async_openai::Client;
use clap::{ArgGroup, Parser};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None,
          group(ArgGroup::new("trans")
                .required(true)
                .args(["lang", "to_eng"])))]
struct Args {
    /// Input audio file (mp3, mp4, mpeg, mpga, m4a, wav, or webm)
    audio_fn: PathBuf,
    /// Output text file
    out_txt: PathBuf,
    /// The input language in ISO-639-1 format (2 letters code)
    #[arg(long)]
    lang: Option<String>,
    /// The model will try to match the style of the prompt
    #[arg(long)]
    prompt: Option<String>,
    /// Produce SubRip SRT output
    #[arg(long, default_value_t = false)]
    srt: bool,
    /// Translate into English
    #[arg(long, default_value_t = false)]
    to_eng: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new();
    let mut out_file = File::create(args.out_txt)?;

    let fmt = if args.srt {
        AudioResponseFormat::Srt
    } else {
        AudioResponseFormat::Json
    };
    if args.to_eng {
        // Translation
        let mut request = CreateTranslationRequestArgs::default()
            .file(args.audio_fn)
            .model("whisper-1")
            .response_format(fmt)
            .build()?;
        request.prompt = args.prompt;

        if args.srt {
            let response = client.audio().translate_raw(request).await?;
            writeln!(out_file, "{}", String::from_utf8_lossy(response.as_ref()))?;
        } else {
            let response = client.audio().translate(request).await?;
            writeln!(out_file, "{}", response.text)?;
        }
    } else {
        // Transcription
        let mut request = CreateTranscriptionRequestArgs::default()
            .file(args.audio_fn)
            .model("whisper-1")
            .language(args.lang.unwrap())
            .response_format(fmt)
            .build()?;
        request.prompt = args.prompt;

        if args.srt {
            let response = client.audio().transcribe_raw(request).await?;
            writeln!(out_file, "{}", String::from_utf8_lossy(response.as_ref()))?;
        } else {
            let response = client.audio().transcribe(request).await?;
            writeln!(out_file, "{}", response.text)?;
        }
    }

    Ok(())
}
