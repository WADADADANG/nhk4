use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use std::path::{Path, PathBuf};

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
