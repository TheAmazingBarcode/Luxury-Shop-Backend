use axum::http::{HeaderMap, StatusCode};

pub fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, StatusCode> {
    let header_str = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if !header_str.starts_with("Bearer ") {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(header_str.trim_start_matches("Bearer "))
}
