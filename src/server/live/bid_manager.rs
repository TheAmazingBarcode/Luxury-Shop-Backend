use crate::server::database::db::get_pool;
use crate::server::models::models::{Bid, InfoMessage, UpdateBid, UserPair};
use axum::Json;
use axum::http::StatusCode;
use bigdecimal::ToPrimitive;
use futures_util::StreamExt;
use jsonwebtoken::signature::digest::generic_array::functional::FunctionalSequence;
use sqlx::{MySql, QueryBuilder};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::{OnceCell, RwLock};
use tokio::time::sleep;

pub static BID_LIST: OnceCell<RwLock<HashMap<u32, Vec<Bid>>>> = OnceCell::const_new();

pub async fn load_bid_list() {
    BID_LIST
        .get_or_init(|| async {
            let mut list: HashMap<u32, Vec<Bid>> = HashMap::new();

            let rows = sqlx::query!(
                "SELECT bid_id, user_bidder_id, amount, auction_auction_id FROM bid INNER JOIN auction ON auction.auction_id = bid.auction_auction_id WHERE auction.is_complete = 0"
            )
                .fetch_all(get_pool())
                .await
                .unwrap_or_default();

            for row in rows {
                list.entry(row.auction_auction_id)
                    .or_insert_with(Vec::new)
                    .push(Bid {
                        id: row.bid_id,
                        amount: row.amount.to_f64().unwrap_or(0.0),
                        auction: row.auction_auction_id,
                        bidder: row.user_bidder_id,
                    });
            }

            RwLock::new(list)
        })
        .await;

    tokio::spawn(bid_worker());
}

pub async fn update_bid_list(bid_update: UpdateBid) {
    if let Some(lock) = BID_LIST.get() {
        let mut map = lock.write().await;
        if let Some(vec) = map.get_mut(&bid_update.auction) {
            let index_to_find = vec
                .iter()
                .position(|member| member.bidder == bid_update.bidder);

            if let Some(index) = index_to_find {
                if let Some(bid) = vec.get_mut(index) {
                    *bid = Bid {
                        id: bid.id,
                        bidder: bid.bidder,
                        auction: bid.auction,
                        amount: bid_update.amount,
                    };

                    println!("Bid updated in memory");

                    let result = sqlx::query!(
                        "INSERT INTO bid (user_bidder_id, auction_auction_id, amount) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE amount = IF(amount <> VALUES(amount), VALUES(amount), amount);",
                        bid_update.bidder,
                        bid_update.auction,
                        bid_update.amount
                    )
                    .execute(get_pool())
                    .await;

                    println!("Bid updated in database");
                }
            }
        }
    }
}

pub async fn conduct_transaction(auction_id: String) {
    write_bids_to_db().await;

    let result_pair : UserPair =
        sqlx::query!("SELECT auction_id,user_seller,user_bidder,amount FROM user_auction_max_pair WHERE auction_id = ?", auction_id)
            .fetch_one(get_pool())
            .await
            .map(|row| UserPair {
                seller_id: row.user_seller,
                bidder_id: row.user_bidder,
                amount: row.amount.expect("UNKNOWN AMOUNT").to_f64().unwrap_or(0.0),
            }).expect("INCOMPLETE RESULT");

    let _ = sqlx::query!(
        "INSERT INTO `transaction` (amount,isPositive,user_user_id,information) VALUES (?,?,?,?), (?,?,?,?)",
        result_pair.amount,
        true,
        result_pair.seller_id,
        "AUCTION_SELL",
         result_pair.amount,
        false,
        result_pair.bidder_id,
        "AUCTION_BUY"
    )
        .execute(get_pool())
        .await;

    let id_as_num: u32 = auction_id.parse().unwrap();

    let _ = sqlx::query!("UPDATE auction set final_price = ? WHERE auction.auction_id = ?", result_pair.amount,id_as_num)
        .execute(get_pool())
        .await;

    remove_from_list(auction_id).await;
}

pub async fn bid_worker() {
    loop {
        sleep(Duration::from_secs(30)).await;
        write_bids_to_db().await;
    }
}

pub async fn write_bids_to_db() {
    if let Some(lock) = BID_LIST.get() {
        let (bidders, auctions, amounts) = {
            let map = lock.read().await;
            if map.is_empty() {
                println!("BID LIST is empty");
                return;
            }
            let all_bids: Vec<&Bid> = map.values().flatten().collect();
            extract_bid_columns(&all_bids)
        };

        let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
            "INSERT IGNORE INTO bid (user_bidder_id, auction_auction_id, amount) ",
        );

        query_builder.push_values(
            bidders.iter().zip(auctions.iter()).zip(amounts.iter()),
            |mut b, ((bidder, auction), amount)| {
                b.push_bind(bidder).push_bind(auction).push_bind(amount);
            },
        );

        let _ = query_builder.build().execute(get_pool()).await;
    }
}

pub async fn get_bids_of_auction(auction_id: String) -> Vec<Bid> {
    let actual_id: u32 = auction_id.parse().expect("Auction id not a number");
    if let Some(lock) = BID_LIST.get() {
        let mut map = lock.write().await;
        map.get(&actual_id)
            .map_or_else(Vec::new, |vec| vec.to_vec())
    } else {
        Vec::new()
    }
}

pub async fn remove_from_list(auction_id: String) {
    if let Some(lock) = BID_LIST.get() {
        let mut map = lock.write().await;
        let auction_id_u32: u32 = auction_id.parse().expect("Auction ID must be u32");
        map.remove(&auction_id_u32);
    }
}

fn extract_bid_columns(bids: &[&Bid]) -> (Vec<i64>, Vec<i64>, Vec<f64>) {
    let bidders = bids.iter().map(|b| b.bidder as i64).collect();
    let auctions = bids.iter().map(|b| b.auction as i64).collect();
    let amounts = bids.iter().map(|b| b.amount).collect();

    (bidders, auctions, amounts)
}
async fn clear_bid_list() {
    if let Some(lock) = BID_LIST.get() {
        let mut map = lock.write().await;
        map.clear();
    }
}

pub async fn add_bid(bid: Bid) {
    if let Some(lock) = BID_LIST.get() {
        write_bids_to_db().await;
        let mut map = lock.write().await;

        if let Some(_val) = map.get(&bid.auction).and_then(|vec| {
            vec.iter()
                .find(|existing_bid| existing_bid.id == bid.bidder)
        }) {
            println!("Adding bid");
            map.entry(bid.auction).or_insert_with(Vec::new).push(bid);
        } else {
            map.entry(bid.auction).or_insert_with(Vec::new).push(bid);
        };
    }
}
