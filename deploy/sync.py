#!/usr/bin/env python3

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


def download_direct_binaries(shared_root: Path, version: str) -> None:
    base = f"https://github.com/Open-Store-Foundation/store-node/releases/download/{version}"
    any_success = False
    for service, binary in SERVICE_BINARIES.items():
        dst_dir = shared_root / service
        ensure_dir(dst_dir)
        url = f"{base}/{binary}"
        dst_bin = dst_dir / binary
        try:
            with urllib.request.urlopen(url) as resp, open(dst_bin, "wb") as out:
                shutil.copyfileobj(resp, out)
            make_executable(dst_bin)
            any_success = True
        except Exception:
            continue
    return


def ensure_service_dirs(shared_root: Path) -> None:
    for service in SERVICE_BINARIES.keys():
        ensure_dir(shared_root / service)
    ensure_dir(shared_root / "redis")
    ensure_dir(shared_root / "postgres")


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
    default_shared = (Path(__file__).resolve().parent / ".shared").resolve()
    folder_input = prompt("Shared folder path (default: ./.shared): ", "")
    shared_root = Path(folder_input).resolve() if folder_input else default_shared
    ensure_dir(shared_root)
    ensure_service_dirs(shared_root)

    db_name = prompt("SQLite database name (default: bsctest): ", "bsctest")
    ensure_sqlite_file(shared_root, db_name)

    version = prompt("Release version tag (default: local): ", "local")

    if version.lower() == "local":
        copy_local_binaries(repo_root, shared_root)
    else:
        download_direct_binaries(shared_root, version)
    create_launch_scripts(shared_root)

    print(f"Synced to {shared_root}")


if __name__ == "__main__":
    main()


