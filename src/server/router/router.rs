use crate::server::jwt::jwt::load_jwt;
use crate::server::live::bid_routes::bid_router;
use crate::server::routes::auction_routes::auction_router;
use crate::server::routes::category_routes::category_router;
use crate::server::routes::product_routes::product_router;
use crate::server::routes::transaction_routes::transaction_router;
use crate::server::routes::user_routes::user_router;
use axum::Router;
use tower_http::cors::CorsLayer;

pub async fn create_router() -> Router {
    load_jwt().await;
    Router::new()
        .nest("/funds", transaction_router())
        .nest("/user", user_router())
        .nest("/categories", category_router())
        .nest("/products", product_router())
        .nest("/auctions", auction_router())
        .nest("/socket", bid_router())
        .layer(CorsLayer::permissive())
}
