use crate::server::database::db::get_pool;
use crate::server::jwt::jwt::UTIL;
use crate::server::models::models::{
    AuctionView, CreateAuction, CreateTransaction, InfoMessage, ModelCreationResponse,
    TransactionView,
};
use crate::server::routes::header_util::extract_bearer_token;
use crate::server::routes::middleware::jwt_authenticatior;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router, middleware};
use bigdecimal::ToPrimitive;
use regex::Regex;

pub fn transaction_router() -> Router {
    Router::new()
        .route("/add/{id}", post(add_funds))
        .route("/user/{id}", get(get_transactions_of_user))
        .layer(middleware::from_fn(jwt_authenticatior))
}

async fn add_funds(request: Request) -> Result<Json<ModelCreationResponse>, StatusCode> {
    let token = extract_bearer_token(request.headers())?;
    let email_in_token = UTIL
        .get()
        .unwrap()
        .decode(token.to_string())
        .unwrap()
        .claims
        .sub;

    let path_reg = Regex::new(r"/\d+$").unwrap();

    if let Some(caps) = path_reg.captures(request.uri().path()) {
        let id_in_url = &caps.get(0).unwrap().as_str()[1..];
        let email_of_user: String =
            sqlx::query_scalar!("SELECT email FROM  user WHERE user_id = ?", id_in_url)
                .fetch_one(get_pool())
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;

        println!("{}", email_of_user);

        if email_of_user != email_in_token {
            Err(StatusCode::FORBIDDEN)
        } else {
            let body = axum::body::to_bytes(request.into_body(), 1024 * 1024 * 10)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            let payload: CreateTransaction =
                serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

            let result = sqlx::query!(
        "INSERT INTO luxury_shop.`transaction` (amount,isPositive,user_user_id,information) VALUES (?,?,?,?)"
                ,payload.amount,payload.is_positive
                ,payload.user_id,"DEPOSIT",
            )
                .execute(get_pool())
                .await;
            Ok(Json(ModelCreationResponse {
                type_of_model: "Transaction".to_string(),
                message: "Success".to_string(),
            }))
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn get_transactions_of_user(
    request: Request,
) -> Result<Json<Vec<TransactionView>>, StatusCode> {
    let path_reg = Regex::new(r"/\d+$").unwrap();

    if let Some(caps) = path_reg.captures(request.uri().path()) {
        let id_in_url = &caps.get(0).unwrap().as_str()[1..];

        let rows = sqlx::query!(
            "SELECT * FROM `transaction` where `transaction`.user_user_id = ?",
            id_in_url
        )
        .fetch_all(get_pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let transactions: Vec<TransactionView> = rows
            .into_iter()
            .map(|row| TransactionView {
                id: row.transaction_id,
                amount: row.amount.to_f64().unwrap_or(0.0),
                information: row.information.unwrap_or_else(|| "UNKNOWN".to_string()),
                user_id: row.user_user_id,
                is_positive: row.isPositive != 0,
            })
            .collect::<Vec<TransactionView>>();

        Ok(Json(transactions))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
