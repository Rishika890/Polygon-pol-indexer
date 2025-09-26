use anyhow::Result;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    // 1) DB path (simple default). Explain: "either use env or default path"
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "./data/polygon.db".to_string());

    // 2) Make sure data directory exists (tiny safety)
    if let Some(dir) = Path::new(&db_path).parent() {
        fs::create_dir_all(dir)?;
    }

    // 3) Open DB
    let conn = Connection::open(&db_path)?;
    println!("Opened DB at {}", db_path);

    // 4) Create tables & indexes (your simple approach, wrapped in execute_batch)
    conn.execute_batch(
        "
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
        ",
    )?;
    println!("Schema created (or already present).");

    // 5) Optional: run compute SQL file (if you added compute_netflow.sql)
    if Path::new("sql/compute_netflow.sql").exists() {
        let compute_sql = fs::read_to_string("sql/compute_netflow.sql")?;
        conn.execute_batch(&compute_sql)?;
        println!("Executed compute_netflow.sql (computed & upserted netflow).");
    } else {
        println!("No compute SQL found; skipping compute step.");
    }

    // 6) Print net_flow rows (easy to explain)
    let mut stmt = conn.prepare("SELECT exchange, token_address, cumulative_amount, last_updated FROM net_flow")?;
    let mut rows = stmt.query([])?;
    println!("\n-- net_flow rows --");
    while let Some(row) = rows.next()? {
        let exchange: String = row.get(0)?;
        let token_address: String = row.get(1)?;
        let cumulative: f64 = row.get(2)?;
        let updated: String = row.get(3)?;
        println!("{} | {} | {} | {}", exchange, token_address, cumulative, updated);
    }

    println!("Done.");
    Ok(())
}
