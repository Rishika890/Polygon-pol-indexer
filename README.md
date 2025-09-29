# Real-time Polygon Blockchain Data Indexer

## Overview
This project showcases a real-time blockchain data indexing system for the Polygon network, focusing on tracking POL token transfers to and from Binance addresses. The primary goal is to calculate cumulative net-flows (inflows minus outflows) and store the data in an SQLite database. The system processes new blocks as they arrive, filters relevant transactions, and updates net-flow.

## Technology Stack
- Blockchain: Polygon Network
- Token: POL
- Database: SQLite
- Programming Language: Rust
- Data Sources: 
  - Polygon RPC Access (via Alchemy)
  - Binance addresses pre provided

## Installation and instruction to run the project 
1. Clone the Repository:
   git clone <https://github.com/Rishika890/Polygon-pol-indexer>
   cd Polygon-pol-indexer

2. Install Dependencies:

Ensure Rust is installed
Run cargo build to fetch dependencies listed in Cargo.toml as per tasks requirement.

3.Set Environment Variable:

In PowerShell: $env:POLYGON_RPC="https://polygon-mainnet.g.alchemy.com/v2/WDjtT7mQZnV0io5bPbuHi"

4.Run application 
cargo run --release

## Schema Design (sql folder)
# Tables

1. transfers:

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
Purpose: Stores raw transaction data for POL transfers involving Binance addresses.

2.net_flow:

exchange TEXT NOT NULL,
  token_address TEXT NOT NULL,
  cumulative_amount_raw TEXT NOT NULL,
  cumulative_amount REAL NOT NULL,
  last_updated TEXT NOT NULL,
  PRIMARY KEY (exchange, token_address)
Purpose: Tracks the cumulative net-flow of POL to/from Binance over time.
3. metadata:

key TEXT PRIMARY KEY,
  value TEXT
Purpose: Reserved for future use (e.g., configuration or stats).

## Functionality

Data Fetching: Connects to the Polygon network via RPC to retrieve the latest block number and its transactions using eth_blockNumber and eth_getBlockByNumber.
Filtering: Identifies transactions involving the following Binance addresses :

0xF977814e90dA44bFA03b6295A0616a897441aceC
0xe7804c37c13166fF0b37F5aE0BB07A3aEbb6e245
0x505e71695E9bc45943c58adEC1650577BcA68fD9
0x290275e3db66394C52272398959845170E4DCb88
0xD5C08681719445A5Fdce2Bda98b341A49050d821
0x082489A616aB4D46d1947eE3F912e080815b08DA


Data Storage: Inserts filtered transaction details into the transfers table.
Net-Flow Calculation: Updates the net_flow table every 10 seconds to compute the cumulative net-flow.

## Data  Flow

Raw Data: Fetched from Polygon RPC as JSON responses containing block and transaction data.
Processing:

Filters transactions where from or to matches Binance addresses.
Extracts tx_hash, from_addr, to_addr, and placeholders for block_number, timestamp, amount_raw, and amount.


Storage: Inserts processed data into transfers.
Transformation: A SQL query aggregates amount values from transfers to calculate net-flow, stored in net_flow with a last_updated timestamp.
Output: The net_flow table reflects the latest calculated  net-flow for Binance, updated in real-time.

## Code Structure(src folder)

1. Dependencies:

anyhow = "1.0"                                          # simple error handling
dotenvy = "0.15"                                        # load .env variables
rusqlite = { version = "0.31", features = ["bundled"] }  # for sqlite handling async
reqwest = { version = "0.11", features = ["json"] }      # for http client
tokio = { version = "1", features = ["full"] }           # for async runtime
serde_json = "1.0"                                       # for json file handling
chrono = "0.4"                                          # for timestamp 

2. Database Setup (Lines 14-54):

Opens data/polygon.db and creates transfers, net_flow, and metadata tables with indexes.
Inserts sample data to initialize the database.


Initial Net-Flow (Lines 56-86):

Computes the initial net-flow using a CTE aggregating sample data inflows and outflows for Binance addresses.


3. Real-Time Indexing (Lines 88-218):

Fetches the latest block number via eth_blockNumber.
Retrieves block transactions with eth_getBlockByNumber.
Filters transactions for Binance addresses and stores them in transfers.
Updates net_flow every 10 seconds with a loop using another CTE.

4.Async Handling:

Uses [tokio::main] for asynchronous RPC calls with timeouts (10-20 seconds).

# Expected Output:

Opened DB at data/polygon.db
Schema created (or already present).
Sample data inserted.
Net-flow computed for Binance.
Binance-related txs in block 0x...: []
Inserted 0 Binance-related txs into transfers
Updated net_flow at 2025-09-28 08:59:19.948305 UTC
Updated net_flow at 2025-09-28 08:59:29.984434 UTC

Submission

Repository: <https://github.com/Rishika890/Polygon-pol-indexer>
Submitted on: September 29, 2025
Author:    Rishika Vaidya
 

