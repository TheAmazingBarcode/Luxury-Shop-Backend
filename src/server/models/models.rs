use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use std::iter::Product;

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ValidateUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserView {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub balance: f64,
}

#[derive(Deserialize)]
pub struct CreateTransaction {
    pub user_id: u32,
    pub amount: f64,
    pub is_positive: bool,
    pub information: String,
}

#[derive(Serialize)]
pub struct TransactionView {
    pub id: u32,
    pub amount: f64,
    pub is_positive: bool,
    pub user_id: u32,
    pub information: String,
}

#[derive(Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub description: String,
    pub category_id: u32,
}

#[derive(Deserialize)]
pub struct CreateCategory {
    pub name: String,
}

#[derive(Serialize)]
pub struct ProductView {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub category: CategoryView,
}

pub struct UserPair {
    pub seller_id: u32,
    pub bidder_id: u32,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct CategoryView {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateAuction {
    pub starting_price: f64,
    pub product_sale_id: u32,
    pub user_seller_id: u32,
}

#[derive(Serialize)]
pub struct AuctionView {
    pub id: u32,
    pub product_name: String,
    pub product_description: String,
    pub product_id: u32,
    pub user_id: u32,
    pub username: String,
    pub product_starting_price: f64,
    pub is_complete: bool,
    pub final_price: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Bid {
    pub id: u32,
    pub bidder: u32,
    pub auction: u32,
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct UpdateBid {
    pub auction: u32,
    pub bidder: u32,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct ModelCreationResponse {
    pub type_of_model: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct InfoMessage {
    pub message: String,
}
