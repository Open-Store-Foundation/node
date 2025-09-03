#!/usr/bin/env python3

import argparse
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional

class EnvGenerator:
    def __init__(self, config_dir: Path):
        self.config_dir = config_dir
        self.templates = self._define_templates()
        self.profiles = self._define_profiles()
    
    def _define_profiles(self) -> Dict[str, Dict[str, str]]:
        return {
            "bsctest": {
                "CHAIN_ID": "97",
                "ORACLE_ADDRESS": "0F61D8D6c9D6886ac7cba72716E1b98C4379E0f7",
                "STORE_ADDRESS": "6Edac88EA58168a47ab61836bCbAD0Ac844498A6", 
                "HISTORICAL_SYNC_BLOCK": "60727665",
                "HISTORICAL_SYNC_THRESHOLD": "500",
                "CONFIRM_COUNT": "1",
                "GF_NODE_URL": "https://gnfd-testnet-fullnode-tendermint-ap.bnbchain.org",
                "RUST_LOG": "info",
                "TX_POLL_TIMEOUT_MS": "5000"
            },
            "localhost": {
                "CHAIN_ID": "31337",
                "ORACLE_ADDRESS": "Dc64a140Aa3E981100a9becA4E685f962f0cF6C9",
                "STORE_ADDRESS": "0165878A594ca255338adfa4d48449f69242Eb8F",
                "HISTORICAL_SYNC_BLOCK": "0", 
                "HISTORICAL_SYNC_THRESHOLD": "5000",
                "CONFIRM_COUNT": "0",
                "GF_NODE_URL": "https://gnfd-testnet-fullnode-tendermint-ap.bnbchain.org",
                "RUST_LOG": "info",
                "TX_POLL_TIMEOUT_MS": "5000"
            }
        }
    
    def _define_templates(self) -> Dict[str, Dict[str, str]]:
        return {
            "oracle": {
                # TELEGRAM
                "TG_TOKEN": "",
                "TG_INFO_CHAT_ID": "",
                "TG_ALERT_CHAT_ID": "",
                # BLOCKCHAIN
                "ETH_NODE_URL": "",
                "CHAIN_ID": "",
                # WALLET
                "WALLET_PK": "",
                # CONTRACTS
                "ORACLE_ADDRESS": "",
                # SYNC
                "CONFIRM_COUNT": ""
            },
            "validator": {
                # TELEGRAM
                "TG_TOKEN": "",
                "TG_INFO_CHAT_ID": "",
                "TG_ALERT_CHAT_ID": "",
                # BLOCKCHAIN
                "ETH_NODE_URL": "",
                "CHAIN_ID": "",
                "GF_NODE_URL": "",
                # WALLET
                "WALLET_PK": "",
                # CONTRACTS
                "ORACLE_ADDRESS": "",
                "STORE_ADDRESS": "",
                # SYNC
                "HISTORICAL_SYNC_THRESHOLD": "",
                "CONFIRM_COUNT": "",
                "TX_POLL_TIMEOUT_MS": "",
                # DATABASE
                "DATABASE_URL": "",
                # SERVICES
                "FILE_STORAGE_PATH": "./tmp/"
            },
            "daemon-client": {
                # TELEGRAM
                "TG_TOKEN": "",
                "TG_INFO_CHAT_ID": "",
                "TG_ALERT_CHAT_ID": "",
                # BLOCKCHAIN
                "ETH_NODE_URL": "",
                "CHAIN_ID": "",
                "GF_NODE_URL": "",
                "ETHSCAN_API_KEY": "",
                # WALLET
                "WALLET_PK": "",
                # CONTRACTS
                "ORACLE_ADDRESS": "",
                "STORE_ADDRESS": "",
                # SYNC
                "HISTORICAL_SYNC_THRESHOLD": "",
                "HISTORICAL_SYNC_BLOCK": "",
                # DATABASE
                "DATABASE_URL": "",
                # SERVICES
                "REDIS_URL": "",
                "REDIS_HOST": "localhost",
                "REDIS_USER": "",
                "CLIENT_HOST_URL": ""
            },
            "api-client": {
                # TELEGRAM
                "TG_TOKEN": "",
                "TG_INFO_CHAT_ID": "",
                "TG_ALERT_CHAT_ID": "",
                # DATABASE
                "DATABASE_URL": "",
                # SERVICES
                "REDIS_URL": "",
                "REDIS_HOST": "localhost",
                "REDIS_USER": "",
                "CLIENT_HOST_URL": ""
            },
            "postgres": {
                # POSTGRES
                "POSTGRES_HOST": "localhost",
                "POSTGRES_DB": "",
                "POSTGRES_USER": "",
                "POSTGRES_PASSWORD": ""
            }
        }
    
    def collect_inputs(self, profile: Optional[str] = None) -> Dict[str, str]:
        inputs = {}
        
        print("=== OpenStore Environment Configuration ===\n")
        
        selected_profile = None
        if profile and profile in self.profiles:
            inputs.update(self.profiles[profile])
            selected_profile = profile
            print(f"Using {profile} profile defaults...")
        else:
            print("Deployment Profile:")
            print("Available profiles:")
            print("  bsctest - BSC Testnet configuration")
            print("  localhost - Local development configuration")
            
            while True:
                chosen = input("Select profile (bsctest/localhost): ").strip().lower()
                if chosen in self.profiles:
                    inputs.update(self.profiles[chosen])
                    selected_profile = chosen
                    print(f"Using {chosen} profile defaults...")
                    break
                else:
                    print("Invalid profile. Please choose 'bsctest' or 'localhost'")
        
        print("\nPrivate Keys:")
        inputs["ADMIN_WALLET_PK"] = input("Admin Wallet Private Key (for oracle/validator): ").strip()
        inputs["USER_WALLET_PK"] = input("User Wallet Private Key (for clients): ").strip()
        
        print("\nTelegram Configuration:")
        inputs["TG_TOKEN"] = input("Telegram Bot Token: ").strip()
        inputs["TG_INFO_CHAT_ID"] = input("Telegram Info Chat ID: ").strip()
        inputs["TG_ALERT_CHAT_ID"] = input("Telegram Alert Chat ID: ").strip()
        
        print("\nDatabase Configuration:")
        inputs["POSTGRES_HOST"] = input("PostgreSQL Host (default: localhost): ").strip() or "localhost"
        inputs["POSTGRES_DB"] = input("PostgreSQL Database Name: ").strip()
        inputs["POSTGRES_USER"] = input("PostgreSQL Username: ").strip()
        inputs["POSTGRES_PASSWORD"] = input("PostgreSQL Password: ").strip()
        inputs["SQLITE_DB"] = input("SQLite Database Name (for validator): ").strip()
        
        print("\nRedis Configuration:")
        inputs["REDIS_HOST"] = input("Redis Host (default: localhost): ").strip() or "localhost"
        inputs["REDIS_USER"] = input("Redis Username (optional): ").strip()
        inputs["REDIS_PASS"] = input("Redis Password (optional): ").strip()
        
        print("\nValidator File Storage:")
        inputs["FILE_STORAGE_PATH"] = input("FILE_STORAGE_PATH for validator(default './tmp/'): ").strip() or "./tmp/"
        
        print(f"\nBlockchain Configuration:")
        inputs["ETH_NODE_URL"] = input("Ethereum Node URL: ").strip()
        inputs["ETHSCAN_API_KEY"] = input("Etherscan API Key: ").strip()
        inputs["CLIENT_HOST_URL"] = input("Client Host URL (default: 127.0.0.1:8081): ").strip() or "127.0.0.1:8081"
        
        print(f"\nProfile defaults (using {selected_profile}):")
        print(f"CHAIN_ID: {inputs['CHAIN_ID']}")
        print(f"GF_NODE_URL: {inputs['GF_NODE_URL']}")
        print(f"Contract addresses: Oracle={inputs['ORACLE_ADDRESS']}, Store={inputs['STORE_ADDRESS']}")
        
        override = input("Override contract addresses? (y/N): ").strip().lower()
        if override == 'y':
            inputs["ORACLE_ADDRESS"] = input(f"Oracle Contract Address (current: {inputs['ORACLE_ADDRESS']}): ").strip() or inputs["ORACLE_ADDRESS"]
            inputs["STORE_ADDRESS"] = input(f"Store Contract Address (current: {inputs['STORE_ADDRESS']}): ").strip() or inputs["STORE_ADDRESS"]
        
        return inputs
    
    def generate_env_content(self, service: str, inputs: Dict[str, str]) -> str:
        content = [f"# {service.upper()} Environment Configuration"]
        template = self.templates[service]
        
        # Define logical blocks
        blocks = {
            "# TELEGRAM": ["TG_TOKEN", "TG_INFO_CHAT_ID", "TG_ALERT_CHAT_ID"],
            "# BLOCKCHAIN": ["ETH_NODE_URL", "CHAIN_ID", "GF_NODE_URL", "ETHSCAN_API_KEY"],
            "# WALLET": ["WALLET_PK"],
            "# CONTRACTS": ["ORACLE_ADDRESS", "STORE_ADDRESS"],
            "# SYNC": ["HISTORICAL_SYNC_THRESHOLD", "HISTORICAL_SYNC_BLOCK", "CONFIRM_COUNT", "TX_POLL_TIMEOUT_MS"],
            "# DATABASE": ["DATABASE_URL"],
            "# SERVICES": ["REDIS_URL", "REDIS_USER", "CLIENT_HOST_URL", "FILE_STORAGE_PATH"],
            "# POSTGRES": ["POSTGRES_DB", "POSTGRES_USER", "POSTGRES_PASSWORD"]
        }
        
        # Track which keys have been added
        added_keys = set()
        
        # Add variables by logical blocks
        for block_comment, block_keys in blocks.items():
            block_vars = []
            for key in block_keys:
                if key in template:
                    value = self._resolve_value(key, inputs, template[key], service)
                    block_vars.append(f"{key}={value}")
                    added_keys.add(key)
            
            if block_vars:
                content.append("")
                content.append(block_comment)
                content.extend(block_vars)
        
        # Add any remaining variables that don't fit in blocks
        remaining_vars = []
        for key, default_value in template.items():
            if key not in added_keys:
                value = self._resolve_value(key, inputs, default_value, service)
                remaining_vars.append(f"{key}={value}")
        
        if remaining_vars:
            content.append("")
            content.append("# OTHER")
            content.extend(remaining_vars)
        
        return "\n".join(content) + "\n"
    
    def _resolve_value(self, key: str, inputs: Dict[str, str], default: str, service: str = "") -> str:
        if key == "WALLET_PK":
            if service in ["oracle", "validator"]:
                return inputs.get("ADMIN_WALLET_PK", "")
            else:
                return inputs.get("USER_WALLET_PK", "")
        
        mappings = {
            "DATABASE_URL": lambda: self._build_database_url(inputs, service, default),
            "REDIS_URL": lambda: self._build_redis_url(inputs),
            "TX_POLL_TIMEOUT_MS": lambda: inputs.get("TX_POLL_TIMEOUT_MS", default),
        }
        
        resolver = mappings.get(key)
        if resolver:
            resolved = resolver()
            return resolved if resolved else default
        
        return inputs.get(key, default)
    
    def _build_database_url(self, inputs: Dict[str, str], service: str, default: str) -> str:
        if service == "validator":
            # Validator always uses SQLite
            sqlite_db = inputs.get("SQLITE_DB", "bsctest")
            return f"sqlite:///app/sqlite/{sqlite_db}.db"
        elif service == "postgres":
            # Postgres service uses direct values
            return default
        elif service in ["api-client", "daemon-client"]:
            # Client services use PostgreSQL
            return self._build_postgres_url(inputs)
        else:
            return default
    
    def _build_postgres_url(self, inputs: Dict[str, str]) -> str:
        host = inputs.get("POSTGRES_HOST", "localhost")
        db = inputs.get("POSTGRES_DB", "")
        user = inputs.get("POSTGRES_USER", "")
        password = inputs.get("POSTGRES_PASSWORD", "")
        return f"postgresql://{user}:{password}@{host}:5432/{db}"
    
    def _build_redis_url(self, inputs: Dict[str, str]) -> str:
        host = inputs.get("REDIS_HOST", "localhost")
        username = inputs.get("REDIS_USER", "")
        password = inputs.get("REDIS_PASS", "")
        auth = ""
        if username and password:
            auth = f"{username}:{password}@"
        elif password:
            auth = f":{password}@"
        elif username:
            auth = f"{username}@"
        return f"redis://{auth}{host}:6379/0"
    
    def create_service_env(self, service: str, inputs: Dict[str, str]) -> None:
        service_dir = self.config_dir / service
        service_dir.mkdir(parents=True, exist_ok=True)
        
        env_file = service_dir / ".env"
        content = self.generate_env_content(service, inputs)
        
        with open(env_file, 'w') as f:
            f.write(content)
        
        print(f"Created {env_file}")
    
    def create_redis_config(self, redis_password: str) -> None:
        redis_dir = self.config_dir / "redis"
        redis_dir.mkdir(parents=True, exist_ok=True)
        conf_file = redis_dir / "redis.conf"
        lines = [
            "bind 0.0.0.0"
        ]
        if redis_password:
            lines.append(f"requirepass {redis_password}")
        content = "\n".join(lines) + "\n"
        with open(conf_file, "w") as f:
            f.write(content)
        print(f"Created {conf_file}")
    
    def generate_all(self, inputs: Optional[Dict[str, str]] = None, profile: Optional[str] = None) -> None:
        if inputs is None:
            inputs = self.collect_inputs(profile)
        
        print(f"\nGenerating .env files in {self.config_dir}")
        
        for service in self.templates.keys():
            self.create_service_env(service, inputs)
        
        self.create_redis_config(inputs.get("REDIS_PASS", ""))
        
        print("\n✅ All .env files generated successfully!")
        print(f"Configuration files created in: {self.config_dir}")
    
    
    
    def list_services(self) -> None:
        print("Available services:")
        for service in self.templates.keys():
            print(f"  - {service}")
    
    def list_profiles(self) -> None:
        print("Available profiles:")
        for profile_name, profile_config in self.profiles.items():
            print(f"  - {profile_name}: Chain ID {profile_config['CHAIN_ID']}")

