"""
自动化发布脚本

1. uv sync
2. uv run publish.py
"""

import os
import sys
import platform
import subprocess
import threading
import tomllib
import re
from typing import IO
from pathlib import Path

import fire
from loguru import logger

logger.remove()
logger.add(sys.stderr, level=os.getenv("LOG_LEVEL", "INFO"))
logger.add("publish.log", level="TRACE")

README_EN_INSTALL_MATCH = "## 📦 Installation"
README_ZH_INSTALL_MATCH = "## 📦 安装"

CWD = Path(os.getcwd())
CARGO_TOML_EGUI_COMPONENT = CWD / "Cargo.toml"
CARGO_TOML_STYLE_MACROS = CWD / "style_macros" / "Cargo.toml"

ERROR_RE = re.compile(r"\b(error|err)\b", re.IGNORECASE)
WARN_RE = re.compile(r"\b(warning|warn)\b", re.IGNORECASE)


def run_cmd(cmd: str, working_dir: Path):
    shell = platform.system() == "Windows"

    # 使用 Popen 开启管道
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        shell=shell,
        text=True,
        bufsize=1,
        cwd=working_dir,
    )

    def log_stream(stream: IO[str], prefix: str):
        for line in stream:
            if line:
                content = line.strip()
                if ERROR_RE.search(content):
                    level = "ERROR"
                elif WARN_RE.search(content):
                    level = "WARNING"
                else:
                    level = "INFO"
                logger.log(level, f"[{prefix}] {content}")

    # 同时读取 stdout 和 stderr，避免缓冲区满导致死锁
    t1 = threading.Thread(target=log_stream, args=(process.stdout, "stdout"))
    t2 = threading.Thread(target=log_stream, args=(process.stderr, "stderr"))

    t1.start()
    t2.start()

    return_code = process.wait()
    t1.join()
    t2.join()

    if return_code != 0:
        raise subprocess.CalledProcessError(return_code, cmd)


def load_toml(path: Path) -> dict:
    with open(path, "rb") as f:
        return tomllib.load(f)


def is_proj_root() -> bool:
    if not CARGO_TOML_EGUI_COMPONENT.exists():
        return False

    with open(CARGO_TOML_EGUI_COMPONENT, "rb") as f:
        data = tomllib.load(f)
        return data["package"]["name"] == "egui_component"


def get_versions(egui_component_toml: dict, style_macros_toml: dict):
    egui_component_version = egui_component_toml["package"]["version"]
    style_macros_version = style_macros_toml["package"]["version"]

    if egui_component_version != style_macros_version:
        raise ValueError(
            f"Versions do not match: {egui_component_version} != {style_macros_version}"
        )

    readme_en_version = get_version_in_readme(
        CWD / "README.md", README_EN_INSTALL_MATCH
    )
    if readme_en_version != egui_component_version:
        raise ValueError(
            f"Version in README.md does not match: {readme_en_version} != {egui_component_version}"
        )

    readme_zh_version = get_version_in_readme(
        CWD / "README.zh.md", README_ZH_INSTALL_MATCH
    )
    if readme_zh_version != egui_component_version:
        raise ValueError(
            f"Version in README.zh.md does not match: {readme_zh_version} != {egui_component_version}"
        )

    return egui_component_version


def get_version_in_readme(path: Path, install_section_match: str):
    with open(path, "r", encoding="utf-8") as f:
        data = f.read()
        is_installation_section = False
        for line in data.split("\n"):
            if line.startswith(install_section_match):
                is_installation_section = True

            if is_installation_section and line.startswith("egui_component = "):
                return line.split("=")[1].strip().strip('"')

    return None


def bump_cargo_toml_version(path: Path, next_version: str):
    with open(path, "r", encoding="utf-8") as f:
        data = f.read()
        # very simple judgement
        is_pkg_section = False
        for line in data.split("\n"):
            if line.startswith("[package]"):
                is_pkg_section = True

            if is_pkg_section and line.startswith("version = "):
                data = data.replace(line, f'version = "{next_version}"')
                break

    with open(path, "w", encoding="utf-8") as f:
        f.write(data)


def bump_readme_version(path: Path, install_section_match: str, next_version: str):
    with open(path, "r", encoding="utf-8") as f:
        data = f.read()
        is_installation_section = False
        for line in data.split("\n"):
            if line.startswith(install_section_match):
                is_installation_section = True

            if is_installation_section and line.startswith("egui_component = "):
                data = data.replace(line, f'egui_component = "{next_version}"')
                break

    with open(path, "w", encoding="utf-8") as f:
        f.write(data)


def sync_self_dependency_version():
    """
    Sync the version of style_macros in egui_component's Cargo.toml
    """
    egui_component_toml = load_toml(CARGO_TOML_EGUI_COMPONENT)
    style_macros_toml = load_toml(CARGO_TOML_STYLE_MACROS)
    version = get_versions(egui_component_toml, style_macros_toml)

    with open(CARGO_TOML_EGUI_COMPONENT, "r", encoding="utf-8") as f:
        data = f.read()
        is_dependency_section = False
        for line in data.split("\n"):
            if line.startswith("[dependencies]"):
                is_dependency_section = True

            if is_dependency_section and line.startswith("style_macros = "):
                data = data.replace(line, f'style_macros = "{version}"')
                break

    with open(CARGO_TOML_EGUI_COMPONENT, "w", encoding="utf-8") as f:
        f.write(data)


def bump_version(next_version: str):
    bump_cargo_toml_version(CARGO_TOML_STYLE_MACROS, next_version)
    bump_cargo_toml_version(CARGO_TOML_EGUI_COMPONENT, next_version)
    bump_readme_version(CWD / "README.md", README_EN_INSTALL_MATCH, next_version)
    bump_readme_version(CWD / "README.zh.md", README_ZH_INSTALL_MATCH, next_version)


class Cli:
    def bump(self):
        """
        Bump the version of all packages.
        """
        if not is_proj_root():
            logger.error("Not in project root directory")
            return

        egui_component_toml = load_toml(CARGO_TOML_EGUI_COMPONENT)
        style_macros_toml = load_toml(CARGO_TOML_STYLE_MACROS)
        version = get_versions(egui_component_toml, style_macros_toml)
        logger.info(f"Current version: {version}")

        next_version = input("Please enter the next version to publish: ")
        if not next_version:
            logger.error("Version cannot be empty")
            return

        bump_version(next_version)
        logger.info(f"All packages version bumped to: {next_version}")

    def publish(self, pkg: str, publish: bool = False):
        if not is_proj_root():
            logger.error("Not in project root directory")
            return

        if not publish:
            logger.warning(
                "Default in dry-run mode, pass --publish to publish packages"
            )

        if pkg == "style_macros" or pkg == "egui_component_style_macros":
            if publish:
                run_cmd("cargo publish", CWD / "style_macros")
            else:
                run_cmd("cargo publish --dry-run --allow-dirty", CWD / "style_macros")
        elif pkg == "egui_component" or pkg == "main":
            if publish:
                run_cmd("cargo publish", CWD)
            else:
                run_cmd("cargo publish --dry-run --allow-dirty", CWD)
        else:
            raise ValueError(f"Unknown package: {pkg}")

        logger.info("Done!")


if __name__ == "__main__":
    fire.Fire(Cli)
