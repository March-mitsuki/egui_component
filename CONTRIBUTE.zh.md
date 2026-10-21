# 贡献指南

[English](CONTRIBUTE.md) | [中文](CONTRIBUTE.zh.md)

欢迎为 egui_component 提交错误修复、组件、文档、翻译和可复现的问题报告。
较大的 API 调整或新功能，请先通过 issue 讨论并确认范围。

## 分支与合并规则

所有修改遵循 `功能分支 → dev → main` 的流程。

- `main` 是稳定集成与发布的基础分支，要求线性历史（linear history）。
- `dev` 是集中集成和验证下一轮修改的共享分支，允许 merge commit，
  不应对它启用线性历史要求。
- 功能、修复和文档分支均从 `dev` 创建，PR 的目标是 `dev`，而非 `main`。
  外部贡献者请通过 fork 提交。

维护者通常使用 **Squash and merge** 将功能 PR 合入 `dev`，每个 PR 对应一个提交。
累计修改验证完成后，创建 `dev → main` 的 PR，同样使用 **Squash and merge**。
合并后保留 `dev` 分支。

每次 `dev → main` 合并完成后，由维护者将 `main` 普通合并回 `dev`：

```sh
git switch dev
git fetch origin
git merge origin/main
git push origin dev
```

执行前确保工作区干净、本地 `dev` 已同步远端。这里假设 `origin` 指向官方仓库。
出现冲突时先解决冲突、检查结果并重新运行相关检查，再推送。
如果 `dev` 禁止直接推送，通过允许 merge commit 的同步 PR 完成这一步。

这一步让 `main` 上的新提交成为 `dev` 的祖先。Squash 会创建新提交，
仅仅文件内容相同并不代表两条分支已经建立了这一历史关系。
后续 PR 的提交列表仍可能包含旧提交，判断本轮发布内容时应查看文件差异。
`dev` 中存在 merge commit 不影响 `main` 保持线性历史。
不要在日常协作中重置或强制推送共享的 `dev`，也不要使用
`git merge -s ours` 同步，因为它会丢弃另一侧的文件改动。

已有进行中分支的贡献者，应将更新后的 `dev` 合入自己的分支，解决冲突后再更新 PR。
发布与上传软件包由维护者处理，普通贡献者不需要运行 `publish.py`。

## 开发环境

安装较新的稳定版 Rust 工具链和 Cargo，需支持 edition 2024。
如果系统需要，请安装 egui/eframe 所需的原生平台依赖。

通过 fork 贡献时，克隆自己的仓库并将官方仓库添加为 `upstream`：

```sh
git clone https://github.com/YOUR_USERNAME/egui_component.git
cd egui_component
git remote add upstream https://github.com/March-mitsuki/egui_component.git
git fetch upstream
git switch -c my-change upstream/dev
```

仓库协作者可改为获取 `origin`，并从 `origin/dev` 创建分支。
分支名称应能说明用途，提交保持聚焦，避免无关的格式调整。

主要目录：

- `src/`：组件、样式、字体与库 API。
- `style_macros/`：样式过程宏。
- `router/`：路由库。
- `examples/gallery/`：原生/Web Gallery、文档与翻译。
- `examples/theme.json`：示例明暗主题。
- `tests/`：集成测试与发布工具测试。

在仓库根目录启动原生 Gallery：

```sh
cargo run --example gallery
```

启动浏览器版 Gallery：

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve
```

打开 <http://127.0.0.1:8080>。Gallery 下载字体时需要联网；
字体行为与更多示例见 [README](README.zh.md)。

## 检查与代码约定

在仓库根目录，根据修改范围执行相关检查：

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-features
cargo test --example gallery
```

Web 相关修改还需运行 `trunk build`，并在浏览器中验证受影响页面。
UI 修改应检查明暗主题、窄窗口和相关交互状态，并在 PR 中附截图或简短录屏。

遵循周围 Rust 代码的风格以及现有组件和样式约定。
行为修复应添加回归测试，新逻辑应有有意义的测试。
公共 API 或行为变化时同步更新示例和文档，并在 PR 中说明兼容性影响。
中英文文档应保持一致。修改 Gallery 文案时，更新所有支持的语言并保留占位符，
详见[翻译指南](examples/gallery/locales/README.md)。

仅在修改 Python 发布工具时需要 Python 环境。使用 Python 3.14+ 和
[uv](https://docs.astral.sh/uv/)，运行：

```sh
uv sync
uv run python -m unittest discover -s tests -p 'test_*.py'
```

更新 Lucide 图标时，将生成的图标资源与 Rust 索引一起提交；
更新脚本和上游许可证声明见 README。

## Issue 与 Pull Request

提交前先搜索已有 issue 和 PR。错误报告应包含 crate 版本或 commit、
操作系统与 Rust 版本、原生/Web 目标、复现步骤或最小示例，以及预期和实际行为。
提供相关日志时，请移除凭据和私人数据。功能请求应说明使用场景和期望行为。

提交 PR 前：

- 以 `dev` 为目标分支，每个 PR 聚焦一个完整目的。
- 说明解决的问题、最终行为，以及 API 或兼容性变化。
- 列出执行过的检查与结果；无法执行的检查请说明原因。
- 关联相关 issue，UI 修改附视觉效果材料。
- 更新相关测试、示例与中英文文档。

Fork 分支可通过 `git push -u origin my-change` 推送，随后创建以官方仓库
`dev` 为目标的 PR。通过追加提交回应评审；重写他人正在使用的分支前请先协调。

讨论时保持尊重，提供建设性反馈。贡献遵循仓库的 [MIT 许可证](LICENSE)。
请保留第三方署名，只提交你有权贡献的代码与资源。
