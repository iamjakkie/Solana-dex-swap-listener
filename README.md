# Solana DeFi Trade Indexer

Solana DeFi Trade Indexer is a comprehensive, real-time and historical data indexing utility for multiple Solana DEXes including Raydium, Meteora, Orca. It decodes on-chain swap transactions, enriches trade data, and exports the results for use in backtesting, trading bots, dashboards, or machine learning pipelines.

## Overview
This repository is organized as a Cargo workspace with several interrelated modules:

  ### Indexer
  The core real-time listener that connects to a Solana RPC endpoint, fetches new blocks, decodes swap instructions, and produces uniform trade data.

  ### Preprocessor
  A supplementary service for historical data enrichment and gap-filling. It processes raw block data from disk (or other storage), augments it with additional on-chain or off-chain data, and prepares the data for backtesting.

  ### Common
  Shared libraries, models, and utilities used by both the Indexer and Preprocessor.


<b>The split between Indexer and Preprocessor allows the real-time component to focus on high-volume, low-latency ingestion while the preprocessor handles more resource-intensive historical enrichment. This division improves throughput and scalability.</b>

## Key Features
### Multi-Protocol Support:
Not just Raydium – the tool now supports several popular Solana DEXes such as Meteora, Orca, Base Uniswap, Eth Uniswap, and others.

### Real-Time Block Monitoring:
Continuously fetches new blocks from the Solana blockchain to capture and decode swap transactions.

### Flexible Decoding:
Extracts essential swap data (e.g., block date, block time, slot, token, price, USD price, volume) while handling variations in DEX instruction layouts.

### Historical & Gap Processing:
The Preprocessor ingests raw data from disk, fills in missing slots, enriches trade data with token metadata and pricing, and prepares the data for downstream analysis or backtesting.

### Modular & Extensible Architecture:
The workspace structure lets you run Indexer and Preprocessor as separate services (or together) and easily integrate additional DEX decoders or enrichment features.

## Getting Started
### Prerequisites:
  - Rust (latest stable version): https://www.rust-lang.org/tools/install
  - A Solana RPC endpoint (default is "https://api.mainnet-beta.solana.com", configurable via environment variables)
  - (Optional) AWS EC2 or another cloud provider to test latency improvements across different regions

### Cloning & Building:
1. Clone the repository:
   git clone https://github.com/iamjakkie/Solana-dex-swap-listener.git
   cd Solana-dex-swap-listener

2. Build the workspace:
   This repository is a Cargo workspace containing the "indexer", "preprocessor", and "common" modules. Build all targets in release mode:
   cargo build --release

3. Run the Indexer:
   To start real-time indexing, configure `indexer/main.rs` run:
   cd indexer
   cargo run --release \
   THIS FUNCTIONALITY WILL BE ADDED SOON

4. Run the Preprocessor:
   To process historical data or fill gaps, run:
   cd preprocessor
   cargo run --release

Configuration:
  - RPC Endpoint:
    Adjust the RPC endpoint via environment variables or in the configuration files.\
    `SOLANA_RPC_URL`
  - Output Paths:
    The tool writes enriched trade data (CSV, Avro, Parquet, etc.) to configured directories. Adjust these as needed.
    `OUTPUT_PATH`

## Usage Example
Below is a simplified example for the Indexer:

```Rust
  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let rpc_url = "https://api.mainnet-beta.solana.com";
      // Instantiate and run the trade listener
      let mut listener = TradeListener::new(rpc_url);
      listener.run().await?;
      Ok(())
  }
```

Under the hood, the tool:
  - Connects to a Solana RPC endpoint.
  - Continuously fetches and processes new blocks.
  - Decodes swap instructions from multiple DEX protocols.
  - Outputs enriched trade data for backtesting, trading bots, or analytics dashboards.

## Current Limitations
• Inner Instruction Decoding:
  Aggregated or inner instruction decoding (such as in Jupiter swaps) is not fully supported yet. This feature is under development. Most of the stuff works though.

• Performance Tuning:
  While real-time ingestion is robust, extremely high throughput may require additional concurrency, batching, or network optimizations.

## Roadmap
1. Enhanced Decoding:
   Expand support for inner instructions and additional DEX protocols.
2. Improved Export Options:
   Add direct database integrations and support for more file formats.
3. Performance Optimization:
   Further refine concurrency controls and network usage.

## Contributing
1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Submit a pull request with your changes.


Your contributions, performance tips, and new decoder logic are always welcome!

## License
Distributed under the MIT License. You can use, modify, and distribute this project for personal or commercial purposes as long as the original license is included.

## Questions or Feedback
  - GitHub Issues:\
    Open an issue on GitHub to report bugs, request features, or discuss improvements.
  - Direct Contact:\
    Reach out with ideas or inquiries—pull requests and forks are highly appreciated!

Happy indexing and backtesting! This tool is designed to simplify real-time Solana DEX data ingestion and processing so you can focus on building your strategies.