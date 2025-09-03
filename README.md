# OpenStore Node

OpenStore is a decentralized application store built on blockchain technology. This repository contains the core node infrastructure that powers the OpenStore ecosystem.

## Architecture

The OpenStore node consists of several specialized components working together to provide a complete decentralized app store experience:

- **Client System**: Synchronizes blockchain data and provides API access
- **Oracle Service**: Validates external data sources against on-chain contracts
- **Validator Service**: Verifies application artifacts for security and compliance
- **Statistics Service**: Collects and processes usage analytics

## Available Binaries

The project provides 4 main executable binaries:

### 1. `api-client`
**Location**: `exec/client/`
**Binary**: `api-client`

REST API server that provides access to synchronized blockchain data. Serves endpoints for applications, categories, search, reviews, and reports.

**Usage**:
```bash
cargo run --bin api-client
```

### 2. `daemon-client` 
**Location**: `exec/client/`
**Binary**: `daemon-client`

Background daemon that continuously synchronizes data from the blockchain and stores it in the database. Monitors blockchain events and updates the local database with the latest information.

**Usage**:
```bash
cargo run --bin daemon-client
```

### 3. `oracle`
**Location**: `exec/oracle/`
**Binary**: `oracle`

Oracle service that observes on-chain requests and compares well-known/assetlink.json data with data defined in smart contracts. Acts as a bridge between external data sources and the blockchain.

**Usage**:
```bash
cargo run --bin oracle
```

### 4. `validator`
**Location**: `exec/validator/`
**Binary**: `validator`

Validator service responsible for validating application artifacts such as APK files. Performs comprehensive verification of mobile applications to ensure they meet security and quality standards.

**Usage**:
```bash
cargo run --bin validator
```

## Deployment

OpenStore provides comprehensive deployment tools and configurations in the `deploy/` folder for easy setup and management of all services.

### Quick Deploy with Docker Compose

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd node/deploy
   ```

2. **Generate environment configurations**:
   ```bash
   # Interactive configuration setup
   python3 envgen.py
   
   # Or use a specific profile (bsctest/localhost)
   python3 envgen.py --profile bsctest
   ```

3. **Sync service binaries**:
   ```bash
   # Download from GitHub releases
   python3 sync.py
   # Enter release version (e.g., v1.0.0) or "local" for local build
   ```

4. **Start all services**:
   ```bash
   # Build and start all services
   docker compose up --build -d
   
   # View logs
   docker compose logs -f
   
   # Stop services
   docker compose down
   ```

### Deployment Services

The Docker Compose setup includes:

- **api-client**: REST API server (port 8080)
- **daemon-client**: Blockchain synchronization daemon
- **oracle**: External data validation service
- **validator**: Application artifact validation service
- **postgres**: PostgreSQL database (port 5432)
- **redis**: Redis cache (port 6379)
- **admin**: Management container with build tools

### Environment Configuration

Use `envgen.py` to generate service-specific environment files:

```bash
# List available profiles
python3 envgen.py --list-profiles

# List available services
python3 envgen.py --list-services

# Generate for specific service
python3 envgen.py --service oracle --profile bsctest
```

**Supported Profiles:**
- **bsctest**: BSC Testnet deployment with predefined contracts
- **localhost**: Local development environment

**Generated Configuration Structure:**
```
deploy/.config/
├── api-client/.env
├── daemon-client/.env
├── oracle/.env
├── validator/.env
└── postgres/.env
```

### Binary Management

Use `sync.py` to manage service binaries:

```bash
# Download from GitHub releases
python3 sync.py
# Enter version: v1.0.0

# Use local build
python3 sync.py
# Enter version: local
```

Binaries are organized in `deploy/.shared/`:
```
deploy/.shared/
├── api-client/
├── daemon-client/
├── oracle/
├── validator/
├── postgres/
├── redis/
└── sqlite/
```

### Production Deployment Process

Follow the deployment workflow in `deploy/DEPLOY.md`:

1. Create and tag release
2. Build release binaries via GitHub Actions
3. Connect to production server
4. Sync repository and generate environment configs
5. Apply database migrations
6. Download and deploy service binaries
7. Restart all services

### Quick Start

### Native Setup

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd node
   ```

2. **Set up environment variables**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Build all binaries**:
   ```bash
   cargo build --release
   ```

4. **Run individual services**:
   ```bash
   # Start the daemon to sync blockchain data
   cargo run --bin daemon-client
   
   # Start the API server (in another terminal)
   cargo run --bin api-client
   
   # Start oracle service (in another terminal)
   cargo run --bin oracle
   
   # Start validator service (in another terminal)
   cargo run --bin validator
   ```

## Project Structure

```
├── deploy/         # Deployment configurations and tools
│   ├── .config/    # Generated environment configurations
│   ├── .shared/    # Service binaries and shared data
│   ├── envgen.py   # Environment configuration generator
│   ├── sync.py     # Binary synchronization tool
│   ├── docker-compose.yml  # Multi-service Docker setup
│   ├── Dockerfile.admin    # Admin container for builds
│   ├── Dockerfile.service  # Service container template
│   ├── DEPLOY.md   # Deployment workflow guide
│   └── ENV.md      # Environment configuration reference
├── exec/           # Executable services
│   ├── client/     # API and daemon services
│   ├── oracle/     # Oracle validation service
│   ├── validator/  # Application validation service
│   └── stat/       # Statistics collection service
├── data/           # Data access layers
├── core/           # Core utilities and libraries
├── net/            # Network clients and utilities
├── codegen/        # Code generation tools
└── tools/          # Build and development tools
```

## Development

### Prerequisites
- Docker & Docker Compose (for containerized setup)
- Rust 1.70+ (for native development)
- PostgreSQL (for native setup only)
- Redis (for native setup only)
- ClickHouse (for statistics, native setup only)

### Build Tools
- `tools/build_release.sh` - Build optimized release binaries
- `tools/codegen.sh` - Generate code from protobuf definitions

### Testing
```bash
cargo test
```

## Documentation

### Service Documentation
Each service has its own detailed documentation:
- [Client Service](exec/client/README.md) - API and daemon components
- [Oracle Service](exec/oracle/README.md) - Data validation oracle
- [Validator Service](exec/validator/README.md) - Application artifact validation

### Deployment Documentation
Comprehensive deployment guides and tools:
- [Deployment Guide](deploy/DEPLOY.md) - Production deployment workflow
- [Environment Configuration](deploy/ENV.md) - Environment variables reference
- [Docker Compose Setup](deploy/docker-compose.yml) - Multi-service container orchestration

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENCE](LICENCE) file for details.

OpenStore is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.