use anyhow::{Context, Result};
use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{channel, Sender};

// Fitbit OAuth 2.0 credentials
const CLIENT_ID: &str = "23RMML";
const CLIENT_SECRET: &str = "919c54d7f064b7756f00a4b8a26981d0";
const AUTH_URL: &str = "https://www.fitbit.com/oauth2/authorize";
const TOKEN_URL: &str = "https://api.fitbit.com/oauth2/token";

// Oauth2 callback query params
#[derive(Debug, Deserialize)]
struct Params {
    code: String,
    state: String,
}

#[derive(Clone)]
struct AppState(Sender<String>);

async fn callback(State(state): State<AppState>, Query(params): Query<Params>) -> String {
    // send the code through the channel
    state.0.send(params.code).await.unwrap();
    "ok".to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let (sender, mut receiver) = channel::<String>(1);

    // run axum in the background listening for callback requests
    tokio::spawn(async move {
        let app = Router::new()
            .route("/callback", get(callback))
            .with_state(AppState(sender));

        let bind = format!("0.0.0.0:{}", 8080);
        let listener = TcpListener::bind(&bind).await.unwrap();
        println!("listening on {}", &bind);
        axum::serve(listener, app).await.unwrap();
    });

    // Construct Fitbit OAuth2 client
    let client_id = ClientId::new(CLIENT_ID.to_string());
    let client_secret = ClientSecret::new(CLIENT_SECRET.to_string());
    let auth_url = AuthUrl::new(AUTH_URL.to_string())?;
    let token_url = TokenUrl::new(TOKEN_URL.to_string())?;

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url));

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
    let authorization_code = receiver
        .recv()
        .await
        .context("recv() did not return data")?;

    // Exchange the authorization code for an access token
    let token = client
        .exchange_code(AuthorizationCode::new(authorization_code))
        .request_async(async_http_client)
        .await?;

    let access_token = token.access_token().secret();

    let data = fetch_heartbeat_data(access_token).await?;

    println!("data: {}", data);
    Ok(())
}

async fn fetch_heartbeat_data(access_token: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let auth_header_value = format!("Bearer {}", access_token);

    let response = client
        .get("https://api.fitbit.com/1/user/-/activities/heart/date/today/1d.json")
        .header(reqwest::header::AUTHORIZATION, auth_header_value)
        .send()
        .await?;

    Ok(response.text().await?)
}
