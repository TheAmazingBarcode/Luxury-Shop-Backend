use crate::server::router::router::create_router;

pub async fn create_server(addr:String,port:String) {
    let app = create_router().await;
    let listener = tokio::net::TcpListener::bind(addr+":"+port.as_str()).await.unwrap();
    axum::serve(listener, app).await.unwrap()
}
