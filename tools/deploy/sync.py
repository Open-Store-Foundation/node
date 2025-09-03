#!/usr/bin/env python3

import os
import shutil
import stat
from pathlib import Path
import urllib.request
import urllib.error


SERVICE_BINARIES = {
    "daemon-client": "daemon-client",
    "api-client": "api-client",
    "validator": "validator",
    "oracle": "oracle",
}

def prompt(text: str, default: str) -> str:
    try:
        value = input(text).strip()
        return value or default
    except EOFError:
        return default


def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)

def ensure_sqlite_file(shared_root: Path, db_name: str) -> Path:
    sqlite_dir = shared_root / "sqlite"
    ensure_dir(sqlite_dir)
    db_path = sqlite_dir / f"{db_name}.db"
    if not db_path.exists():
        db_path.touch()
    return db_path


def make_executable(path: Path) -> None:
    try:
        mode = path.stat().st_mode
        path.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    except Exception:
        pass


def copy_local_binaries(repo_root: Path, shared_root: Path) -> None:
    src_dir = repo_root / "target" / "release"
    for service, binary in SERVICE_BINARIES.items():
        dst_dir = shared_root / service
        ensure_dir(dst_dir)
        src_bin = src_dir / binary
        if not src_bin.exists():
            continue
        dst_bin = dst_dir / binary
        shutil.copy2(src_bin, dst_bin)
        make_executable(dst_bin)


def download_direct_binaries(shared_root: Path, version: str, repo_url: str) -> bool:
    base = f"{repo_url}/releases/download/{version}"
    any_success = False
    failed_downloads = []
    
    for service, binary in SERVICE_BINARIES.items():
        dst_dir = shared_root / service
        ensure_dir(dst_dir)
        url = f"{base}/{binary}"
        dst_bin = dst_dir / binary
        try:
            print(f"Downloading {binary} from {url}...")
            with urllib.request.urlopen(url) as resp, open(dst_bin, "wb") as out:
                shutil.copyfileobj(resp, out)
            make_executable(dst_bin)
            any_success = True
            print(f"✓ Successfully downloaded {binary}")
        except urllib.error.HTTPError as e:
            failed_downloads.append(f"{binary}: HTTP {e.code} - {e.reason}")
        except urllib.error.URLError as e:
            failed_downloads.append(f"{binary}: {e.reason}")
        except Exception as e:
            failed_downloads.append(f"{binary}: {str(e)}")
    
    if failed_downloads:
        print("\nDownload failures:")
        for failure in failed_downloads:
            print(f"✗ {failure}")
    
    if not any_success:
        print(f"\n❌ All downloads failed for version {version}")
        print("Please check:")
        print(f"  - Version tag exists: {repo_url}/releases/tag/{version}")
        print("  - Release contains the required binaries")
        print("  - Internet connection is working")
    
    return any_success


def ensure_service_dirs(shared_root: Path) -> None:
    for service in SERVICE_BINARIES.keys():
        ensure_dir(shared_root / service)
    ensure_dir(shared_root / "redis")
    ensure_dir(shared_root / "postgres")
    ensure_dir(shared_root / "certbot" / "conf")
    ensure_dir(shared_root / "certbot" / "www")
    ensure_dir(shared_root / "nginx" / "logs")


def create_launch_scripts(shared_root: Path) -> None:
    for service, binary in SERVICE_BINARIES.items():
        dst_dir = shared_root / service
        ensure_dir(dst_dir)
        launch_path = dst_dir / "launch"
        content = """#!/usr/bin/env sh
cd "$(dirname "$0")"
nohup ./{} >/dev/null 2>&1 &
""".format(binary)
        with open(launch_path, "w") as f:
            f.write(content)
        make_executable(launch_path)


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    default_volume = (Path(__file__).resolve().parent / ".shared").resolve()
    
    repo_url = prompt("Repository URL (default: https://github.com/Open-Store-Foundation/node): ", "https://github.com/Open-Store-Foundation/node")
    
    # Priority: 1. Interactive input, 2. Environment variable, 3. Default
    volume_input = prompt("Volume directory path (default: ./.shared): ", "")
    if volume_input:
        volume_root = Path(volume_input).resolve()
    else:
        # Check environment variable
        env_volume = os.environ.get("VOLUME_DIR")
        if env_volume:
            volume_root = Path(env_volume).resolve()
            print(f"Using VOLUME_DIR from environment: {volume_root}")
        else:
            volume_root = default_volume
    ensure_dir(volume_root)
    ensure_service_dirs(volume_root)

    db_name = prompt("SQLite database name (default: bsctest): ", "bsctest")
    ensure_sqlite_file(volume_root, db_name)

    version = prompt("Release version tag (default: local): ", "local")

    success = True
    if version.lower() == "local":
        copy_local_binaries(repo_root, volume_root)
    else:
        success = download_direct_binaries(volume_root, version, repo_url)
        if not success:
            print(f"\n❌ Sync failed - could not download binaries for version {version}")
            exit(1)
    
    create_launch_scripts(volume_root)
    print(f"\n✅ Successfully synced to {volume_root}")


if __name__ == "__main__":
    main()


