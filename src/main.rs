// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
//
// Authors: pdolinic, GPT-4
//
use reqwest::header::{HeaderMap, CONTENT_TYPE, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Prompt {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
}

fn build_prompt(model: &str, prompts: &[String]) -> Prompt {
    let messages: Vec<Message> = prompts
        .iter()
        .map(|prompt| Message {
            role: "user".to_string(),
            content: prompt.clone(),
        })
        .collect();

    Prompt {
        model: model.to_string(),
        messages,
    }
}

async fn send_request(prompt: &Prompt, api_key: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let json_data = serde_json::to_string(prompt)?;

    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(AUTHORIZATION, format!("Bearer {}", api_key).parse().unwrap());

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .body(json_data)
        .send()
        .await?;

    if response.status().is_success() {
        let output: Output = response.json().await?;
        Ok(output)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error processing prompts: {} {}", response.status(), response.text().await?),
        )))
    }
}

fn process_output(output: &Output) {
    for choice in &output.choices {
        println!("{}\n", choice.message.content);
    }

    println!(
        "--------------------------------------------------------------------------------------------"
    );
    println!(
        "Debugging fields: {:#?}",
        (
            ("id", &output.id),
            ("object", &output.object),
            ("created", &output.created),
            ("model", &output.model)
        )
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = "gpt-3.5-turbo";

    let api_key_file = "my_api_key.txt";
    let api_key_path = format!("/usr/local/bin/{}", api_key_file);

    let my_api_key = fs::read_to_string(api_key_path)?.trim().to_owned();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run prompts ...");
        return Ok(());
    }

    let prompts = &args[1..];

    let prompt = build_prompt(model, prompts);

    match send_request(&prompt, &my_api_key).await {
        Ok(output) => process_output(&output),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}

