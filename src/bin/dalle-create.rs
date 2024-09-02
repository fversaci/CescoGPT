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

use anyhow::Result;
use async_openai::types::{
    CreateImageRequestArgs, ImageModel, ImageQuality, ImageResponseFormat, ImageSize,
};
use async_openai::Client;
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Text file containing the prompt (max 1000 chars)
    prompt_file: PathBuf,
    /// Enable high detail image generation
    #[arg(long)]
    hd: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new();
    // read prompt from file
    let prompt_f = File::open(args.prompt_file)?;
    let mut prompt = String::new();
    let mut file_reader = BufReader::new(prompt_f);
    file_reader.read_to_string(&mut prompt)?;
    // create image
    let quality = if args.hd {
        ImageQuality::HD
    } else {
        ImageQuality::Standard
    };
    let request = CreateImageRequestArgs::default()
        .prompt(prompt)
        .model(ImageModel::DallE3)
        .n(1)
        .response_format(ImageResponseFormat::B64Json)
        .size(ImageSize::S1024x1024)
        .quality(quality)
        .build()?;

    let response = client.images().create(request).await?;
    let paths = response.save("/tmp/dalle").await?;

    paths
        .iter()
        .for_each(|path| println!("Image file path: {}", path.display()));

    Ok(())
}
