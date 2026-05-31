use crate::server::database::db::get_pool;
use crate::server::models::models::{
    CategoryView, CreateCategory, ModelCreationResponse, ProductView,
};
use crate::server::routes::middleware::jwt_authenticatior;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::routing::{get, post};
use axum::{Json, Router};

pub fn category_router() -> Router {
    Router::new()
        .route("/new", post(create_category))
        .layer(from_fn(jwt_authenticatior))
        .route("/all", get(get_all_categories))
}

async fn create_category(Json(payload): Json<CreateCategory>) -> Json<ModelCreationResponse> {
    let result = sqlx::query!("INSERT INTO category (name) VALUES (?)", payload.name)
        .execute(get_pool())
        .await;

    Json(ModelCreationResponse {
        type_of_model: "Category".to_string(),
        message: "Success".to_string(),
    })
}

async fn get_all_categories() -> Result<Json<Vec<CategoryView>>, StatusCode> {
    let rows = sqlx::query!("SELECT * FROM category")
        .fetch_all(get_pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let categories: Vec<CategoryView> = rows
        .into_iter()
        .map(|row| CategoryView {
            id: row.category_id,
            name: row.name,
        })
        .collect();

    Ok(Json(categories))
}
