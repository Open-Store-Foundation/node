# OpenStore Client

OpenStore Client consists of two main components:

## client-daemon
Synchronizes data from the blockchain and stores it in the database. The daemon continuously monitors blockchain events and updates the local database with the latest information.

## client-api  
Retrieves synced data from the database and provides it to users through REST API endpoints. The API serves as the interface for applications to access the synchronized blockchain data.

## Docker Setup

### Using Docker Compose (from project root)

```bash
# Start both client services
docker compose up api-client daemon-client

# Start only API service
docker compose up api-client

# Start only daemon service
docker compose up daemon-client

# View logs
docker compose logs -f api-client
docker compose logs -f daemon-client
```

### Environment Variables for Docker

The Docker containers automatically read environment variables from your `.env` file. Make sure to set up the `.env` file in the project root with all required variables listed below.

## Environment Variables

The following environment variables are required:

### Blockchain Configuration
- `ETH_NODE_URL` - Ethereum node URL for blockchain connection
- `GF_NODE_URL` - Greenfield node URL for additional blockchain data
- `ETHSCAN_API_KEY` - Etherscan API key for blockchain data verification
- `CHAIN_ID` - Blockchain chain ID (e.g., 1 for mainnet, 31337 for local)

### Wallet Configuration  
- `WALLET_PK` - Private key for wallet operations

### Contract Addresses
- `ORACLE_ADDRESS` - Address of the Oracle smart contract
- `STORE_ADDRESS` - Address of the OpenStore smart contract

### Synchronization Settings
- `HISTORICAL_SYNC_THRESHOLD` - Block threshold for historical sync (default: 500)
- `HISTORICAL_SYNC_BLOCK` - Starting block for historical synchronization (default: 0)

### API Configuration
- `CLIENT_HOST_URL` - Host URL for the client API (default: 127.0.0.1:8081)

### Database Configuration
- `DATABASE_URL` - PostgreSQL database connection URL
- `REDIS_URL` - Redis connection URL for caching

### Telegram Notifications (Optional)
- `TG_TOKEN` - Telegram bot token for notifications
- `TG_INFO_CHAT_ID` - Chat ID for info notifications
- `TG_ALERT_CHAT_ID` - Chat ID for alert notifications

