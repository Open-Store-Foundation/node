#!/usr/bin/env python3

import argparse
import os
import sys
import shutil
from pathlib import Path
from typing import Dict, List, Optional

class EnvGenerator:
    def __init__(self, config_dir: Path):
        self.config_dir = config_dir
        self.templates = self._define_templates()
        self.profiles = self._define_profiles()
    
    def _parse_env_file(self, path: Path) -> Dict[str, str]:
        values: Dict[str, str] = {}
        try:
            with open(path, 'r') as f:
                for raw in f:
                    line = raw.strip()
                    if not line or line.startswith('#'):
                        continue
                    if '=' not in line:
                        continue
                    key, val = line.split('=', 1)
                    key = key.strip()
                    val = val.strip()
                    if (val.startswith('"') and val.endswith('"')) or (val.startswith("'") and val.endswith("'")):
                        val = val[1:-1]
                    values[key] = val
        except FileNotFoundError:
            print(f"Warning: config file not found: {path}")
        except Exception as e:
            print(f"Warning: failed to read config file {path}: {e}")
        return values
    
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
                "CONFIRM_COUNT": "",
                "TX_POLL_TIMEOUT_MS": "",
                # SERVICES
                "LOG_PATH": ""
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
                "FILE_STORAGE_PATH": "./tmp/",
                "LOG_PATH": ""
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
                "LOG_PATH": ""
            },
            "api-client": {
                # TELEGRAM
                "TG_TOKEN": "",
                "TG_INFO_CHAT_ID": "",
                "TG_ALERT_CHAT_ID": "",
                # WALLET
                "WALLET_PK": "",
                # DATABASE
                "DATABASE_URL": "",
                # SERVICES
                "REDIS_URL": "",
                "CLIENT_HOST_URL": "",
                "LOG_PATH": ""
            },
            "postgres": {
                # POSTGRES
                "POSTGRES_HOST": "postgres",
                "POSTGRES_DB": "",
                "POSTGRES_USER": "",
                "POSTGRES_PASSWORD": "",
                "DATA_SOURCE_NAME": "postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:5432/${POSTGRES_DB}?sslmode=disable"
            },
            "nginx": {
                # NGINX
                "DOMAIN_NAME": "",
                "NGINX_VARIANT": "",
                "CERTBOT_EMAIL": ""
            },
            "grafana": {
                "GRAFANA_VARIANT": "",
                "GRAFANA_REMOTE_WRITE_URL": "",
                "GRAFANA_REMOTE_WRITE_USER": "",
                "GRAFANA_REMOTE_WRITE_PASSWORD": ""
            }
        }
    
    def get_service_input_requirements(self, service: str) -> set:
        template = self.templates.get(service, {})
        required_inputs = set()
        
        for key in template.keys():
            if key == "WALLET_PK":
                if service in ["oracle", "validator"]:
                    required_inputs.add("ADMIN_WALLET_PK")
                else:
                    required_inputs.add("USER_WALLET_PK")
            elif key == "DATABASE_URL":
                if service == "validator":
                    required_inputs.add("SQLITE_DB")
                elif service in ["api-client", "daemon-client"]:
                    required_inputs.update(["POSTGRES_HOST", "POSTGRES_DB", "POSTGRES_USER", "POSTGRES_PASSWORD"])
            elif key == "REDIS_URL":
                required_inputs.update(["REDIS_HOST", "REDIS_USER", "REDIS_PASS"])
            elif key in ["TG_TOKEN", "TG_INFO_CHAT_ID", "TG_ALERT_CHAT_ID"]:
                required_inputs.update(["TG_TOKEN", "TG_INFO_CHAT_ID", "TG_ALERT_CHAT_ID"])
            elif key == "ETH_NODE_URL":
                required_inputs.add("ETH_NODE_URL")
            elif key == "ETHSCAN_API_KEY":
                required_inputs.add("ETHSCAN_API_KEY")
            elif key == "CLIENT_HOST_URL":
                required_inputs.add("CLIENT_HOST_URL")
            elif key == "FILE_STORAGE_PATH":
                required_inputs.add("FILE_STORAGE_PATH")
            elif key == "DOMAIN_NAME":
                required_inputs.add("DOMAIN_NAME")
            elif key == "NGINX_VARIANT":
                required_inputs.add("NGINX_VARIANT")
            elif key == "CERTBOT_EMAIL":
                required_inputs.add("CERTBOT_EMAIL")
        
        return required_inputs
    
    def collect_inputs(self, profile: Optional[str] = None, service: Optional[str] = None, config_path: Optional[str] = None) -> Dict[str, str]:
        inputs: Dict[str, str] = {}
        
        print("=== OpenStore Environment Configuration ===\n")
        
        if service:
            print(f"Configuring for service: {service}")
            required_inputs = self.get_service_input_requirements(service)
        else:
            print("Configuring for all services")
            required_inputs = set()
            for svc in self.templates.keys():
                required_inputs.update(self.get_service_input_requirements(svc))
        
        if config_path:
            preset = self._parse_env_file(Path(config_path))
            inputs.update(preset)
        
        # Check if service needs profile-specific configuration
        profile_dependent_keys = {"CHAIN_ID", "ORACLE_ADDRESS", "STORE_ADDRESS", "HISTORICAL_SYNC_BLOCK", 
                                "HISTORICAL_SYNC_THRESHOLD", "CONFIRM_COUNT", "GF_NODE_URL", "TX_POLL_TIMEOUT_MS"}
        if service:
            keys_in_scope = set(self.templates.get(service, {}).keys())
        else:
            keys_in_scope = set()
            for svc in self.templates.keys():
                keys_in_scope.update(self.templates[svc].keys())
        needs_profile = any((k in keys_in_scope) and (k not in inputs) for k in profile_dependent_keys)
        
        selected_profile = None
        if needs_profile:
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
        else:
            print("Service doesn't require blockchain profile configuration.")
        
        if ("ADMIN_WALLET_PK" in required_inputs and "ADMIN_WALLET_PK" not in inputs) or ("USER_WALLET_PK" in required_inputs and "USER_WALLET_PK" not in inputs):
            print("\nPrivate Keys:")
            if "ADMIN_WALLET_PK" in required_inputs and "ADMIN_WALLET_PK" not in inputs:
                inputs["ADMIN_WALLET_PK"] = input("Admin Wallet Private Key (for oracle/validator): ").strip()
            if "USER_WALLET_PK" in required_inputs and "USER_WALLET_PK" not in inputs:
                inputs["USER_WALLET_PK"] = input("User Wallet Private Key (for clients): ").strip()
        
        telegram_inputs = {"TG_TOKEN", "TG_INFO_CHAT_ID", "TG_ALERT_CHAT_ID"}
        if any(k in required_inputs and k not in inputs for k in telegram_inputs):
            print("\nTelegram Configuration:")
            if "TG_TOKEN" in required_inputs and "TG_TOKEN" not in inputs:
                inputs["TG_TOKEN"] = input("Telegram Bot Token: ").strip()
            if "TG_INFO_CHAT_ID" in required_inputs and "TG_INFO_CHAT_ID" not in inputs:
                inputs["TG_INFO_CHAT_ID"] = input("Telegram Info Chat ID: ").strip()
            if "TG_ALERT_CHAT_ID" in required_inputs and "TG_ALERT_CHAT_ID" not in inputs:
                inputs["TG_ALERT_CHAT_ID"] = input("Telegram Alert Chat ID: ").strip()
        
        database_inputs = {"POSTGRES_HOST", "POSTGRES_DB", "POSTGRES_USER", "POSTGRES_PASSWORD", "SQLITE_DB"}
        if any(k in required_inputs and k not in inputs for k in database_inputs):
            print("\nDatabase Configuration:")
            if "POSTGRES_HOST" in required_inputs and "POSTGRES_HOST" not in inputs:
                inputs["POSTGRES_HOST"] = input("PostgreSQL Host (default: postgres): ").strip() or "postgres"
            if "POSTGRES_DB" in required_inputs and "POSTGRES_DB" not in inputs:
                inputs["POSTGRES_DB"] = input("PostgreSQL Database Name: ").strip()
            if "POSTGRES_USER" in required_inputs and "POSTGRES_USER" not in inputs:
                inputs["POSTGRES_USER"] = input("PostgreSQL Username: ").strip()
            if "POSTGRES_PASSWORD" in required_inputs and "POSTGRES_PASSWORD" not in inputs:
                inputs["POSTGRES_PASSWORD"] = input("PostgreSQL Password: ").strip()
            if "SQLITE_DB" in required_inputs and "SQLITE_DB" not in inputs:
                inputs["SQLITE_DB"] = input("SQLite Database Name (for validator): ").strip()
        if "DATA_SOURCE_NAME" not in inputs:
            inputs["DATA_SOURCE_NAME"] = "postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:5432/${POSTGRES_DB}?sslmode=disable"
        
        redis_inputs = {"REDIS_URL"}
        if any(k in required_inputs and k not in inputs for k in redis_inputs):
            print("\nRedis Configuration:")
            if "REDIS_HOST" in required_inputs and "REDIS_HOST" not in inputs:
                inputs["REDIS_HOST"] = input("Redis Host (default: redis): ").strip() or "redis"
            if "REDIS_USER" in required_inputs and "REDIS_USER" not in inputs:
                inputs["REDIS_USER"] = input("Redis Username (optional): ").strip()
            if "REDIS_PASS" in required_inputs and "REDIS_PASS" not in inputs:
                inputs["REDIS_PASS"] = input("Redis Password (optional): ").strip()
        
        if "FILE_STORAGE_PATH" in required_inputs and "FILE_STORAGE_PATH" not in inputs:
            print("\nValidator File Storage:")
            inputs["FILE_STORAGE_PATH"] = input("FILE_STORAGE_PATH for validator(default './tmp/'): ").strip() or "./tmp/"
        
        # Ask for log dir base when logging is used by any selected service template
        needs_logging = False
        if service:
            needs_logging = "LOG_PATH" in self.templates.get(service, {})
        else:
            for svc in self.templates.keys():
                if "LOG_PATH" in self.templates[svc]:
                    needs_logging = True
                    break
        if needs_logging and ("LOG_DIR" not in inputs):
            print("\nLogging:")
            inputs["LOG_DIR"] = input("Base LOG_DIR (default './log'): ").strip() or "./log"
        
        nginx_inputs = {"DOMAIN_NAME", "NGINX_VARIANT", "CERTBOT_EMAIL"}
        if any(k in required_inputs and k not in inputs for k in nginx_inputs):
            print("\nNginx Configuration:")
            if "NGINX_VARIANT" in required_inputs and "NGINX_VARIANT" not in inputs:
                print("Available nginx variants:")
                print("  http - HTTP only configuration")
                print("  https - HTTPS with SSL certificates")
                print("  none - No nginx configuration files")
                while True:
                    variant = input("Select nginx variant (http/https/none): ").strip().lower()
                    if variant in ["http", "https", "none"]:
                        inputs["NGINX_VARIANT"] = variant
                        break
                    else:
                        print("Invalid variant. Please choose 'http', 'https', or 'none'")
                
                if inputs["NGINX_VARIANT"] != "none" and "DOMAIN_NAME" in required_inputs and "DOMAIN_NAME" not in inputs:
                    inputs["DOMAIN_NAME"] = input("Domain name (e.g., example.com): ").strip()
                
                if inputs["NGINX_VARIANT"] in ["http", "https"] and "CERTBOT_EMAIL" in required_inputs and "CERTBOT_EMAIL" not in inputs:
                    inputs["CERTBOT_EMAIL"] = input("Email for SSL certificates (Let's Encrypt): ").strip()
        # Grafana configuration prompt (always ask if not preset)
        if "GRAFANA_VARIANT" not in inputs:
            print("\nGrafana Configuration:")
            print("  full - Enable Grafana Agent with remote write")
            print("  none - Do not configure Grafana")
            while True:
                g_variant = input("Select grafana variant (full/none): ").strip().lower()
                if g_variant in ["full", "none"]:
                    inputs["GRAFANA_VARIANT"] = g_variant
                    break
                else:
                    print("Invalid variant. Please choose 'full' or 'none'")
        if inputs.get("GRAFANA_VARIANT") == "full":
            if "GRAFANA_REMOTE_WRITE_URL" not in inputs:
                inputs["GRAFANA_REMOTE_WRITE_URL"] = input("Grafana Remote Write URL: ").strip()
            if "GRAFANA_REMOTE_WRITE_USER" not in inputs:
                inputs["GRAFANA_REMOTE_WRITE_USER"] = input("Grafana Remote Write Username: ").strip()
            if "GRAFANA_REMOTE_WRITE_PASSWORD" not in inputs:
                inputs["GRAFANA_REMOTE_WRITE_PASSWORD"] = input("Grafana Remote Write Password: ").strip()
        
        blockchain_inputs = {"ETH_NODE_URL", "ETHSCAN_API_KEY", "CLIENT_HOST_URL"}
        if any(k in required_inputs and k not in inputs for k in blockchain_inputs):
            print(f"\nBlockchain Configuration:")
            if "ETH_NODE_URL" in required_inputs and "ETH_NODE_URL" not in inputs:
                inputs["ETH_NODE_URL"] = input("Ethereum Node URL: ").strip()
            if "ETHSCAN_API_KEY" in required_inputs and "ETHSCAN_API_KEY" not in inputs:
                inputs["ETHSCAN_API_KEY"] = input("Etherscan API Key: ").strip()
            if "CLIENT_HOST_URL" in required_inputs and "CLIENT_HOST_URL" not in inputs:
                inputs["CLIENT_HOST_URL"] = input("Client Host URL (default: 127.0.0.1:8080): ").strip() or "127.0.0.1:8080"
        
        if selected_profile and (service is None or any(key in self.templates.get(service, {}) for key in ["ORACLE_ADDRESS", "STORE_ADDRESS"])):
            print(f"\nProfile defaults (using {selected_profile}):")
            print(f"CHAIN_ID: {inputs['CHAIN_ID']}")
            print(f"GF_NODE_URL: {inputs['GF_NODE_URL']}")
            print(f"Contract addresses: Oracle={inputs['ORACLE_ADDRESS']}, Store={inputs['STORE_ADDRESS']}")
            
            override = input("Override contract addresses? (y/N): ").strip().lower()
            if override == 'y':
                inputs["ORACLE_ADDRESS"] = input(f"Oracle Contract Address (current: {inputs['ORACLE_ADDRESS']}): ").strip() or inputs["ORACLE_ADDRESS"]
                inputs["STORE_ADDRESS"] = input(f"Store Contract Address (current: {inputs['STORE_ADDRESS']}): ").strip() or inputs["STORE_ADDRESS"]
        
        return inputs
    
    def write_output_env(self, inputs: Dict[str, str], output_path: str) -> None:
        try:
            with open(output_path, 'w') as f:
                sections: List[tuple] = [
                    ("# TELEGRAM", ["TG_TOKEN", "TG_INFO_CHAT_ID", "TG_ALERT_CHAT_ID"]),
                    ("# WALLET", ["ADMIN_WALLET_PK", "USER_WALLET_PK"]),
                    ("# BLOCKCHAIN", ["ETH_NODE_URL", "CHAIN_ID", "GF_NODE_URL", "ETHSCAN_API_KEY"]),
                    ("# CONTRACTS", ["ORACLE_ADDRESS", "STORE_ADDRESS"]),
                    ("# SYNC", ["HISTORICAL_SYNC_THRESHOLD", "HISTORICAL_SYNC_BLOCK", "CONFIRM_COUNT", "TX_POLL_TIMEOUT_MS"]),
                    ("# DATABASE", ["DATABASE_URL"]),
                    ("# POSTGRES", ["POSTGRES_HOST", "POSTGRES_DB", "POSTGRES_USER", "POSTGRES_PASSWORD"]),
                    ("# SERVICES", ["REDIS_URL", "CLIENT_HOST_URL", "FILE_STORAGE_PATH", "LOG_DIR", "LOG_PATH"]),
                    ("# NGINX", ["NGINX_VARIANT", "DOMAIN_NAME", "CERTBOT_EMAIL"]),
                    ("# GRAFANA", ["GRAFANA_REMOTE_WRITE_URL", "GRAFANA_REMOTE_WRITE_USER", "GRAFANA_REMOTE_WRITE_PASSWORD"]),
                ]
                written: set = set()
                first_block = True
                for header, keys in sections:
                    present = [k for k in keys if k in inputs and inputs[k] != ""]
                    if not present:
                        continue
                    if not first_block:
                        f.write("\n")
                    first_block = False
                    f.write(f"{header}\n")
                    for key in present:
                        f.write(f"{key}={inputs[key]}\n")
                        written.add(key)
                remaining = sorted(k for k in inputs.keys() if k not in written)
                if remaining:
                    if not first_block:
                        f.write("\n")
                    f.write("# OTHER\n")
                    for key in remaining:
                        f.write(f"{key}={inputs[key]}\n")
            print(f"Created consolidated env: {output_path}")
        except Exception as e:
            print(f"Warning: failed to write output env {output_path}: {e}")
    
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
            "# SERVICES": ["REDIS_URL", "CLIENT_HOST_URL", "FILE_STORAGE_PATH", "LOG_PATH"],
            "# POSTGRES": ["POSTGRES_DB", "POSTGRES_USER", "POSTGRES_PASSWORD"],
            "# NGINX": ["DOMAIN_NAME", "CERTBOT_EMAIL"],
            "# GRAFANA": ["GRAFANA_REMOTE_WRITE_URL", "GRAFANA_REMOTE_WRITE_USER", "GRAFANA_REMOTE_WRITE_PASSWORD"]
        }
        
        # Track which keys have been added
        added_keys = set()
        
        # Add variables by logical blocks
        for block_comment, block_keys in blocks.items():
            block_vars = []
            for key in block_keys:
                if key in template:
                    value = self._resolve_value(key, inputs, template[key], service)
                    if value is not None:
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
                if value is not None:
                    remaining_vars.append(f"{key}={value}")
        
        if remaining_vars:
            content.append("")
            content.append("# OTHER")
            content.extend(remaining_vars)
        
        return "\n".join(content) + "\n"
    
    def _resolve_value(self, key: str, inputs: Dict[str, str], default: str, service: str = "") -> str:
        # Skip NGINX_VARIANT for nginx service - it's only used for generation logic
        if key == "NGINX_VARIANT" and service == "nginx":
            return None
        if key == "GRAFANA_VARIANT" and service == "grafana":
            return None
            
        if key == "WALLET_PK":
            if service in ["oracle", "validator"]:
                return inputs.get("ADMIN_WALLET_PK", "")
            else:
                return inputs.get("USER_WALLET_PK", "")
        
        mappings = {
            "DATABASE_URL": lambda: self._build_database_url(inputs, service, default),
            "REDIS_URL": lambda: self._build_redis_url(inputs),
            "TX_POLL_TIMEOUT_MS": lambda: inputs.get("TX_POLL_TIMEOUT_MS", default),
            "LOG_PATH": lambda: f"{inputs.get('LOG_DIR', './log').rstrip('/')}/{service}.log" if 'LOG_DIR' in inputs or default == "" else default,
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
    
    def create_nginx_config(self, inputs: Dict[str, str]) -> None:
        nginx_variant = inputs.get("NGINX_VARIANT", "none")
        domain_name = inputs.get("DOMAIN_NAME", "")
        
        nginx_dir = self.config_dir / "nginx"
        nginx_dir.mkdir(parents=True, exist_ok=True)
        
        project_root = Path(__file__).parent.parent.parent
        source_nginx_dir = project_root / "tools" / "templates" / "nginx"
        
        if not source_nginx_dir.exists():
            print(f"Warning: Nginx templates not found at {source_nginx_dir}")
            return
        
        shutil.copy2(source_nginx_dir / "nginx.conf", nginx_dir / "nginx.conf")
        print(f"Created {nginx_dir / 'nginx.conf'}")
        
        if nginx_variant == "none":
            print("Nginx variant set to 'none' - no site configuration created")
            return
        
        if nginx_variant == "http":
            source_template = source_nginx_dir / "templates" / "openstore-initial.conf.template"
            target_template = nginx_dir / "default.conf.template"
        elif nginx_variant == "https":
            source_template = source_nginx_dir / "templates" / "openstore-ssl.conf.template"
            target_template = nginx_dir / "default.conf.template"
        else:
            print(f"Warning: Unknown nginx variant '{nginx_variant}'")
            return
        
        if source_template.exists():
            shutil.copy2(source_template, target_template)
            print(f"Created {target_template} (variant: {nginx_variant})")
        else:
            print(f"Warning: Template not found at {source_template}")
    
    def create_service_env(self, service: str, inputs: Dict[str, str]) -> None:
        if service == "nginx":
            self.create_nginx_config(inputs)
        if service == "grafana":
            self.create_grafana_config(inputs)
            if inputs.get("GRAFANA_VARIANT") == "none":
                print("Grafana variant set to 'none' - no grafana .env created")
                return
            
        service_dir = self.config_dir / service
        service_dir.mkdir(parents=True, exist_ok=True)
        
        env_file = service_dir / ".env"
        content = self.generate_env_content(service, inputs)
        
        with open(env_file, 'w') as f:
            f.write(content)
        
        print(f"Created {env_file}")

    def create_grafana_config(self, inputs: Dict[str, str]) -> None:
        grafana_variant = inputs.get("GRAFANA_VARIANT", "none")
        grafana_dir = self.config_dir / "grafana"
        grafana_dir.mkdir(parents=True, exist_ok=True)
        if grafana_variant != "full":
            print("Grafana variant is not 'full' - skipping agent.yaml")
            return
        project_root = Path(__file__).parent.parent.parent
        template_dir = project_root / "tools" / "templates" / "grafana"
        source_grafana = template_dir / "agent.yaml"
        target_grafana = grafana_dir / "agent.yaml"
        if source_grafana.exists():
            shutil.copy2(source_grafana, target_grafana)
            print(f"Created {target_grafana}")
        else:
            print(f"Warning: Grafana agent template not found at {source_grafana}")
    
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
    
    def generate_all(self, inputs: Optional[Dict[str, str]] = None, profile: Optional[str] = None, config_path: Optional[str] = None, output_path: Optional[str] = None) -> None:
        if inputs is None:
            inputs = self.collect_inputs(profile, None, config_path)
        
        print(f"\nGenerating .env files in {self.config_dir}")
        
        for service in self.templates.keys():
            self.create_service_env(service, inputs)
        
        self.create_redis_config(inputs.get("REDIS_PASS", ""))
        
        if output_path:
            self.write_output_env(inputs, output_path)
        
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
  python env_generator.py --service nginx          # Generate nginx config (http/https/none)
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
        help="Directory where service .env files will be written (e.g., deploy/config)"
    )
    
    # Removed --volume-dir: LOG_PATH is internal with default ./log/app.log
    
    parser.add_argument(
        "--input",
        help="Path to a consolidated env file to pre-seed inputs"
    )
    
    parser.add_argument(
        "--output",
        help="Path to write a consolidated env file for reuse"
    )
    
    args = parser.parse_args()
    
    # Handle help commands that don't need config-dir
    if args.list_services:
        # Use a dummy path for help commands
        generator = EnvGenerator(Path("."))
        generator.list_services()
        return
    
    if args.list_profiles:
        # Use a dummy path for help commands
        generator = EnvGenerator(Path("."))
        generator.list_profiles()
        return
    
    # For actual generation, config-dir is required
    config_dir = None
    if args.config_dir:
        # Priority 1: Command line argument
        config_dir = args.config_dir
    else:
        # Priority 2: Environment variable
        env_config_dir = os.environ.get("CONFIG_DIR")
        if env_config_dir:
            config_dir = env_config_dir
            print(f"Using CONFIG_DIR from environment: {config_dir}")
        else:
            # Priority 3: Ask user to define
            print("Error: --config-dir is required for service generation")
            print("You can either:")
            print("  1. Use --config-dir argument")
            print("  2. Set CONFIG_DIR environment variable")
            parser.print_help()
            sys.exit(1)
    
    generator = EnvGenerator(Path(config_dir))
    
    if args.service:
        if args.service not in generator.templates:
            print(f"Error: Unknown service '{args.service}'")
            generator.list_services()
            sys.exit(1)
        
        inputs = generator.collect_inputs(args.profile, args.service, args.input)
        generator.create_service_env(args.service, inputs)
        if args.output:
            generator.write_output_env(inputs, args.output)
    else:
        generator.generate_all(profile=args.profile, config_path=args.input, output_path=args.output)

if __name__ == "__main__":
    main()
