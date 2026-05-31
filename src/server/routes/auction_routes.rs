use crate::server::database::db::get_pool;
use crate::server::live::bid_manager::get_bids_of_auction;
use crate::server::live::bid_routes::broadcast_externally;
use crate::server::models::models::{
    AuctionView, Bid, CreateAuction, InfoMessage, ModelCreationResponse,
};
use crate::server::routes::middleware::jwt_authenticatior;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use bigdecimal::ToPrimitive;
use regex::Regex;

pub fn auction_router() -> Router {
    Router::new()
        .route("/new", post(create_auction))
        .route("/end/{id}", put(end_auction))
        .layer(from_fn(jwt_authenticatior))
        .route("/all", get(get_all_auctions))
        .route("/product/{id}", get(auction_of_product))
        .route("/bid_list/{id}", get(get_bids))
}

async fn get_all_auctions() -> Result<Json<Vec<AuctionView>>, StatusCode> {
    let rows = sqlx::query!("SELECT * FROM auction_view")
        .fetch_all(get_pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let products: Vec<AuctionView> = rows
        .into_iter()
        .map(|row| AuctionView {
            id: row.auction_id,
            product_id: row.product_id,
            product_name: row.name,
            product_description: row.description.unwrap_or_else(|| "UNKNOWN".to_string()),
            user_id: row.user_seller_id,
            username: row.username,
            product_starting_price: row.starting_price.to_f64().unwrap_or(0.0),
            is_complete: row.is_complete != 0,
            final_price: row.final_price.map_or(0.0, |v| v.to_f64().unwrap_or(0.0)),
        })
        .collect::<Vec<AuctionView>>();

    Ok(Json(products))
}

async fn create_auction(Json(payload): Json<CreateAuction>) -> Json<ModelCreationResponse> {
    let _result = sqlx::query!(
        "INSERT INTO auction (starting_price,product_sale_id,user_seller_id) VALUES (?,?,?)",
        payload.starting_price,
        payload.product_sale_id,
        payload.user_seller_id,
    )
    .execute(get_pool())
    .await;

    Json(ModelCreationResponse {
        type_of_model: "Auction".to_string(),
        message: "Success".to_string(),
    })
}

async fn auction_of_product(request: Request) -> Result<Json<AuctionView>, StatusCode> {
    let path_reg = Regex::new(r"/\d+$").unwrap();

    if let Some(caps) = path_reg.captures(request.uri().path()) {
        let id_in_url = &caps.get(0).unwrap().as_str()[1..];

        let product = sqlx::query!(
            "SELECT * FROM auction_view where auction_view.product_id = ?",
            id_in_url
        )
        .fetch_one(get_pool())
        .await
        .map(|row| AuctionView {
            id: row.auction_id,
            product_id: row.product_id,
            product_name: row.name,
            product_description: row.description.unwrap_or_else(|| "UNKNOWN".to_string()),
            user_id: row.user_seller_id,
            username: row.username,
            product_starting_price: row.starting_price.to_f64().unwrap_or(0.0),
            is_complete: row.is_complete != 0,
            final_price: row.final_price.map_or(0.0, |v| v.to_f64().unwrap_or(0.0)),
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(product))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_bids(request: Request) -> Result<Json<Vec<Bid>>, StatusCode> {
    let path_reg = Regex::new(r"/\d+$").unwrap();

    if let Some(caps) = path_reg.captures(request.uri().path()) {
        let id_in_url = &caps.get(0).unwrap().as_str()[1..];

        let bid_list: Vec<Bid> = get_bids_of_auction(id_in_url.into()).await;
        Ok(Json(bid_list))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn end_auction(request: Request) -> Result<Json<InfoMessage>, StatusCode> {
    let path_reg = Regex::new(r"/\d+$").unwrap();

    if let Some(caps) = path_reg.captures(request.uri().path()) {
        let id_in_url = &caps.get(0).unwrap().as_str()[1..];
        let _ = sqlx::query!(
            "UPDATE auction set is_complete = 1 where auction_id = ?",
            id_in_url
        )
        .execute(get_pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut msg: String = String::new();
        msg.push_str("END_AUCTION-");
        msg.push_str(id_in_url);

        broadcast_externally(msg).await;
    }

    Ok(Json(InfoMessage {
        message: "Auction succesfully ended".to_string(),
    }))
}
