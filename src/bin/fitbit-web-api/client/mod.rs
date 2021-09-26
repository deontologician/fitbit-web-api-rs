//! Top level client API for communicating with Fitbit.

use std::io::Write;

use fitbit_web_api::*;
use log::*;

pub(super) mod activity;
pub(super) mod body;
pub(super) mod heart_rate;
mod oauth;
pub(super) mod sleep;

// For now, just record the token to a file.
const TOKEN_FILE: &str = "auth_token";

fn write_auth_token(token: String) {
    let mut file = std::fs::File::create(TOKEN_FILE).unwrap();
    file.write_all(token.as_bytes()).unwrap();
}

fn read_auth_token() -> String {
    match std::fs::read_to_string(TOKEN_FILE) {
        Ok(token) => token,
        Err(e) => {
            eprintln!(
                "Failed to read the auth token ({})\nHave you run the `auth` command?",
                e
            );
            std::process::exit(1);
        }
    }
}

async fn get(url: url::Url) -> String {
    let token: String = read_auth_token();

    let client = reqwest::Client::new();
    info!("GET {:?}", url);

    let res = client.get(url).bearer_auth(token).send().await.unwrap();
    let status = res.status();
    if !status.is_success() {
        eprintln!("Bad HTTP status code: {}", status);
        match status {
            reqwest::StatusCode::UNAUTHORIZED => {
                eprintln!("Check that your API token is correct?");
            },
            reqwest::StatusCode::BAD_REQUEST => {
                eprintln!("Error: {}", res.text().await.unwrap())
            }
            _ => (),
        };
        std::process::exit(1);
    };

    let body = res.text().await.unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    info!("{:?}", v);

    body
}

pub(super) fn get_auth_token(id: String, secret: String) {
    let token = oauth::get_token(id, secret);
    write_auth_token(token);
    println!("Success! OAuth2 token recorded to {}.", TOKEN_FILE);
}

pub(super) async fn get_badges() {
    let url = user::badges::get::url(UserId::Current);
    let body = get(url).await;
    let badges: user::badges::get::Response = serde_json::from_str(&body).unwrap();
    println!("{}", badges);
}

pub(super) async fn get_devices() {
    let url = devices::get::url(UserId::Current);
    let body = get(url).await;
    let devices: devices::get::Response = serde_json::from_str(&body).unwrap();
    println!("{}", devices);
}

pub(super) async fn get_profile() {
    let url = user::profile::url(UserId::Current);
    let body = get(url).await;
    let profile: user::profile::Response = serde_json::from_str(&body).unwrap();
    println!("{}", profile);
}
