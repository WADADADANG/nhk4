use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;
use std::path::{Path, PathBuf};

#[derive(RustEmbed)]
#[folder = "public/"]
pub struct PublicAssets;

#[derive(RustEmbed)]
#[folder = "launcher/ui/"]
pub struct LauncherAssets;

#[derive(RustEmbed)]
#[folder = "icons/"]
pub struct IconAssets;

pub fn guess_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

pub fn serve_embedded_file(path_name: &str, data: &[u8]) -> Response {
    let mime = guess_mime(Path::new(path_name));
    let mut res = data.to_vec().into_response();
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(mime),
    );
    res.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    res
}

pub async fn serve_file_from_disk(file_path: PathBuf) -> Response {
    if !file_path.exists() || !file_path.is_file() {
        return (StatusCode::NOT_FOUND, "404 Not Found").into_response();
    }

    match tokio::fs::read(&file_path).await {
        Ok(data) => {
            let mime = guess_mime(&file_path);
            let mut res = data.into_response();
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime),
            );
            res.headers_mut().insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            );
            res
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "500 Read Error").into_response(),
    }
}
