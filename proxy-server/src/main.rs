use std::{env, io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}};
use dotenv;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct ProxyRequest {
  required_pairs: Vec<String>,
  dir_name: String,
}

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

#[tokio::main]
async fn main() {
  dotenv::dotenv().ok();
  let listener = TcpListener::bind("127.0.0.1:4000").unwrap();
  for stream in listener.incoming() {
    let stream = stream.unwrap();

    println!("connection established: {:#?}", stream);
    println!("Connected");
    handle_connection(stream).await;
  }
}

async fn handle_connection(mut stream: TcpStream) {
  let prompt_ref = &include_str!("./prompt_format.txt");
  let prompt = prompt_ref.to_string();

  let mut buf_reader = BufReader::new(&stream);
  let mut content_length: usize = 0;
  let mut header_line = String::new();
  let mut header_lines: Vec<String> = Vec::new();
  let mut payload_string = String::from("{}");

  loop {
    header_line.clear();
    match buf_reader.read_line(&mut header_line) {
      Ok(0) | Err(_) => break,
      Ok(_) => {
        if header_line.trim().is_empty() {
          break;
        }

        if header_line.to_lowercase().starts_with("content-length:") {
          println!("header_line: {header_line:#?}");
          if let Some(len_string) = header_line.split(":").nth(1) {
            println!("len_string: {len_string:#?}");
            if let Ok(len_num) = len_string.trim().parse::<usize>() {
              println!("len_num: {len_num:#?}");
              content_length = len_num;
            }
          }
        }

        header_lines.push(header_line.trim().to_string());
      }
    }
  }

  let mut buffer = vec![0; content_length];
  println!("content_length: {content_length:#?}");
  if content_length > 0 {
    if buf_reader.read_exact(&mut buffer).is_err() {
      eprintln!("Error during reading body");
      return;
    }
    payload_string = String::from_utf8_lossy(&buffer).to_string();
    println!("payload_string {payload_string:#?}");
  }
  println!("headers: {header_lines:#?}");
  println!("payload string: {payload_string:#?}");
  let payload_json: ProxyRequest = conversion_string_to_json(&payload_string).unwrap_or(ProxyRequest { required_pairs: vec![], dir_name: String::new() });
  println!("body: {payload_json:#?}");

  let formatted_prompt = prompt.replace("REQUIRED_KEY_VALUE_PAIRS", payload_json.required_pairs.join(",").as_str()).replace("LANGUAGE_CODE", &payload_json.dir_name);
  println!("formatted prompt: {:?}", formatted_prompt);

  let gemini_response = post_call(formatted_prompt).await;
  // println!("gemini_respone: {gemini_response:#?}");
  let response = match gemini_response {
    Ok(gemini_result) => format!("HTTP/1.1 200 OK\r\n\r\n{}", gemini_result),
    Err(e) => {
      eprintln!("Server failed to process: {e:#?}");
      format!("HTTP/1.1 500 Internal Server Error\r\n\r\n")
    }
  };
  println!("proxy server response: {response:#?}");
  stream.write_all(response.as_bytes()).unwrap();
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

fn conversion_string_to_json(payload_str: &str) -> Result<ProxyRequest, serde_json::Error> {
  let payload_json = serde_json::from_str(payload_str)?;
  Ok(payload_json)
}
