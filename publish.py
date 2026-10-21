"""Prepare and manually publish the three crates at a shared version.

Run from the repository root:
    uv run publish.py bump                       # interactive version selection
    uv run publish.py bump 0.1.0-alpha.3         # or supply the version
    uv run publish.py check                     # validate without changing files

Commit the prepared manifests, READMEs and Cargo.lock before publishing.
For each package in this order, dry-run first, then publish:
    uv run publish.py publish router
    uv run publish.py publish router --publish
    uv run publish.py publish style_macros
    uv run publish.py publish style_macros --publish
    uv run publish.py publish main
    uv run publish.py publish main --publish

Wait until each dependency is available on crates.io before checking/publishing
main. A main-package dry-run needs the dependencies to have been published too.
"""

import os
import re
import subprocess
import sys
import threading
import tomllib
from pathlib import Path
from typing import IO

import fire
from loguru import logger

logger.remove()
logger.add(sys.stderr, level=os.getenv("LOG_LEVEL", "INFO"))
logger.add("publish.log", level="TRACE")

README_EN_INSTALL_MATCH = "## 📦 Installation"
README_ZH_INSTALL_MATCH = "## 📦 安装"
CWD = Path.cwd()
CARGO_TOML_EGUI_COMPONENT = CWD / "Cargo.toml"
CARGO_TOML_STYLE_MACROS = CWD / "style_macros" / "Cargo.toml"
CARGO_TOML_ROUTER = CWD / "router" / "Cargo.toml"
ERROR_RE = re.compile(r"\b(error|err)\b", re.IGNORECASE)
WARN_RE = re.compile(r"\b(warning|warn)\b", re.IGNORECASE)
VERSION_RE = re.compile(
    r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"
    r"(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
)


def run_cmd(cmd: list[str], working_dir: Path):
    # An argv list works on POSIX and Windows without shell parsing.
    process = subprocess.Popen(
        cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True, bufsize=1, cwd=working_dir,
    )

    def log_stream(stream: IO[str], prefix: str):
        for line in stream:
            content = line.strip()
            if content:
                level = "ERROR" if ERROR_RE.search(content) else (
                    "WARNING" if WARN_RE.search(content) else "INFO"
                )
                logger.log(level, f"[{prefix}] {content}")

    threads = [threading.Thread(target=log_stream, args=(stream, name))
               for stream, name in [(process.stdout, "stdout"), (process.stderr, "stderr")]]
    for thread in threads:
        thread.start()
    return_code = process.wait()
    for thread in threads:
        thread.join()
    process.stdout.close()
    process.stderr.close()
    if return_code:
        raise subprocess.CalledProcessError(return_code, cmd)


def load_toml(path: Path) -> dict:
    with path.open("rb") as file:
        return tomllib.load(file)


def is_proj_root() -> bool:
    return (CARGO_TOML_EGUI_COMPONENT.exists()
            and load_toml(CARGO_TOML_EGUI_COMPONENT)["package"]["name"] == "egui_component")


def require_proj_root():
    if not is_proj_root():
        raise ValueError("Run publish.py from the repository root")


def get_version_in_readme(path: Path, install_section_match: str):
    in_section = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("## "):
            in_section = line.startswith(install_section_match)
        if in_section and line.startswith("egui_component = "):
            return line.split("=", 1)[1].strip().strip('"')
    raise ValueError(f"Missing installation version in {path}")


def get_versions(main: dict, macros: dict, router: dict):
    version = main["package"]["version"]
    for manifest in (macros, router):
        package = manifest["package"]
        if package["version"] != version:
            raise ValueError(f"Version mismatch: {package['name']}={package['version']}, expected {version}")
    for filename, heading in [("README.md", README_EN_INSTALL_MATCH), ("README.zh.md", README_ZH_INSTALL_MATCH)]:
        if get_version_in_readme(CWD / filename, heading) != version:
            raise ValueError(f"Version in {filename} does not match {version}")
    return version


def current_version():
    return get_versions(load_toml(CARGO_TOML_EGUI_COMPONENT),
                        load_toml(CARGO_TOML_STYLE_MACROS), load_toml(CARGO_TOML_ROUTER))


