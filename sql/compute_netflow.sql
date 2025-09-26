-- compute_netflow.sql
-- OPTIONAL: sample data (comment out or remove if you will use real data)
INSERT OR IGNORE INTO transfers (tx_hash, log_index, block_number, timestamp, from_addr, to_addr, token_address, amount_raw, amount)
VALUES
('0xaaa0001', 0, 50000000, '2025-09-20T12:34:56Z', '0xabcdefabcdefabcdefabcdefabcdefabcdefabcd', '0xF977814e90dA44bFA03b6295A0616a897441aceC', 'POL_TOKEN_ADDRESS', '10500000000000000000', 10.5),
('0xaaa0002', 0, 50000001, '2025-09-20T12:50:10Z', '0xF977814e90dA44bFA03b6295A0616a897441aceC', '0x1111222233334444555566667777888899990000', 'POL_TOKEN_ADDRESS', '2000000000000000000', 2.0),
('0xaaa0003', 0, 50000002, '2025-09-20T13:12:30Z', '0x222233334444555566667777888899990000aaaa', '0x33334444555566667777888899990000bbbbaaaa', 'POL_TOKEN_ADDRESS', '750000000000000000', 0.75),
('0xaaa0004', 0, 50000003, '2025-09-20T13:45:00Z', '0x555566667777888899990000aaaa111122223333', '0xD5C08681719445A5Fdce2Bda98b341A49050d821', 'POL_TOKEN_ADDRESS', '100000000000000000000', 100.0),
('0xaaa0005', 0, 50000004, '2025-09-20T14:05:05Z', '0x082489A616aB4D46d1947eE3F912e080815b08DA', '0x66667777888899990000aaaabbbbccccddddeeee', 'POL_TOKEN_ADDRESS', '1000000000000000', 0.001),
('0xaaa0006', 0, 50000005, '2025-09-20T14:20:20Z', '0x7777888899990000aaaabbbbccccddddeeeeffff', '0x505e71695E9bc45943c58adEC1650577BcA68fD9', 'POL_TOKEN_ADDRESS', '50000000000000000000', 50.0);

-- compute and upsert netflow for Binance (human-readable)
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
VALUES (
  'Binance',
  'POL_TOKEN_ADDRESS',
  '0',
  (SELECT net_flow FROM net),
  datetime('now')
)
ON CONFLICT(exchange, token_address) DO UPDATE SET
  cumulative_amount = excluded.cumulative_amount,
  last_updated = excluded.last_updated;
