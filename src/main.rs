use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::env;
use url::Url;

// Fitbit OAuth 2.0 credentials
const CLIENT_ID: &str = "23RMML";
const CLIENT_SECRET: &str = "919c54d7f064b7756f00a4b8a26981d0";
// const REDIRECT_URL: &str = "http://localhost:8080/callback"; // Update with your redirect URL
const AUTH_URL: &str = "https://www.fitbit.com/oauth2/authorize";
const TOKEN_URL: &str = "https://api.fitbit.com/oauth2/token";

// Struct to represent a Fitbit heartbeat reading
#[derive(Debug, Deserialize)]
struct Heartbeat {
    time: String,
    value: i32,
}

#[tokio::main]
async fn main() {
    // Construct Fitbit OAuth2 client
    let client_id = ClientId::new(CLIENT_ID.to_string());
    let client_secret = ClientSecret::new(CLIENT_SECRET.to_string());
    let auth_url = AuthUrl::new(AUTH_URL.to_string()).unwrap();
    let token_url = TokenUrl::new(TOKEN_URL.to_string()).unwrap();

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url));
    // .set_redirect_url(RedirectUrl::new(REDIRECT_URL.to_string()).unwrap());

    // Generate and open the authorization URL
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("heartrate".to_string()))
        .url();

    println!(
        "Open this URL in your browser to authorize the application: {}",
        authorize_url
    );

    // Simulate a web server that waits for the callback with the authorization code
    // In a real application, you'd set up a web server to handle the redirect and obtain the authorization code
    let authorization_code = fetch_authorization_code();

    // Exchange the authorization code for an access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(authorization_code))
        .request_async(async_http_client)
        .await;

    match token_result {
        Ok(token_response) => {
            let access_token = token_response.access_token().secret();

            println!("Received access token: {:?}", access_token);

            // Fetch heartbeat data using the obtained access token
            match fetch_heartbeat_data(&access_token).await {
                Ok(heartbeats) => {
                    println!("Heartbeat data: {:?}", heartbeats);
                }
                Err(err) => {
                    eprintln!("Failed to fetch heartbeat data: {:?}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("Token exchange failed: {:?}", err);
        }
    }
}

// Simulate obtaining the authorization code from the callback URL (replace this with your actual code)
fn fetch_authorization_code() -> String {
    println!("Paste the authorization code obtained after granting access:");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

async fn fetch_heartbeat_data(access_token: &str) -> Result<Vec<Heartbeat>, reqwest::Error> {
    let client = reqwest::Client::new();

    let auth_header_value = format!("Bearer {}", access_token);

    let response = client
        .get("https://api.fitbit.com/1/user/-/activities/heart/date/today/1d.json")
        .header(reqwest::header::AUTHORIZATION, auth_header_value)
        .send()
        .await?;

    println!("response: {:?}", response);
    println!();
    println!();
    println!();
    println!();
    println!("response body: {}", response.text().await?);

    panic!("end");

    // if response.status().is_success() {
    //     let heartbeat_data: Vec<Heartbeat> = response.json().await?;
    //     Ok(heartbeat_data)
    // } else {
    //     panic!("shit broke");
    // }
}
