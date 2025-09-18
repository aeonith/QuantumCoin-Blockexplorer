CREATE TABLE IF NOT EXISTS chain_meta (
  id SMALLINT PRIMARY KEY DEFAULT 1,
  tip_height BIGINT NOT NULL DEFAULT 0,
  tip_hash TEXT NOT NULL DEFAULT '',
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS blocks (
  height BIGINT PRIMARY KEY,
  hash TEXT UNIQUE NOT NULL,
  parent_hash TEXT NOT NULL,
  timestamp BIGINT NOT NULL,
  miner TEXT,
  tx_count INT NOT NULL,
  size_bytes INT NOT NULL,
  difficulty TEXT,
  nonce TEXT
);
CREATE INDEX IF NOT EXISTS blocks_ts_idx ON blocks (timestamp DESC);

CREATE TABLE IF NOT EXISTS transactions (
  txid TEXT PRIMARY KEY,
  block_height BIGINT NOT NULL REFERENCES blocks(height) ON DELETE CASCADE,
  index_in_block INT NOT NULL,
  timestamp BIGINT NOT NULL,
  from_addr TEXT,
  to_addr TEXT,
  amount TEXT NOT NULL,   -- store as string minimal units
  fee TEXT NOT NULL DEFAULT '0',
  raw JSONB
);
CREATE INDEX IF NOT EXISTS tx_block_idx ON transactions (block_height DESC);
CREATE INDEX IF NOT EXISTS tx_from_idx ON transactions (from_addr);
CREATE INDEX IF NOT EXISTS tx_to_idx ON transactions (to_addr);

CREATE TABLE IF NOT EXISTS address_summaries (
  address TEXT PRIMARY KEY,
  tx_count BIGINT NOT NULL DEFAULT 0,
  last_seen BIGINT NOT NULL DEFAULT 0
);
