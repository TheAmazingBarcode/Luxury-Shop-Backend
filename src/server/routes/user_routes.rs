use crate::server::database::db::get_pool;
use crate::server::jwt::jwt::{Claims, UTIL};
use crate::server::models::models::{
    CreateUser, InfoMessage, TokenResponse, UserPair, UserView, ValidateUser,
};
use crate::server::routes::header_util::extract_bearer_token;
use crate::server::routes::middleware::jwt_authenticatior;
use axum::extract::Request;
use axum::middleware::from_fn;
use axum::routing::{get, post, put};
use axum::{Json, Router, http::StatusCode, response::Result};
use bcrypt::{DEFAULT_COST, hash, verify};
use bigdecimal::ToPrimitive;
use chrono::{Duration, Utc};
use sqlx::Executor;

pub fn user_router() -> Router {
    Router::new()
        .route("/info", get(get_user))
        .layer(from_fn(jwt_authenticatior))
        .route("/register", post(register_user))
        .route("/login", put(log_in_user))
}

async fn get_user(request: Request) -> Result<Json<UserView>, StatusCode> {
    let token = extract_bearer_token(request.headers())?;
    let email_in_token = UTIL
        .get()
        .unwrap()
        .decode(token.to_string())
        .unwrap()
        .claims
        .sub;

    let user: UserView = sqlx::query!("SELECT * FROM user_view WHERE email = ?", email_in_token)
        .fetch_one(get_pool())
        .await
        .map(|row| UserView {
            id: row.user_id,
            username: row.username,
            email: row.email,
            balance: row.balance.map_or(0.0, |v| v.to_f64().unwrap_or(0.0)),
        })
        .expect("INCOMPLETE RESULT");

    Ok(Json(user))
}

async fn register_user(Json(payload): Json<CreateUser>) -> Json<TokenResponse> {
    let result = sqlx::query!(
        "INSERT INTO user (username,email,password) VALUES (?,?,?)",
        payload.username,
        payload.email,
        hash(payload.password, DEFAULT_COST).expect("Failed to hash password."),
    )
    .execute(get_pool())
    .await;

    let [access, refresh] = generate_token_pair(payload.email).await;

    Json(TokenResponse {
        access_token: access,
        refresh_token: refresh,
    })
}

async fn log_in_user(
    Json(payload): Json<ValidateUser>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<InfoMessage>)> {
    let result: String =
        sqlx::query_scalar!("SELECT password FROM `user` WHERE email = ?", payload.email)
            .fetch_one(get_pool())
            .await
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(InfoMessage {
                        message: "User does not exist".to_string(),
                    }),
                )
            })?;

    let password_result: String = result;

    if verify(payload.password, password_result.as_str()).expect("Failed to verify password.") {
        let [access, refresh] = generate_token_pair(payload.email).await;

        Ok(Json(TokenResponse {
            access_token: access,
            refresh_token: refresh,
        }))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(InfoMessage {
                message: "Invalid password".to_string(),
            }),
        ))
    }
}

async fn generate_token_pair(email: String) -> [String; 2] {
    let mut access_email = String::new();
    let mut refresh_email = String::new();

    email.clone_into(&mut access_email);
    email.clone_into(&mut refresh_email);

    let expiration_refresh = (Utc::now() + Duration::seconds(3600)).timestamp() as usize;
    let expiration_access = (Utc::now() + Duration::seconds(900)).timestamp() as usize;

    let claims_access = Claims {
        sub: access_email,
        exp: expiration_access,
        token_type: "ACCESS_TOKEN".to_string(),
    };

    let refresh_token = Claims {
        sub: refresh_email,
        exp: expiration_refresh,
        token_type: "REFRESH_TOKEN".to_string(),
    };

    let access = UTIL.get().unwrap().encode(&claims_access);
    let refresh = UTIL.get().unwrap().encode(&refresh_token);

    [access, refresh]
}
