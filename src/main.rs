//created by pdolinic made with use of gpt-4
//gplv3 license 

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

    let messages: Vec<Message> = prompts
        .iter()
        .map(|prompt| Message {
            role: "user".to_string(),
            content: prompt.clone(),
        })
        .collect();

    let prompt = Prompt {
        model: model.to_string(),
        messages,
    };

    let json_data = serde_json::to_string(&prompt)?;

    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(AUTHORIZATION, format!("Bearer {}", my_api_key).parse().unwrap());

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .body(json_data)
        .send()
        .await?;

    if response.status().is_success() {
        let output: Output = response.json().await?;

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
    } else {
        println!(
            "Error processing prompts: {} {}",
            response.status(),
            response.text().await?
        );
    }

    Ok(())
}
