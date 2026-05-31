use crate::server::live::bid_manager::{add_bid, conduct_transaction, remove_from_list, update_bid_list};
use crate::server::models::models::{Bid, UpdateBid};
use axum::Router;
use axum::extract::State;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use serde::Deserialize;
use std::sync::{Arc, OnceLock};
use tokio::sync::broadcast;

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
enum IncomingMessage {
    Bid(Bid),
    UpdateBid(UpdateBid),
}

struct AppState {
    tx: broadcast::Sender<String>,
}

pub static BID_BROADCASTER: OnceLock<broadcast::Sender<String>> = OnceLock::new();

pub fn bid_router() -> Router {
    let (tx, _rx) = broadcast::channel(100);

    BID_BROADCASTER
        .set(tx.clone())
        .expect("BID_BROADCASTER incorrectly initialized");

    let state = Arc::new(AppState { tx });

    Router::new()
        .route("/bids", get(websocket_handler))
        .with_state(state)
}

pub async fn broadcast_externally(msg: String) {
    match BID_BROADCASTER.get() {
        Some(tx) => {
            if let Some((type_of_msg, content)) = msg.split_once("-") {
                if(type_of_msg == "END_AUCTION") {
                    conduct_transaction(content.to_string()).await;
                    let _ = tx.send(msg);
                }
            } else {
            }
        }
        None => {
            println!("BID_BROADCASTER not initialized");
        }
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_failed_upgrade(|error| println!("Connection not established: {}", error))
        .on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();

    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(utf8_bytes)) => {
                    match serde_json::from_str::<IncomingMessage>(utf8_bytes.as_ref()) {
                        Ok(IncomingMessage::Bid(bid)) => {
                            println!(
                                "Bid received — bid_id: {}, bidder: {}, auction: {}, amount: {:.2}",
                                bid.id, bid.bidder, bid.auction, bid.amount
                            );
                            let broadcast_msg =
                                format!("NEW_BID-{}:{:.2}:{}", bid.auction, bid.amount, bid.bidder);
                            add_bid(bid).await;
                            let _ = tx.send(broadcast_msg);
                        }
                        Ok(IncomingMessage::UpdateBid(update)) => {
                            println!(
                                "UpdateBid received — bidder_id: {}, auction: {}, new amount: {:.2}",
                                update.bidder, update.auction, update.amount
                            );
                            let broadcast_msg = format!(
                                "UPDATE_BID-{}:{}:{:.2}",
                                update.bidder, update.auction, update.amount
                            );
                            update_bid_list(update).await;
                            let _ = tx.send(broadcast_msg);
                        }
                        Err(parse_error) => {
                            println!("Failed to parse message JSON: {}", parse_error);
                            break;
                        }
                    }
                }
                Ok(_) => {}
                Err(error) => {
                    println!("Error receiving message: {:?}", error);
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };
}
