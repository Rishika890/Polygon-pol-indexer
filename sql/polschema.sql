-- polschema.sql
CREATE TABLE IF NOT EXISTS transfers (
  tx_hash TEXT NOT NULL,
  log_index INTEGER NOT NULL,
  block_number INTEGER NOT NULL,
  timestamp TEXT NOT NULL,
  from_addr TEXT NOT NULL,
  to_addr TEXT NOT NULL,
  token_address TEXT NOT NULL,
  amount_raw TEXT NOT NULL,
  amount REAL NOT NULL,
  tx_fee_raw TEXT,
  receipt_status INTEGER,
  PRIMARY KEY (tx_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_transfers_to ON transfers(to_addr);
CREATE INDEX IF NOT EXISTS idx_transfers_from ON transfers(from_addr);
CREATE INDEX IF NOT EXISTS idx_transfers_block ON transfers(block_number);

CREATE TABLE IF NOT EXISTS net_flow (
  exchange TEXT NOT NULL,
  token_address TEXT NOT NULL,
  cumulative_amount_raw TEXT NOT NULL,
  cumulative_amount REAL NOT NULL,
  last_updated TEXT NOT NULL,
  PRIMARY KEY (exchange, token_address)
);

CREATE TABLE IF NOT EXISTS metadata (
  key TEXT PRIMARY KEY,
  value TEXT
);