def main():
    parser = argparse.ArgumentParser(
        description="Generate .env files for OpenStore deployment",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python env_generator.py                           # Interactive mode
  python env_generator.py --profile bsctest        # Use BSC testnet profile
  python env_generator.py --profile localhost      # Use localhost profile
  python env_generator.py --list-services          # List available services
  python env_generator.py --list-profiles          # List available profiles
  python env_generator.py --service oracle         # Generate for specific service
  python env_generator.py --config-dir DIR         # Target directory
        """
    )
    
    parser.add_argument(
        "--list-services", 
        action="store_true",
        help="List available services"
    )
    
    parser.add_argument(
        "--list-profiles",
        action="store_true", 
        help="List available profiles"
    )
    
    parser.add_argument(
        "--service",
        help="Generate .env for specific service only"
    )
    
    parser.add_argument(
        "--profile",
        choices=["bsctest", "localhost"],
        help="Use predefined profile (bsctest or localhost)"
    )

    parser.add_argument(
        "--config-dir",
        required=True,
        help="Directory where service .env files will be written (e.g., deploy/config)"
    )
    
    args = parser.parse_args()
    
    generator = EnvGenerator(Path(args.config_dir))
    
    if args.list_services:
        generator.list_services()
        return
    
    if args.list_profiles:
        generator.list_profiles()
        return
    
    if args.service:
        if args.service not in generator.templates:
            print(f"Error: Unknown service '{args.service}'")
            generator.list_services()
            sys.exit(1)
        
        inputs = generator.collect_inputs(args.profile)
        generator.create_service_env(args.service, inputs)
    else:
        generator.generate_all(profile=args.profile)

if __name__ == "__main__":
    main()
