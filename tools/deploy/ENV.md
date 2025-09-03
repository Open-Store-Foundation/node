# OpenStore Environment Generator

A Python CLI tool for generating `.env` configuration files for OpenStore deployment services.

## Features

- Interactive configuration collection
- Deployment profiles (BSC Testnet and Localhost)
- Service-specific environment templates
- Wallet private key mapping (admin vs user)
- PostgreSQL and Redis URL construction

## Usage

### Interactive Mode (Recommended)

```bash
cd tools
python3 env_generator.py
```

This will:
1. Prompt to select deployment profile (bsctest/localhost)
2. Prompt for all required configuration values
3. Generate `.env` files for all services
4. Place them in `deploy/config/{service}/.env`

### Use Specific Profile

```bash
# BSC Testnet profile
python3 env_generator.py --profile bsctest

# Localhost development profile  
python3 env_generator.py --profile localhost
```

### List Available Services

```bash
python3 env_generator.py --list-services
```

### List Available Profiles

```bash
python3 env_generator.py --list-profiles
```

### Generate for Specific Service

```bash
python3 env_generator.py --service oracle --profile bsctest
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
- **REDIS_PASS**: Redis password (optional)

### Blockchain
- **ETH_NODE_URL**: Ethereum node URL (leave empty for BSC testnet)
- **CHAIN_ID**: Blockchain chain ID
- **ORACLE_ADDRESS**: Oracle contract address
- **STORE_ADDRESS**: Store contract address
- **HISTORICAL_SYNC_BLOCK**: Starting block for historical sync

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

## File Structure

After running, your configuration will be organized as:

```
deploy/config/
├── oracle/.env
├── validator/.env
├── daemon-client/.env
├── api-client/.env
└── postgres/.env
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
**Daemon Client**: Telegram, Blockchain, Wallet, Contracts, Sync, Database (PostgreSQL), Services  
**API Client**: Telegram, Database (PostgreSQL), Services  
**PostgreSQL Database**: Postgres block only

## Requirements

- Python 3.6+
- No external dependencies (uses standard library only)

## Examples

### Quick Start with BSC Testnet
```bash
python3 env_generator.py --profile bsctest
# Fill in your private keys and other required values
```

### Local Development
```bash
python3 env_generator.py --profile localhost
# Configure for local blockchain development
```

### Interactive Mode
```bash
python3 env_generator.py
# Choose profile interactively
# Customize all settings as needed
```
