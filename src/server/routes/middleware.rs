use crate::server::jwt::jwt::UTIL;
use crate::server::routes::header_util::extract_bearer_token;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

pub async fn jwt_authenticatior(request: Request, next: Next) -> Result<Response, StatusCode> {
    let token = match extract_bearer_token(request.headers()) {
        Ok(t) => t,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    let data = match UTIL.get().unwrap().decode(token.to_string()) {
        Ok(data) => data,
        Err(e) => {
            println!("JWT Decode Error: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let response = next.run(request).await;
    Ok(response)
}
