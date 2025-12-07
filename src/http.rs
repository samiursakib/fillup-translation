use dotenv;
use serde::{Serialize};

#[derive(Serialize, Debug)]
struct ProxyRequest {
  required_pairs: Vec<String>,
  dir_name: String,
}

pub async fn post_method(required_paris: Vec<String>, dir_name: String) -> Result<String, reqwest::Error> {
  dotenv::dotenv().ok();
  let client = reqwest::Client::new();
  let body = ProxyRequest {
    required_pairs: required_paris,
    dir_name: dir_name,
  };

  let request_url = "https://fillup-translation.onrender.com/api/translate";
  let response = client.post(request_url)
    .json(&body)
    .send()
    .await?;
  let result = response.text().await?;
  Ok(result)
}
