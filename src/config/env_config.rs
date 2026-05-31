use dotenvy::dotenv;
use std::path::PathBuf;
use std::sync::OnceLock;

static ENV_INSTANCE: OnceLock<PathBuf> = OnceLock::new();

pub fn get_env_instance() -> &'static PathBuf {
    let path = dotenv().expect(".env file missing");
    println!("LOADING ENV FILE");
    ENV_INSTANCE.get_or_init(|| path)
}
