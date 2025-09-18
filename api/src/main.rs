use axum::{routing::get, extract::Path, Json, Router};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    supply_total: String,
    supply_circ: String,
}

#[derive(Serialize)]
struct Supply { total: String, circulating: String }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new().max_connections(8)
        .connect(&std::env::var("DATABASE_URL")?).await?;

    let cors = CorsLayer::new()
        .allow_origin(std::env::var("CORS_ORIGIN").unwrap_or("*".into()).parse::<http::HeaderValue>().ok().into_iter().collect::<Vec<_>>())
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState {
        pool,
        supply_total: std::env::var("SUPPLY_TOTAL").unwrap_or("22000000".into()),
        supply_circ: std::env::var("SUPPLY_CIRC").unwrap_or("22000000".into()),
    };

    let app = Router::new()
        .route("/status", get(status))
        .route("/blocks", get(list_blocks))
        .route("/block/:height", get(get_block))
        .route("/tx/:txid", get(get_tx))
        .route("/address/:addr", get(get_address))
        .route("/search/:q", get(search_any))
        .route("/supply", get(supply))
        .with_state(state)
        .layer(cors);

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn status(state: axum::extract::State<AppState>) -> Json<serde_json::Value> {
    let (tip_height,): (i64,) = sqlx::query_as("SELECT tip_height FROM chain_meta WHERE id=1")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    Json(serde_json::json!({"ok": true, "tip_height": tip_height}))
}

async fn list_blocks(state: axum::extract::State<AppState>) -> Json<serde_json::Value> {
    let rows = sqlx::query!(
        "SELECT height, hash, timestamp, tx_count FROM blocks ORDER BY height DESC LIMIT 20"
    ).fetch_all(&state.pool).await.unwrap_or_default();
    Json(serde_json::json!({ "blocks": rows }))
}

async fn get_block(state: axum::extract::State<AppState>, Path(height): Path<i64>) -> Json<serde_json::Value> {
    let b = sqlx::query!("SELECT * FROM blocks WHERE height=$1", height)
        .fetch_one(&state.pool).await.ok();
    let txs = sqlx::query!(
        "SELECT txid, index_in_block, from_addr, to_addr, amount, fee FROM transactions WHERE block_height=$1 ORDER BY index_in_block",
        height
    ).fetch_all(&state.pool).await.unwrap_or_default();
    Json(serde_json::json!({ "block": b, "transactions": txs }))
}

async fn get_tx(state: axum::extract::State<AppState>, Path(txid): Path<String>) -> Json<serde_json::Value> {
    let t = sqlx::query!("SELECT * FROM transactions WHERE txid=$1", txid)
        .fetch_one(&state.pool).await.ok();
    Json(serde_json::json!({ "tx": t }))
}

async fn get_address(state: axum::extract::State<AppState>, Path(addr): Path<String>) -> Json<serde_json::Value> {
    let txs = sqlx::query!(
        "SELECT txid, block_height, from_addr, to_addr, amount, fee, timestamp 
         FROM transactions WHERE from_addr=$1 OR to_addr=$1 ORDER BY timestamp DESC LIMIT 50", addr)
        .fetch_all(&state.pool).await.unwrap_or_default();
    Json(serde_json::json!({ "address": addr, "txs": txs }))
}

async fn search_any(state: axum::extract::State<AppState>, Path(q): Path<String>) -> Json<serde_json::Value> {
    if let Ok(h) = q.parse::<i64>() {
        if let Ok(b) = sqlx::query!("SELECT height, hash FROM blocks WHERE height=$1", h).fetch_one(&state.pool).await {
            return Json(serde_json::json!({"type": "block", "data": b}));
        }
    }
    if let Ok(b) = sqlx::query!("SELECT height, hash FROM blocks WHERE hash=$1", q).fetch_one(&state.pool).await {
        return Json(serde_json::json!({"type": "block", "data": b}));
    }
    if let Ok(t) = sqlx::query!("SELECT txid FROM transactions WHERE txid=$1", q).fetch_one(&state.pool).await {
        return Json(serde_json::json!({"type": "tx", "data": t}));
    }
    Json(serde_json::json!({"type": "address", "data": { "address": q }}))
}

async fn supply(state: axum::extract::State<AppState>) -> Json<Supply> {
    Json(Supply { total: state.supply_total.clone(), circulating: state.supply_circ.clone() })
}
