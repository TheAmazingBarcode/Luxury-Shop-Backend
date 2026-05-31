use crate::server::live::bid_manager::load_bid_list;
use sqlx::{MySql, MySqlPool};
use std::env;
use std::sync::OnceLock;

static DB_POOL: OnceLock<MySqlPool> = OnceLock::new();

pub async fn init_db() -> Result<(), sqlx::Error> {
    let pool = MySqlPool::connect(&env::var("DATABASE_URL").unwrap().as_str()).await?;
    println!("Connected to database");

    DB_POOL.set(pool).map_err(|_| {
        sqlx::Error::Configuration(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Database pool already initialized",
        )))
    })

}

pub fn get_pool() -> &'static MySqlPool {
    DB_POOL
        .get()
        .expect("Database not initialized. Call init_db() first.")
}
