# OpenStore Environment Generator

A Python CLI tool for generating `.env` configuration files for OpenStore deployment services.

## Features

- Interactive configuration collection
- Deployment profiles (BSC Testnet and Localhost)
- Service-specific environment templates
- Wallet private key mapping (admin vs user)
- PostgreSQL and Redis URL construction
- Nginx configuration generation (HTTP/HTTPS/none variants)
- Redis configuration file generation
- SSL certificate management integration

## Usage

### Interactive Mode (Recommended)

```bash
cd tools/deploy
python3 envgen.py --config-dir /path/to/config
```

This will:
1. Prompt to select deployment profile (bsctest/localhost)
2. Prompt for all required configuration values
3. Generate `.env` files for all services
4. Generate nginx configuration files (if nginx service selected)
5. Generate redis configuration file
6. Place them in the specified config directory structure

### Use Specific Profile

```bash
# BSC Testnet profile
python3 envgen.py --profile bsctest --config-dir /path/to/config

# Localhost development profile  
python3 envgen.py --profile localhost --config-dir /path/to/config
```

### List Available Services

```bash
python3 envgen.py --list-services
```

### List Available Profiles

```bash
python3 envgen.py --list-profiles
```

### Generate for Specific Service

```bash
python3 envgen.py --service oracle --profile bsctest --config-dir /path/to/config
python3 envgen.py --service nginx --config-dir /path/to/config  # Special nginx configuration
```

### Using Environment Variables

```bash
# Set CONFIG_DIR environment variable to skip --config-dir argument
export CONFIG_DIR=/path/to/config
python3 envgen.py --profile bsctest
```

## Configuration Inputs

### Private Keys
- **Admin Wallet Private Key**: Used for oracle and validator services
- **User Wallet Private Key**: Used for client services

### Telegram
- **TG_TOKEN**: Telegram bot token
- **TG_INFO_CHAT_ID**: Chat ID for info messages
- **TG_ALERT_CHAT_ID**: Chat ID for alert messages

### Database
- **POSTGRES_DB**: PostgreSQL database name
- **POSTGRES_USER**: PostgreSQL username
- **POSTGRES_PASSWORD**: PostgreSQL password

### Redis
- **REDIS_HOST**: Redis host (default: redis)
- **REDIS_USER**: Redis username (optional)
- **REDIS_PASS**: Redis password (optional)

### Blockchain
- **ETH_NODE_URL**: Ethereum node URL
- **ETHSCAN_API_KEY**: Etherscan API key
- **CLIENT_HOST_URL**: Client host URL (default: 127.0.0.1:8080)
- **CHAIN_ID**: Blockchain chain ID
- **ORACLE_ADDRESS**: Oracle contract address
- **STORE_ADDRESS**: Store contract address
- **HISTORICAL_SYNC_BLOCK**: Starting block for historical sync

