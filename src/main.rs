use anyhow::{Context, Result};
use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use log::info;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{channel, Sender};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, env)]
    client_id: String,
    #[clap(long, env)]
    client_secret: String,
    #[clap(long, env, default_value = "https://www.fitbit.com/oauth2/authorize")]
    auth_url: String,
    #[clap(long, env, default_value = "https://api.fitbit.com/oauth2/token")]
    token_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TokenStorage {
    access_token: String,
}

// Oauth2 callback query params
#[derive(Debug, Deserialize)]
struct Params {
    code: String,
    // state: String,
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
    simple_logger::init_with_level(log::Level::Info)?;
    let args = Args::parse();
    let (sender, mut receiver) = channel::<String>(1);

    let token_storage_file = "access_token.json";
    let mut access_token = String::new();

    // Try to read the access token from storage
    if let Ok(token_storage_content) = fs::read_to_string(token_storage_file).await {
        let token_storage: TokenStorage = serde_json::from_str(&token_storage_content)?;
        access_token = token_storage.access_token;
    }

    // run axum in the background listening for callback requests
    tokio::spawn(async move {
        let app = Router::new()
            .route("/callback", get(callback))
            .with_state(AppState(sender));

        let bind = format!("0.0.0.0:{}", 8080);
        let listener = TcpListener::bind(&bind).await.unwrap();
        info!("listening on {}", &bind);
        axum::serve(listener, app).await.unwrap();
    });

    let client = BasicClient::new(
        ClientId::new(args.client_id),
        Some(ClientSecret::new(args.client_secret)),
        AuthUrl::new(args.auth_url)?,
        Some(TokenUrl::new(args.token_url)?),
    );

    let scopes = vec![
        // "activity",
        // "cardio_fitness",
        // "electrocardiogram",
        "heartrate",
        // "location",
        // "nutrition",
        // "oxygen_saturation",
        // "profile",
        // "respiratory_rate",
        // "settings",
        // "sleep",
        // "social",
        // "temperature",
        // "weight",
    ];

    if access_token.is_empty() {
        // Generate and open the authorization URL
        let (authorize_url, _csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes.into_iter().map(|s| Scope::new(s.to_string())))
            .url();

        info!(
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

        access_token = token.access_token().secret().to_string();
        // Store the access token
        let token_storage = TokenStorage {
            access_token: access_token.clone(),
        };
        let token_storage_content = serde_json::to_string(&token_storage)?;
        fs::write(token_storage_file, token_storage_content).await?;
    }

    let data: serde_json::Value =
        serde_json::from_str(&fetch_heartbeat_data(&access_token).await?)?;
    info!("{}", serde_json::to_string_pretty(&data)?);
    Ok(())
}

async fn fetch_heartbeat_data(access_token: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let auth_header_value = format!("Bearer {}", access_token);

    let response = client
        .get("https://api.fitbit.com/1/user/-/activities/heart/date/today/1d/1min.json")
        .header(reqwest::header::AUTHORIZATION, auth_header_value)
        .send()
        .await?;

    Ok(response.text().await?)
}
