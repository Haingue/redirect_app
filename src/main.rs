use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use std::{net::SocketAddr, sync::Arc};

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

struct AppConfig {
    redirect_url: String,
    message_title: String,
    message: String,
    delay_seconds: u32,
}

/// Escape a string for safe inclusion inside an HTML attribute / text node.
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn load_config() -> AppConfig {
    // Load a `config.env` file located next to the executable, if present.
    // This lets operators change settings without rebuilding or touching the code.
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let config_path = exe_dir.join("config.env");
            let _ = dotenvy::from_path(config_path);
        }
    }
    // Fallback: also try a config.env in the current working directory.
    let _ = dotenvy::from_filename("config.env");

    let redirect_url = std::env::var("REDIRECT_URL")
        .unwrap_or_else(|_| "https://example.com".to_string());
    let message_title = std::env::var("REDIRECT_MESSAGE_TITLE")
        .unwrap_or_else(|_| "Cette page a déménagé".to_string());
    let message = std::env::var("REDIRECT_MESSAGE").unwrap_or_else(|_| {
        "Cette application a été déplacée vers une nouvelle adresse. \
         Merci de mettre à jour vos favoris."
            .to_string()
    });
    let delay_seconds = std::env::var("REDIRECT_DELAY_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(10);

    AppConfig {
        redirect_url,
        message_title,
        message,
        delay_seconds,
    }
}

async fn index_handler(State(config): State<Arc<AppConfig>>) -> impl IntoResponse {
    let template = Templates::get("index.html")
        .map(|f| String::from_utf8_lossy(&f.data).into_owned())
        .unwrap_or_else(|| "<h1>Template introuvable</h1>".to_string());

    let html = template
        .replace("__REDIRECT_URL__", &html_escape(&config.redirect_url))
        .replace("__MESSAGE_TITLE__", &html_escape(&config.message_title))
        .replace("__MESSAGE__", &html_escape(&config.message))
        .replace("__DELAY__", &config.delay_seconds.to_string());

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

async fn tailwind_handler() -> impl IntoResponse {
    match Assets::get("tailwind.min.css") {
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
            .body(Body::from(file.data.into_owned()))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}

#[tokio::main]
async fn main() {
    let config = Arc::new(load_config());

    let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let addr: SocketAddr = listen_addr
        .parse()
        .unwrap_or_else(|_| "0.0.0.0:8080".parse().unwrap());

    println!("Configuration chargée :");
    println!("  REDIRECT_URL          = {}", config.redirect_url);
    println!("  REDIRECT_MESSAGE_TITLE= {}", config.message_title);
    println!("  REDIRECT_MESSAGE      = {}", config.message);
    println!("  REDIRECT_DELAY_SECONDS= {}", config.delay_seconds);
    println!("Serveur démarré sur http://{}", addr);

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/tailwind.min.css", get(tailwind_handler))
        .route("/{*key}", get(index_handler))
        .with_state(config);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "Erreur : impossible d'écouter sur {} ({}).\n\
                 Vérifiez que le port n'est pas déjà utilisé et que LISTEN_ADDR \
                 dans config.env est correct (ex: 0.0.0.0:8080). Sous Windows, \
                 certains ports peuvent être réservés par le système \
                 (voir `netsh interface ipv4 show excludedportrange protocol=tcp`).",
                addr, e
            );
            std::process::exit(1);
        }
    };
    axum::serve(listener, app).await.unwrap();
}
