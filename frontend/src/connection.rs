use gloo_net::http::Request;
use serde::{Serialize, de::DeserializeOwned};

pub async fn post_from_db<T, W>(url: &str, json: T) -> Option<W> 
where T: Serialize,
      W: DeserializeOwned,
{
    let response = Request::post(url)
        .json(&json)
        .unwrap()
        .send()
        .await;

    if let Ok(response) = response {
        if response.ok() {
            let get: W = response.json().await.unwrap();
            return Some(get)
        }
    }

    None
}
