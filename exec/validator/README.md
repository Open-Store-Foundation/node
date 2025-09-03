# OpenStore Validator

The Validator service is responsible for validating application artifacts, such as APK files. It performs comprehensive verification of mobile applications to ensure they meet security and quality standards before being approved for distribution.

## Docker Setup

### Using Docker Compose (from project root)

```bash
# Start validator service
docker compose up validator

# Run in background
docker compose up -d validator

# View logs
docker compose logs -f validator

# Stop service
docker compose stop validator
```

### Environment Variables for Docker

The Docker container automatically reads environment variables from your `.env` file. Make sure to set up the `.env` file in the project root with all required variables listed below.

### Docker Volume

The validator service uses a Docker volume `validator-tmp` for temporary file storage. This volume persists between container restarts and is automatically managed by Docker Compose.

## Environment Variables

The following environment variables are required:

### Database Configuration
- `DATABASE_URL` - SQLite database path for validator data storage

### Blockchain Configuration
- `ETH_NODE_URL` - Ethereum node URL for blockchain connection
- `GF_NODE_URL` - Greenfield node URL for additional blockchain data
- `CHAIN_ID` - Blockchain chain ID (e.g., 1 for mainnet, 31337 for local)

### Wallet Configuration
- `WALLET_PK` - Private key for wallet operations

### Contract Addresses
- `STORE_ADDRESS` - Address of the OpenStore smart contract

### File Storage
- `FILE_STORAGE_PATH` - Path for temporary file storage (default: exec/validator/tmp)

### Synchronization Settings
- `HISTORICAL_SYNC_THRESHOLD` - Block threshold for historical sync (default: 500)

### Smart Contract Configuration (from sc/.env)
- `CONFIRM_COUNT` - Number of confirmations required for transactions (default: 0)
- `TX_POLL_TIMEOUT_MS` - Transaction polling timeout in milliseconds (default: 500)

### Telegram Notifications (Optional)
- `TG_TOKEN` - Telegram bot token for notifications
- `TG_INFO_CHAT_ID` - Chat ID for info notifications
- `TG_ALERT_CHAT_ID` - Chat ID for alert notifications

