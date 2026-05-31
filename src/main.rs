use crate::config::env_config::get_env_instance;
use crate::server::database::db;
use crate::server::database::db::init_db;
use crate::server::live::bid_manager::load_bid_list;
use crate::server::server::create_server;
mod config;
mod server;

#[tokio::main]
async fn main() {
    println!("Starting server");
    get_env_instance();
    init_db().await.expect("Error initializing database");
    load_bid_list().await;
    create_server("192.168.0.10".to_string(), "8080".to_string()).await;
}
