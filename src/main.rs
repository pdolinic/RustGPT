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
//use std::env;
use std::fs;
use std::io::{self, Write};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum MyError {
    #[error("error parsing json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("error sending request: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("unknown error")]
    Unknown,
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

async fn send_request(prompt: &Prompt, api_key: &str) -> Result<Output, MyError> {
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
        Err(MyError::Unknown)
    }
}

async fn ask(prompt: &mut Prompt, api_key: &str) -> Result<String, MyError> {
    let output = send_request(prompt, api_key).await?;
    let reply = &output.choices[0].message.content;
    println!("\nGPT-3.5-turbo: {}\n", reply);
    Ok(reply.clone())
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let model = "gpt-4";

    let api_key_file = "my_api_key.txt";
    let api_key_path = format!("/usr/local/bin/{}", api_key_file);

    let my_api_key = fs::read_to_string(api_key_path)?.trim().to_owned();

    let mut prompt = build_prompt(model, &[]); // Start with an empty history

    // For demonstration purposes, you can run a loop until the user types "exit"
    loop {
        print!("You: ");
        io::stdout().flush()?; // Flush stdout to print the prompt immediately
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input = input.trim().to_string(); // Remove trailing whitespace

        if input.to_lowercase() == "exit" {
            break;
        }

        // Add user input to the history
        prompt.messages.push(Message {
            role: "user".to_string(),
            content: input.clone(),
        });

        // Send the updated Prompt with history, get GPT's response, and add it to the history
        let reply = ask(&mut prompt, &my_api_key).await?;
        prompt.messages.push(Message {
            role: "assistant".to_string(),
            content: reply,
        });
    }

    Ok(())
}