def replace_toml_value(path: Path, section: str, key: str, value: str):
    """Replace only the requested section/key, preserving other lines and comments."""
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    active = False
    for index, line in enumerate(lines):
        if line.lstrip().startswith("["):
            active = line.strip() == f"[{section}]"
        if active and re.match(rf"^{re.escape(key)}\s*=", line):
            comment = line.partition("#")[2].strip()
            lines[index] = f"{key} = {value}" + (f" # {comment}" if comment else "") + "\n"
            path.write_text("".join(lines), encoding="utf-8")
            return
    raise ValueError(f"Missing [{section}] {key} in {path}")


def bump_cargo_toml_version(path: Path, next_version: str):
    replace_toml_value(path, "package", "version", f'"{next_version}"')


def bump_readme_version(path: Path, install_section_match: str, next_version: str):
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    in_section = False
    for index, line in enumerate(lines):
        if line.startswith("## "):
            in_section = line.startswith(install_section_match)
        if in_section and line.startswith("egui_component = "):
            lines[index] = f'egui_component = "{next_version}"\n'
            path.write_text("".join(lines), encoding="utf-8")
            return
    raise ValueError(f"Missing installation version in {path}")


def sync_self_dependency_version():
    """Keep local paths and crates.io version requirements in sync."""
    version = current_version()
    for section, name, path in [
        ("dependencies", "egui_component_style_macros", "style_macros"),
        ("dev-dependencies", "egui_component_router", "router"),
    ]:
        replace_toml_value(CARGO_TOML_EGUI_COMPONENT, section, name,
                           f'{{ version = "{version}", path = "{path}" }}')


def validate_release():
    version = current_version()
    main = load_toml(CARGO_TOML_EGUI_COMPONENT)
    for path in (CARGO_TOML_EGUI_COMPONENT, CARGO_TOML_STYLE_MACROS, CARGO_TOML_ROUTER):
        if load_toml(path)["package"].get("publish") is False:
            raise ValueError(f"Publishing is disabled in {path}")
    for section, name in [("dependencies", "egui_component_style_macros"),
                          ("dev-dependencies", "egui_component_router")]:
        dependency = main[section][name]
        dependency_version = dependency.get("version") if isinstance(dependency, dict) else dependency
        if dependency_version != version:
            raise ValueError(f"{name} dependency version does not match {version}; run bump")
    return version


def bump_version(next_version: str):
    match = VERSION_RE.fullmatch(next_version)
    if not match or (match[4] and any(part.isdigit() and len(part) > 1 and part[0] == "0"
                                     for part in match[4].split("."))):
        raise ValueError(f"Invalid SemVer version: {next_version}")
    for path in (CARGO_TOML_STYLE_MACROS, CARGO_TOML_ROUTER, CARGO_TOML_EGUI_COMPONENT):
        bump_cargo_toml_version(path, next_version)
    bump_readme_version(CWD / "README.md", README_EN_INSTALL_MATCH, next_version)
    bump_readme_version(CWD / "README.zh.md", README_ZH_INSTALL_MATCH, next_version)
    sync_self_dependency_version()
    run_cmd(["cargo", "update", "--workspace"], CWD)
    validate_release()


class Cli:
    def bump(self, next_version: str | None = None):
        """Update all three crates, internal dependencies, READMEs and Cargo.lock."""
        require_proj_root()
        logger.info(f"Current version: {current_version()}")
        if next_version is None:
            next_version = input("Please enter the next version to publish: ").strip()
        bump_version(next_version)
        logger.info(f"All packages version bumped to: {next_version}; commit the changes before publishing")

    def check(self):
        """Check release versions and publish settings without modifying files."""
        require_proj_root()
        logger.info(f"All three crates are ready at version {validate_release()}")

    def publish(self, pkg: str, publish: bool = False):
        """Dry-run by default; --publish uploads the selected crate to crates.io."""
        require_proj_root()
        packages = {
            "router": CWD / "router", "egui_component_router": CWD / "router",
            "style_macros": CWD / "style_macros", "egui_component_style_macros": CWD / "style_macros",
            "main": CWD, "egui_component": CWD,
        }
        if pkg not in packages:
            raise ValueError(f"Unknown package: {pkg}; choose router, style_macros or main")
        version = validate_release()
        command = ["cargo", "publish"]
        if not publish:
            logger.warning("Dry-run mode; pass --publish to publish to crates.io")
            command.extend(["--dry-run", "--allow-dirty"])
        logger.info(f"{'Publishing' if publish else 'Checking'} {pkg} {version}")
        run_cmd(command, packages[pkg])
        logger.info("Done!")


if __name__ == "__main__":
    fire.Fire(Cli)
