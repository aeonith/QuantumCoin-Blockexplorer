use anyhow::*;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::time::{sleep, Duration};
use tracing::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct RpcTx {
    txid: String,
    #[serde(default)]
    from_addr: Option<String>,
    #[serde(default)]
    to_addr: Option<String>,
    #[serde(default)]
    amount: String,    // minimal units as string
    #[serde(default)]
    fee: String,       // minimal units as string
    #[serde(default)]
    timestamp: u64,
    #[serde(default)]
    index_in_block: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct RpcBlock {
    height: u64,
    hash: String,
    parent_hash: String,
    timestamp: u64,
    #[serde(default)]
    txs: Vec<RpcTx>,
    #[serde(default)]
    size_bytes: u32,
    #[serde(default)]
    difficulty: String,
    #[serde(default)]
    nonce: String,
}

#[derive(Clone)]
struct Cfg {
    rpc_url: String,
    pg_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_target(false).init();
    let cfg = Cfg {
        rpc_url: std::env::var("QTC_RPC").context("QTC_RPC env not set")?,
        pg_url: std::env::var("DATABASE_URL").context("DATABASE_URL env not set")?,
    };
    let pool = PgPoolOptions::new().max_connections(8).connect(&cfg.pg_url).await?;

    // init meta row
    sqlx::query("INSERT INTO chain_meta (id, tip_height, tip_hash) VALUES (1, 0, '') ON CONFLICT (id) DO NOTHING")
        .execute(&pool).await?;

    info!("qtc-indexer starting; rpc={}", cfg.rpc_url);

    loop {
        if let Err(e) = sync_loop(&cfg, &pool).await {
            error!("sync error: {e:?}");
            sleep(Duration::from_secs(5)).await;
        }
        sleep(Duration::from_millis(800)).await;
    }
}

async fn sync_loop(cfg: &Cfg, pool: &PgPool) -> Result<()> {
    let (local_tip,): (i64,) = sqlx::query_as("SELECT tip_height FROM chain_meta WHERE id=1")
        .fetch_one(pool).await?;
    let remote_tip = get_latest_height(&cfg.rpc_url).await?;
    if remote_tip <= local_tip as u64 {
        return Ok(());
    }
    for h in (local_tip as u64 + 1)..=remote_tip {
        let b = get_block_by_height(&cfg.rpc_url, h).await?;
        ingest_block(pool, &b).await?;
        sqlx::query("UPDATE chain_meta SET tip_height=$1, tip_hash=$2, updated_at=NOW() WHERE id=1")
            .bind(b.height as i64)
            .bind(&b.hash)
            .execute(pool).await?;
        info!("ingested block {}", b.height);
    }
    Ok(())
}

/* ========= RPC ADAPTERS =========
Expected endpoints (adjust if your backend differs):
GET {RPC}/latest_height -> { "height": <u64> }
GET {RPC}/block/{height} -> RpcBlock (as modeled above)

If your shapes differ, map them here before returning RpcBlock.
*/

async fn get_latest_height(rpc: &str) -> Result<u64> {
    let url = format!("{rpc}/latest_height");
    let v: serde_json::Value = reqwest::Client::new().get(&url).send().await?.json().await?;
    Ok(v.get("height").and_then(|x| x.as_u64()).unwrap_or(0))
}

async fn get_block_by_height(rpc: &str, h: u64) -> Result<RpcBlock> {
    let url = format!("{rpc}/block/{h}");
    let mut b: RpcBlock = reqwest::Client::new().get(&url).send().await?.json().await?;

    // Defensive defaults in case your backend omits fields:
    if b.txs.is_empty() {
        // If your backend returns "transactions", remap like:
        // let v: serde_json::Value = reqwest::Client::new().get(&url).send().await?.json().await?;
        // b.txs = v["transactions"].as_array().unwrap_or(&vec![]).iter().map(|t| ... ).collect();
    }
    Ok(b)
}

async fn ingest_block(pool: &PgPool, b: &RpcBlock) -> Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query(r#"
        INSERT INTO blocks (height, hash, parent_hash, timestamp, tx_count, size_bytes, difficulty, nonce)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        ON CONFLICT (height) DO NOTHING
    "#)
    .bind(b.height as i64)
    .bind(&b.hash)
    .bind(&b.parent_hash)
    .bind(b.timestamp as i64)
    .bind(b.txs.len() as i32)
    .bind(b.size_bytes as i32)
    .bind(&b.difficulty)
    .bind(&b.nonce)
    .execute(&mut *tx).await?;

    for t in &b.txs {
        sqlx::query(r#"
            INSERT INTO transactions (txid, block_height, index_in_block, timestamp, from_addr, to_addr, amount, fee, raw)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            ON CONFLICT (txid) DO NOTHING
        "#)
        .bind(&t.txid)
        .bind(b.height as i64)
        .bind(t.index_in_block as i32)
        .bind(t.timestamp as i64)
        .bind(&t.from_addr)
        .bind(&t.to_addr)
        .bind(&t.amount)
        .bind(&t.fee)
        .bind(serde_json::to_value(t)?)
        .execute(&mut *tx).await?;
    }

    // lightweight address stats
    for t in &b.txs {
        if let Some(a) = &t.from_addr {
            sqlx::query(r#"
                INSERT INTO address_summaries(address, tx_count, last_seen)
                VALUES ($1,1,$2)
                ON CONFLICT (address) DO UPDATE SET 
                  tx_count = address_summaries.tx_count + 1,
                  last_seen = GREATEST(address_summaries.last_seen, EXCLUDED.last_seen)
            "#).bind(a).bind(t.timestamp as i64).execute(&mut *tx).await?;
        }
        if let Some(a) = &t.to_addr {
            sqlx::query(r#"
                INSERT INTO address_summaries(address, tx_count, last_seen)
                VALUES ($1,1,$2)
                ON CONFLICT (address) DO UPDATE SET 
                  tx_count = address_summaries.tx_count + 1,
                  last_seen = GREATEST(address_summaries.last_seen, EXCLUDED.last_seen)
            "#).bind(a).bind(t.timestamp as i64).execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;
    Ok(())
}