### Nginx Configuration
- **DOMAIN_NAME**: Your domain name (e.g., example.com)
- **NGINX_VARIANT**: Configuration type (http/https/none)
- **CERTBOT_EMAIL**: Email for SSL certificates (Let's Encrypt)

### File Storage
- **FILE_STORAGE_PATH**: Validator file storage path (default: ./tmp/)

## Deployment Profiles

### BSC Testnet Profile (`bsctest`)
```
CHAIN_ID=97
GF_NODE_URL=https://gnfd-testnet-fullnode-tendermint-ap.bnbchain.org
ORACLE_ADDRESS=0F61D8D6c9D6886ac7cba72716E1b98C4379E0f7
STORE_ADDRESS=6Edac88EA58168a47ab61836bCbAD0Ac844498A6
HISTORICAL_SYNC_BLOCK=60727665
HISTORICAL_SYNC_THRESHOLD=500
CONFIRM_COUNT=1
TX_POLL_TIMEOUT_MS=5000
FILE_STORAGE_PATH=exec/validator/tmp
RUST_LOG=info
```

User provides:
- ETH_NODE_URL (Alchemy/Infura endpoint)
- ETHSCAN_API_KEY
- CLIENT_HOST_URL
- Database credentials (PostgreSQL + SQLite names)
- Private keys
- Telegram tokens

### Localhost Profile (`localhost`)
```
CHAIN_ID=31337
GF_NODE_URL=http://127.0.0.1:26657
ORACLE_ADDRESS=0x0000000000000000000000000000000000000000
STORE_ADDRESS=0x0000000000000000000000000000000000000000
HISTORICAL_SYNC_BLOCK=0
HISTORICAL_SYNC_THRESHOLD=500
CONFIRM_COUNT=1
TX_POLL_TIMEOUT_MS=500
FILE_STORAGE_PATH=exec/validator/tmp
RUST_LOG=info
```

User provides:
- ETH_NODE_URL (local node endpoint)
- ETHSCAN_API_KEY (optional for localhost)
- CLIENT_HOST_URL
- Database credentials (PostgreSQL + SQLite names)
- Private keys
- Telegram tokens

## Generated Services

The tool generates `.env` files for:

- **oracle**: Oracle service configuration
- **validator**: Validator service configuration  
- **daemon-client**: Client daemon configuration
- **api-client**: API client configuration
- **postgres**: PostgreSQL database configuration
- **nginx**: Nginx reverse proxy configuration

Additional configuration files generated:
- **redis.conf**: Redis server configuration
- **nginx.conf**: Main nginx configuration
- **default.conf.template**: Nginx site template (HTTP/HTTPS variants)

## File Structure

After running, your configuration will be organized as:

```
config/
├── oracle/.env
├── validator/.env
├── daemon-client/.env
├── api-client/.env
├── postgres/.env
├── nginx/
│   ├── .env
│   ├── nginx.conf
│   └── default.conf.template
└── redis/
    └── redis.conf
```

## Environment Variables Reference

Generated `.env` files are organized into logical blocks for better readability:

### Logical Blocks Structure
```
# SERVICE Environment Configuration

# TELEGRAM
TG_TOKEN=...
TG_INFO_CHAT_ID=...
TG_ALERT_CHAT_ID=...

# BLOCKCHAIN  
ETH_NODE_URL=...
CHAIN_ID=...
GF_NODE_URL=...
ETHSCAN_API_KEY=...

# WALLET
WALLET_PK=...

# CONTRACTS
ORACLE_ADDRESS=...
STORE_ADDRESS=...

# SYNC
HISTORICAL_SYNC_THRESHOLD=...
HISTORICAL_SYNC_BLOCK=...
CONFIRM_COUNT=...
TX_POLL_TIMEOUT_MS=...

# DATABASE
DATABASE_URL=...

# SERVICES
REDIS_URL=...
CLIENT_HOST_URL=...
FILE_STORAGE_PATH=...

# POSTGRES (postgres service only)
POSTGRES_DB=...
POSTGRES_USER=...
POSTGRES_PASSWORD=...
```

### Service-Specific Variables

**Oracle Service**: Telegram, Blockchain, Wallet, Contracts, Sync  
**Validator Service**: Telegram, Blockchain, Wallet, Contracts, Sync, Database (SQLite), Services  
**Daemon Client**: Telegram, Blockchain, Wallet, Contracts, Sync, Database (PostgreSQL)  
**API Client**: Telegram, Wallet, Database (PostgreSQL), Services  
**PostgreSQL Database**: Postgres block only  
**Nginx Service**: Nginx block only (DOMAIN_NAME, CERTBOT_EMAIL)

## Requirements

- Python 3.6+
- No external dependencies (uses standard library only)

## Examples

### Quick Start with BSC Testnet
```bash
python3 envgen.py --profile bsctest --config-dir ./config
```

### Local Development
```bash
python3 envgen.py --profile localhost --config-dir ./config
```

### Interactive Mode
```bash
python3 envgen.py --config-dir ./config
```

### SSL Setup with Nginx
```bash
# 1. Generate HTTP configuration first
python3 envgen.py --service nginx --config-dir ./config
# Select "http" variant, provide domain and email

# 2. After obtaining SSL certificate, switch to HTTPS
python3 envgen.py --service nginx --config-dir ./config
# Select "https" variant
```
