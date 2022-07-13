use crate::*;
extern crate chrono;

use chrono::prelude::*;
use near_sdk::json_types::Base64VecU8;
use near_sdk::{env};
use std::collections::HashMap;

const STYLES_BODY: &str = include_str!("../res/style.css");

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Web4Request {
    #[serde(rename = "accountId")]
    account_id: Option<AccountId>,
    path: String,
    params: Option<HashMap<String, String>>,
    query: Option<HashMap<String, Vec<String>>>,
    preloads: Option<HashMap<String, Web4Response>>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct Web4Response {
    #[serde(rename = "contentType")]
    content_type: Option<String>,
    status: Option<u32>,
    body: Option<Base64VecU8>,
    #[serde(rename = "bodyUrl")]
    body_url: Option<String>,
    #[serde(rename = "preloadUrls")]
    preload_urls: Option<Vec<String>>,
}

impl Web4Response {
    pub fn html_response(text: String) -> Self {
        Self {
            content_type: Some(String::from("text/html; charset=UTF-8")),
            body: Some(text.into_bytes().into()),
            ..Default::default()
        }
    }

    pub fn plain_response(text: String) -> Self {
        Self {
            content_type: Some(String::from("text/plain; charset=UTF-8")),
            body: Some(text.into_bytes().into()),
            ..Default::default()
        }
    }

    pub fn preload_urls(urls: Vec<String>) -> Self {
        Self {
            preload_urls: Some(urls),
            ..Default::default()
        }
    }

    pub fn body_url(url: String) -> Self {
        Self {
            body_url: Some(url),
            ..Default::default()
        }
    }

    pub fn status(status: u32) -> Self {
        Self {
            status: Some(status),
            ..Default::default()
        }
    }
}

#[near_bindgen]
impl Contract {
    #[allow(unused_variables)]
    pub fn web4_get(&self, request: Web4Request) -> Web4Response {
        let path = request.path;

        if path == "/robots.txt" {
            return Web4Response::plain_response("User-agent: *\nDisallow:".to_string());
        }

        if path == "/new" {
            return Web4Response::html_response(
                include_str!("../res/new.html")
                    .replace("%STYLESHEET%", &STYLES_BODY)
                    .replace("%CONTRACT_ID%", &env::current_account_id().to_string())
                    .replace("%NETWORK%", "testnet")
            );
        }

        let mut post_html = "".to_string();
        for post in self.get_posts().iter().rev() {
            if let post = post {
                post_html = format!("{}<div class='data'><p><strong>{}</strong></p><p>{}</p><p><i><small>by <span>{}</span> on <span>{}</span></small></i></p></div>", &post_html,
                    post.1.title,
                    post.1.text,
                    post.1.author,
                    NaiveDateTime::from_timestamp(post.1.created_at as i64/1000000000, 0),
                );
            }
        };

        Web4Response::html_response(
            include_str!("../res/index.html")
                .replace("%STYLESHEET%", &STYLES_BODY)
                .replace("%POSTS%", &post_html)
                .replace("%CONTRACT_ID%", &env::current_account_id().to_string())
                .replace("%NETWORK%", "testnet")
        )

    }
}
