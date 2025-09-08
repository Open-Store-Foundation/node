# 🚀 Deployment Guide

This document outlines the complete deployment process for the OpenStore node application using the automated deployment tools.

## 📋 Prerequisites

Before starting the deployment process, ensure you have:
- Access to the production server
- Python 3.6+ installed
- Docker and Docker Compose installed
- Required permissions for repository operations
- OpenStore admin tools configured
- Sufficient disk space for binaries and data

## 🔄 Deployment Steps

### 1. 📦 Release Preparation
Push changes to the `main` branch and create a new tag and release with version `NUMBER`.

### 2. ⚙️ Build Process
Launch the [build-release.yml](../../.github/workflows/build-release.yml) workflow for the created release tag.

### 3. 🔗 Server Connection
Connect to the target deployment server and navigate to the project directory.

### 4. 📥 Repository Sync
Synchronize the repository on the server with the latest changes:
```bash
git pull origin main
git checkout {TAG_VERSION}
```

### 5. 🔧 Environment Configuration
Use `envgen.py` to generate all environment configurations:

```bash
cd tools/deploy
python3 envgen.py --config-dir ./config --profile bsctest
```

This will:
- Generate `.env` files for all services
- Create nginx configuration files
- Set up redis configuration
- Configure SSL settings if needed

**For SSL/HTTPS setup:**
```bash
# First generate HTTP configuration
python3 envgen.py --service nginx --config-dir ./config
# Select "http" variant

# After obtaining SSL certificate, switch to HTTPS
python3 envgen.py --service nginx --config-dir ./config  
# Select "https" variant
```

### 6. 📲 Binary Synchronization
Execute `sync.py` to download the latest compiled binaries and set up infrastructure:

```bash
python3 sync.py --volume-dir ./volume
```

This will:
- Download release binaries from GitHub
- Create service directory structure
- Generate launch scripts
- Set up SQLite databases
- Create data directories for all services

### 7. 🗄️ Database Migration
Apply database migrations for both:
- **VALIDATOR** service (SQLite)
- **CLIENT** services (PostgreSQL)

Use the `openstore-admin` tool for migration execution.

### 8. 🐳 Container Deployment
Deploy services using Docker Compose:

```bash
# Set environment variables
export CONFIG_DIR=$(pwd)/config
export VOLUME_DIR=$(pwd)/volume

# Start infrastructure services first
docker compose up -d postgres redis

# Wait for services to be ready, then start application services
docker compose up -d oracle validator daemon-client api-client

# Start nginx (if configured)
docker compose up -d nginx
```

### 9. 🔄 Service Health Check
Verify all services are running correctly:

```bash
docker compose ps
docker compose logs -f
```

## ✅ Verification

After deployment, verify that all services are running correctly:

### Service Status
```bash
docker compose ps
```

### Service Logs
```bash
# View all service logs
docker compose logs

# View specific service logs
docker compose logs oracle
docker compose logs validator
docker compose logs api-client
```

### Health Checks
```bash
# Check service health
docker compose exec api-client curl http://localhost:8080/health
docker compose exec postgres pg_isready
docker compose exec redis redis-cli ping
```

### Database Connectivity
```bash
# Test PostgreSQL connection
docker compose exec postgres psql -U {POSTGRES_USER} -d {POSTGRES_DB} -c "SELECT version();"

# Check SQLite database
docker compose exec validator ls -la /app/sqlite/
```

## 🛠️ Troubleshooting

### Common Issues

**Configuration Problems:**
```bash
# Re-generate configuration files
python3 envgen.py --config-dir ./config --profile bsctest

# Check environment variables
docker compose config
```

**Binary Issues:**
```bash
# Re-sync binaries
python3 sync.py --volume-dir ./volume

# Check binary permissions
ls -la volume/*/
```

**Service Failures:**
```bash
# Restart specific service
docker compose restart {service_name}

# View detailed logs
docker compose logs --tail=100 {service_name}
```

## 📚 Additional Documentation

- [Environment Generator Guide](ENVGEN.md) - Detailed envgen.py usage
- [Binary Sync Tool Guide](SYNC.md) - Detailed sync.py usage
- [SSL Setup Guide](../ssl/SSL.txt) - SSL certificate configuration
