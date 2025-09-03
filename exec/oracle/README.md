# OpenStore Oracle

The Oracle service observes on-chain requests and compares well-known/assetlink.json data with data defined in smart contracts. It acts as a bridge between external data sources and the blockchain, ensuring data integrity and validation.

## Docker Setup

### Using Docker Compose (from project root)

```bash
# Start oracle service
docker compose up oracle

# Run in background
docker compose up -d oracle

# View logs
docker compose logs -f oracle

# Stop service
docker compose stop oracle
```

### Environment Variables for Docker

The Docker container automatically reads environment variables from your `.env` file. Make sure to set up the `.env` file in the project root with all required variables listed below.

## Environment Variables

The following environment variables are required:

### Blockchain Configuration
- `ETH_NODE_URL` - Ethereum node URL for blockchain connection
- `CHAIN_ID` - Blockchain chain ID (e.g., 1 for mainnet, 31337 for local)

### Wallet Configuration
- `WALLET_PK` - Private key for wallet operations

### Contract Addresses
- `ORACLE_ADDRESS` - Address of the Oracle smart contract

### Smart Contract Configuration (from sc/.env)
- `CONFIRM_COUNT` - Number of confirmations required for transactions (default: 0)
- `TX_POLL_TIMEOUT_MS` - Transaction polling timeout in milliseconds (default: 500)

### Telegram Notifications (Optional)
- `TG_TOKEN` - Telegram bot token for notifications
- `TG_INFO_CHAT_ID` - Chat ID for info notifications
- `TG_ALERT_CHAT_ID` - Chat ID for alert notifications

