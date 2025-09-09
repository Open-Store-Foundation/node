# OpenStore Binary Sync Tool

A Python CLI tool for synchronizing OpenStore service binaries and setting up deployment infrastructure.

## Features

- Download release binaries from GitHub releases
- Copy local development binaries
- Create service directory structure
- Generate launch scripts for all services
- Set up SQLite databases with proper permissions
- Support for custom repositories and volume directories

## Usage

### Interactive Mode (Recommended)

```bash
cd tools/deploy
python3 sync.py --volume-dir /path/to/volume
```

This will:
1. Prompt for repository URL (default: https://github.com/Open-Store-Foundation/node)
2. Prompt for SQLite database name (default: bsctest)
3. Prompt for release version tag (default: local)
4. Download or copy binaries based on version selection
5. Create launch scripts for all services
6. Set up directory structure and database files

### Using Environment Variables

```bash
# Set VOLUME_DIR to skip volume directory prompt
export VOLUME_DIR=/path/to/volume
python3 sync.py
```

### Command Line Arguments

```bash
# Specify volume directory
python3 sync.py --volume-dir /path/to/volume

# Use custom repository
python3 sync.py --volume-dir /path/to/volume --repo-url https://github.com/custom/repo

# Non-interactive with environment variable
VOLUME_DIR=/path/to/volume python3 sync.py
```

## Service Binaries

The tool manages the following service binaries:

- **daemon-client**: Client daemon service
- **api-client**: API client service  
- **validator**: Validator service
- **oracle**: Oracle service

## Directory Structure

After running, your volume directory will be organized as:

```
volume/
├── daemon-client/
│   ├── daemon-client          # Binary
│   └── launch                 # Launch script
├── api-client/
│   ├── api-client             # Binary
│   └── launch                 # Launch script
├── validator/
│   ├── validator              # Binary
│   └── launch                 # Launch script
├── oracle/
│   ├── oracle                 # Binary
│   └── launch                 # Launch script
├── sqlite/
│   └── {database_name}.db     # SQLite database
├── redis/                     # Redis data directory
├── postgres/                  # PostgreSQL data directory
├── certbot/
│   ├── conf/                  # SSL certificates
│   └── www/                   # Webroot for challenges
└── nginx/
    └── logs/                  # Nginx log files
```

## Binary Sources

### Local Development (`local` version)

When version is set to `local`, the tool copies binaries from:
```
{repo_root}/target/release/{binary_name}
```

This is useful for development and testing with locally built binaries.

### GitHub Releases (version tags)

For any other version tag, the tool downloads binaries from:
```
{repo_url}/releases/download/{version}/{binary_name}
```

Example URLs:
- `https://github.com/Open-Store-Foundation/node/releases/download/v1.0.0/oracle`
- `https://github.com/Open-Store-Foundation/node/releases/download/v1.0.0/validator`

## Launch Scripts

The tool generates launch scripts for each service with the following template:

```bash
#!/usr/bin/env sh
cd "$(dirname "$0")"
nohup ./{binary_name} >/dev/null 2>&1 &
```

Launch scripts are made executable and can be used to start services in the background.

## Database Setup

### SQLite Database

- Creates SQLite database file in `{volume}/sqlite/{db_name}.db`
- Sets 777 permissions for container access
- Handles permission errors gracefully
- Default database name: `bsctest`

### Directory Permissions

All service and data directories are created with appropriate permissions for container mounting.

## Error Handling

### Download Failures

The tool provides detailed error reporting for failed downloads:

- HTTP errors with status codes
- Network connectivity issues  
- Missing release files
- Invalid repository URLs

### Recovery Options

When downloads fail, the tool:
1. Lists all failed downloads with specific error messages
2. Provides troubleshooting suggestions
3. Exits with error code for automation integration

## Examples

### Production Deployment

```bash
# Download v1.2.0 release binaries
python3 sync.py --volume-dir /opt/openstore/data
# Enter: v1.2.0 when prompted for version
```

### Development Setup

```bash
# Use local binaries for development
export VOLUME_DIR=/tmp/openstore-dev
python3 sync.py
# Enter: local when prompted for version
```

### Custom Repository

```bash
# Use fork or custom repository
python3 sync.py \
  --volume-dir /data/openstore \
  --repo-url https://github.com/myorg/openstore-node
```

### Automated Deployment

```bash
#!/bin/bash
export VOLUME_DIR=/opt/openstore/data
export REPO_URL=https://github.com/Open-Store-Foundation/node

# Non-interactive sync with specific version
echo -e "\n\nbsctest\nv1.2.0" | python3 sync.py
```

## Integration with Docker

The sync tool is designed to work with Docker Compose deployments:

1. **Volume Directory**: Maps to Docker volume mounts
2. **Service Structure**: Matches container expectations  
3. **Launch Scripts**: Can be executed from containers
4. **Permissions**: Set appropriately for container access

Example Docker Compose integration:
```yaml
volumes:
  - ${VOLUME_DIR}/oracle:/app/oracle
  - ${VOLUME_DIR}/sqlite:/app/sqlite
```

## Requirements

- Python 3.6+
- Internet connection (for release downloads)
- Write permissions to volume directory
- No external dependencies (uses standard library only)

## Troubleshooting

### Permission Denied Errors

```bash
# Ensure write permissions to volume directory
sudo chown -R $USER:$USER /path/to/volume
chmod -R 755 /path/to/volume
```

### Download Failures

```bash
# Check release exists
curl -I https://github.com/Open-Store-Foundation/node/releases/download/v1.0.0/oracle

# Verify internet connectivity
ping github.com
```

### Missing Local Binaries

```bash
# Build binaries first
cd {repo_root}
cargo build --release
```

### Database Permission Issues

The tool attempts to set 777 permissions on SQLite databases but handles failures gracefully. Manual permission setting may be required:

```bash
chmod 777 /path/to/volume/sqlite/*.db
```



