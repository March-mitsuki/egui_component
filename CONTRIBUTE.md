# Contributing

[English](CONTRIBUTE.md) | [中文](CONTRIBUTE.zh.md)

Contributions to egui_component are welcome: bug fixes, components, documentation,
translations, and reproducible bug reports. Discuss substantial API changes or new
features in an issue before implementation so the scope can be agreed upon.

## Branches and merge policy

Changes flow through `feature branch → dev → main`.

- `main` is the stable integration and release base. It requires linear history.
- `dev` is the shared integration branch for testing upcoming changes. Merge
  commits are allowed here; do not apply the linear-history requirement to `dev`.
- Feature, fix, and documentation branches start from `dev` and open PRs against
  `dev`, not `main`. External contributors should use a fork.

Maintainers normally **Squash and merge** feature PRs into `dev`, giving each PR
one commit. Once the accumulated changes have been verified, open a `dev → main`
PR and **Squash and merge** it. Keep `dev` after merging.

After every `dev → main` merge, a maintainer merges `main` back into `dev`:

```sh
git switch dev
git fetch origin
git merge origin/main
git push origin dev
```

Start with a clean working tree and an up-to-date local `dev`. These commands
assume `origin` is the canonical repository. Resolve any conflicts, review the
result, and rerun relevant checks before pushing. If direct pushes to `dev` are
restricted, submit the synchronization through a PR that permits a merge commit.

This synchronization records the new `main` commit in `dev`'s ancestry. Squash
merges create new commits, so file contents alone do not establish that ancestry.
Old commits can still appear in later PR commit lists; review the file diff for
the actual release changes. `main` stays linear even though `dev` has merge
commits. Do not routinely reset or force-push shared `dev`, and do not use
`git merge -s ours` for synchronization: it discards changes from the other side.

Contributors with ongoing branches should merge the updated `dev` into their
branch and resolve conflicts before the next PR update. A maintainer handles
releases and publishing; contributors do not need to run `publish.py`.

## Development setup

Install a current stable Rust toolchain with Cargo (edition 2024 support is
required). Install the native platform libraries needed by egui/eframe when your
OS requires them.

For a fork, clone your fork and add the canonical repository as `upstream`:

```sh
git clone https://github.com/YOUR_USERNAME/egui_component.git
cd egui_component
git remote add upstream https://github.com/March-mitsuki/egui_component.git
git fetch upstream
git switch -c my-change upstream/dev
```

Repository collaborators can instead fetch `origin` and create their branch from
`origin/dev`. Use a descriptive branch name, keep commits focused, and avoid
unrelated formatting changes.

Useful locations:

- `src/`: components, styling, fonts, and library APIs.
- `style_macros/`: styling procedural macros.
- `router/`: routing library.
- `examples/gallery/`: native/Web gallery, documentation, and translations.
- `examples/theme.json`: example light and dark themes.
- `tests/`: integration tests and publishing-tool tests.

Run the native gallery from the repository root:

```sh
cargo run --example gallery
```

For the browser gallery:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve
```

Open <http://127.0.0.1:8080>. Gallery font downloads need network access; see the
[README](README.md) for font behavior and more examples.

## Checks and coding expectations

Run checks appropriate to the change from the repository root:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-features
cargo test --example gallery
```

For Web changes, also run `trunk build` and exercise the affected page in a
browser. For UI changes, check light/dark themes, narrow windows, and relevant
interaction states; include screenshots or a short recording in the PR.

Follow surrounding Rust style and existing component/styling conventions. Add
regression tests for behavior fixes and meaningful tests for new logic. Update
examples and documentation when public APIs or behavior change, and explain
compatibility breaks in the PR. Keep English and Chinese documentation aligned.
For gallery strings, update all supported locales and preserve placeholders;
see the [translation guide](examples/gallery/locales/README.md).

Only changes to Python publishing tools require Python tooling. Use Python 3.14+
and [uv](https://docs.astral.sh/uv/), then run:

```sh
uv sync
uv run python -m unittest discover -s tests -p 'test_*.py'
```

Commit generated Lucide assets and their Rust index together when updating icons;
see the README for the update script and upstream license notices.

## Issues and pull requests

Search existing issues and PRs first. A bug report should include the crate
version or commit, OS and Rust version, native/Web target, reproduction steps or
a minimal example, and expected versus actual behavior. Include relevant logs
without credentials or private data. Feature requests should describe the use
case and proposed behavior.

Before opening a PR:

- Target `dev` and keep one coherent purpose per PR.
- Explain the problem, resulting behavior, and any API or compatibility changes.
- List checks performed and their results; explain checks you could not run.
- Link related issues and include visual evidence for UI changes.
- Update relevant tests, examples, and both documentation languages.

For a fork, push with `git push -u origin my-change`, then open a PR with the
canonical repository's `dev` as the base. Respond to review with additional
commits; coordinate before rewriting a branch that others use.

Keep discussions respectful and constructive. Contributions are made under the
repository's [MIT license](LICENSE). Preserve third-party attribution and only
include code and assets you have permission to contribute.
