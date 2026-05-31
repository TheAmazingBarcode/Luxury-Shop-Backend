use crate::server::database::db::get_pool;
use crate::server::models::models::{
    CategoryView, CreateProduct, ModelCreationResponse, ProductView,
};
use crate::server::routes::middleware::jwt_authenticatior;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::routing::{get, post};
use axum::{Json, Router};
use regex::Regex;

pub fn product_router() -> Router {
    Router::new()
        .route("/new", post(create_product))
        .layer(from_fn(jwt_authenticatior))
        .route("/all", get(get_all_products))
        .route("/category/{id}", get(get_products_of_category))
}

async fn create_product(Json(payload): Json<CreateProduct>) -> Json<ModelCreationResponse> {
    let result = sqlx::query!(
        "INSERT INTO product (name,description,category_category_id) VALUES (?,?,?)",
        payload.name,
        payload.description,
        payload.category_id,
    )
    .execute(get_pool())
    .await;

    Json(ModelCreationResponse {
        type_of_model: "Product".to_string(),
        message: "Success".to_string(),
    })
}

async fn get_all_products() -> Result<Json<Vec<ProductView>>, StatusCode> {
    let rows = sqlx::query!("SELECT * FROM product_view")
        .fetch_all(get_pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let products: Vec<ProductView> = rows
        .into_iter()
        .map(|row| ProductView {
            id: row.product_id,
            name: row.name,
            description: row.description.unwrap_or_else(|| "UNKNOWN".to_string()),
            category: CategoryView {
                id: row.category_category_id,
                name: row.category_name,
            },
        })
        .collect();

    Ok(Json(products))
}

async fn get_products_of_category(request: Request) -> Result<Json<Vec<ProductView>>, StatusCode> {
    let path_reg = Regex::new(r"/\d+$").unwrap();

    if let Some(caps) = path_reg.captures(request.uri().path()) {
        let id_in_url = &caps.get(0).unwrap().as_str()[1..];
        let rows = sqlx::query!(
            "SELECT * FROM product_view where product_view.category_category_id = ?",
            id_in_url
        )
        .fetch_all(get_pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let products: Vec<ProductView> = rows
            .into_iter()
            .map(|row| ProductView {
                id: row.product_id,
                name: row.name,
                description: row.description.unwrap_or_else(|| "UNKNOWN".to_string()),
                category: CategoryView {
                    id: row.category_category_id,
                    name: row.category_name,
                },
            })
            .collect();
        Ok(Json(products))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
