use anyhow::Result; // error handling
use rusqlite::Connection; // Connects to SQLite database
use reqwest::Client; // Makes HTTP requests to Alchemy
use serde_json::Value; // Parses JSON 
use std::env; // Gets the Alchemy URL from environment
use std::time::Duration; // Sets a timeout for the request

#[tokio::main] // Makes main async to handle network waits
async fn main() -> Result<()> {
    // 1. Open the database
    let conn = Connection::open("data/polygon.db")?;
    println!("Opened DB at data/polygon.db");

    // 2. Create tables
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

    // 3. Insert sample data
    conn.execute_batch(
        "
        INSERT OR IGNORE INTO transfers (tx_hash, log_index, block_number, timestamp, from_addr, to_addr, token_address, amount_raw, amount)
        VALUES
        ('0xaaa0001', 0, 50000000, '2025-09-20T12:34:56Z', '0xabcdefabcdefabcdefabcdefabcdefabcdefabcd', '0xF977814e90dA44bFA03b6295A0616a897441aceC', 'POL_TOKEN_ADDRESS', '10500000000000000000', 10.5),
        ('0xaaa0002', 0, 50000001, '2025-09-20T12:50:10Z', '0xF977814e90dA44bFA03b6295A0616a897441aceC', '0x1111222233334444555566667777888899990000', 'POL_TOKEN_ADDRESS', '2000000000000000000', 2.0),
        ('0xaaa0003', 0, 50000002, '2025-09-20T13:12:30Z', '0x222233334444555566667777888899990000aaaa', '0x33334444555566667777888899990000bbbbaaaa', 'POL_TOKEN_ADDRESS', '750000000000000000', 0.75),
        ('0xaaa0004', 0, 50000003, '2025-09-20T13:45:00Z', '0x555566667777888899990000aaaa111122223333', '0xD5C08681719445A5Fdce2Bda98b341A49050d821', 'POL_TOKEN_ADDRESS', '100000000000000000000', 100.0),
        ('0xaaa0005', 0, 50000004, '2025-09-20T14:05:05Z', '0x082489A616aB4D46d1947eE3F912e080815b08DA', '0x66667777888899990000aaaabbbbccccddddeeee', 'POL_TOKEN_ADDRESS', '1000000000000000', 0.001),
        ('0xaaa0006', 0, 50000005, '2025-09-20T14:20:20Z', '0x7777888899990000aaaabbbbccccddddeeeeffff', '0x505e71695E9bc45943c58adEC1650577BcA68fD9', 'POL_TOKEN_ADDRESS', '50000000000000000000', 50.0);
        ",
    )?;
    println!("Sample data inserted.");

    // 4. Compute net-flow
    conn.execute_batch(
        "
        WITH
        inflow AS (
            SELECT COALESCE(SUM(amount), 0.0) AS total_in
            FROM transfers
            WHERE to_addr IN (
                '0xF977814e90dA44bFA03b6295A0616a897441aceC',
                '0xe7804c37c13166fF0b37F5aE0BB07A3aEbb6e245',
                '0x505e71695E9bc45943c58adEC1650577BcA68fD9',
                '0x290275e3db66394C52272398959845170E4DCb88',
                '0xD5C08681719445A5Fdce2Bda98b341A49050d821',
                '0x082489A616aB4D46d1947eE3F912e080815b08DA'
            )
        ),
        outflow AS (
            SELECT COALESCE(SUM(amount), 0.0) AS total_out
            FROM transfers
            WHERE from_addr IN (
                '0xF977814e90dA44bFA03b6295A0616a897441aceC',
                '0xe7804c37c13166fF0b37F5aE0BB07A3aEbb6e245',
                '0x505e71695E9bc45943c58adEC1650577BcA68fD9',
                '0x290275e3db66394C52272398959845170E4DCb88',
                '0xD5C08681719445A5Fdce2Bda98b341A49050d821',
                '0x082489A616aB4D46d1947eE3F912e080815b08DA'
            )
        ),
        net AS (
            SELECT (inflow.total_in - outflow.total_out) AS net_flow
            FROM inflow, outflow
        )
        INSERT INTO net_flow (exchange, token_address, cumulative_amount_raw, cumulative_amount, last_updated)
        VALUES ('Binance', 'POL_TOKEN_ADDRESS', '0', (SELECT net_flow FROM net), datetime('now'))
        ON CONFLICT(exchange, token_address) DO UPDATE SET
            cumulative_amount = excluded.cumulative_amount,
            last_updated = excluded.last_updated;
        ",
    )?;
    println!("Net-flow computed for Binance.");

    // 5. RPC to fetch latest block (new part)
    let rpc_url = env::var("POLYGON_RPC").expect("POLYGON_RPC must be set"); // Get Alchemy URL
    let client = Client::new(); // Sets up a tool to send requests

    let res: Value = client
        .post(&rpc_url) // Sends request to Alchemy
        .json(&serde_json::json!({ // Data to send
            "jsonrpc": "2.0", // RPC protocol version
            "method": "eth_blockNumber", // Asks for the latest block
            "params": [], 
            "id": 1 // Unique request ID
        }))
        .timeout(Duration::from_secs(10)) // Limits wait to 10 seconds
        .send()
        .await? // Waits for the response
        .json()
        .await?; // Turns response into JSON
    let block_hex = res["result"].as_str().unwrap_or("0x0"); // Gets the block number (in hex)
    let block_num = u64::from_str_radix(&block_hex[2..], 16)?; // Converts hex to number
    println!("Latest block number: {}", block_num); // Shows the result

    Ok(())
}