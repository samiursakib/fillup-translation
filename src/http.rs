use std::{env};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct BodyPart {
  text: String,
}

#[derive(Serialize)]
struct BodyContent {
  parts: Vec<BodyPart>,
}

#[derive(Serialize)]
struct GeminiRequest {
  contents: Vec<BodyContent>,
}

#[derive(Deserialize, Debug)]
struct ResponsePart {
  text: String,
}

#[derive(Deserialize, Debug)]
struct ResponseContent {
  parts: Vec<ResponsePart>,
}

#[derive(Deserialize, Debug)]
struct ResponseCandidate {
  content: ResponseContent,
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
  candidates: Vec<ResponseCandidate>,
}

pub async fn post_call(prompt: String) -> Result<String, reqwest::Error> {
  // println!("entered post call");
  let gemini_url = env::var("GEMINI_URL").unwrap_or_else(|_| String::new());
  let gemini_api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| String::new());
  let body = GeminiRequest {
    contents: vec![
      BodyContent {
        parts: vec![
          BodyPart { text: prompt }
        ]
      }
    ]
  };
  let client = reqwest::Client::new();
  let response = client.post(gemini_url)
    .header("x-goog-api-key", gemini_api_key)
    .json(&body)
    .send()
    .await?
    .error_for_status()?;
  // println!("{:?}", response);

  let result: GeminiResponse = response.json().await?;
  let answer = result.candidates.get(0)
    .and_then(|c| c.content.parts.get(0))
    .map(|p| p.text.clone())
    .unwrap_or_else(|| String::new());

  // println!("\napi response: {:?}", answer);
  Ok(answer)
}
