use serde::{Serialize};

#[derive(Serialize, Debug)]
struct ProxyRequest {
  required_pairs: Vec<String>,
  dir_name: String,
}

pub async fn post_method(required_paris: Vec<String>, dir_name: String) -> Result<String, reqwest::Error> {
  let client = reqwest::Client::new();
  let body = ProxyRequest {
    required_pairs: required_paris,
    dir_name: dir_name,
  };
  println!("client payload: {body:#?}");
  let response = client.post("https://fillup-translation.onrender.com")
    .json(&body)
    .send()
    .await?;
  eprintln!("client response: {response:#?}");
  let result = response.text().await?;
  eprintln!("text result: {result:#?}");
  Ok(result)
}
