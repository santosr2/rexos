# RexOS Development Best Practices

This document outlines the tools, configurations, and development practices RexOS should adopt, inspired by well-established open source projects.

## Reference Projects Analyzed

| Project | Language | What We Learned |
|---------|----------|-----------------|
| [mise](https://github.com/jdx/mise) | Rust | Task automation, mise.toml, CI/CD, cargo-deny |
| [uv](https://github.com/astral-sh/uv) | Rust | Workspace structure, build profiles, strict linting |
| [ripgrep](https://github.com/BurntSushi/ripgrep) | Rust | Feature flags, LTO profiles, workspace organization |
| [bat](https://github.com/sharkdp/bat) | Rust | Optional features, MSRV policy, build scripts |
| [fish-shell](https://github.com/fish-shell/fish-shell) | Rust | Edition 2024, overflow checks, custom lints |
| [ruff](https://github.com/astral-sh/ruff) | Rust+Python | Changelog automation, code quality enforcement |
| [curl](https://github.com/curl/curl) | C | CMake best practices, feature toggles, testing |

---

## Rust Configuration

### Cargo.toml Workspace Setup

Based on mise, uv, and ripgrep patterns:

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "MIT OR Apache-2.0"
repository = "https://github.com/user/rexos"
authors = ["RexOS Contributors"]

[workspace.lints.rust]
unsafe_code = "warn"
# Deny in CI, warn in development
# unsafe_code = "deny"

[workspace.lints.clippy]
# Pedantic but practical
pedantic = { level = "warn", priority = -1 }
# Allow common patterns
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
# Deny bad practices
print_stdout = "deny"
print_stderr = "deny"
dbg_macro = "deny"
todo = "warn"
unimplemented = "warn"

[workspace.dependencies]
# Pin all dependencies here for consistency
```

### Build Profiles

From ripgrep, bat, and uv:

```toml
[profile.dev]
# Keep overflow checks for catching bugs
overflow-checks = true
# Speed up compilation of dependencies
[profile.dev.package."*"]
opt-level = 2

[profile.release]
# Standard release with some debug info
debug = 1
overflow-checks = false

[profile.release-lto]
inherits = "release"
lto = "fat"
strip = true
codegen-units = 1
panic = "abort"

[profile.profiling]
inherits = "release"
debug = true
strip = false
```

### Feature Flags

From ripgrep and bat:

```toml
[features]
default = ["application"]
application = ["input", "display", "audio", "storage"]
minimal = []  # Embedded/constrained environments

# Optional hardware support
gpu = ["dep:drm", "dep:gbm"]
bluetooth = ["dep:bluer"]

# Development features
dev = ["tracing/max_level_trace"]
```

---

## Development Tools (mise.toml)

Based on mise's own configuration:

```toml
[tools]
# Rust ecosystem
rust = "1.85"
cargo-binstall = "latest"
cargo-deny = "latest"
cargo-machete = "latest"
cargo-nextest = "latest"
cargo-insta = "latest"
cargo-release = "latest"
cargo-audit = "latest"
taplo = "latest"

# Git & versioning
git-cliff = "latest"
gh = "latest"

# Code quality
pre-commit = "latest"
actionlint = "latest"

# Shell scripts
shellcheck = "latest"
shfmt = "latest"

# General utilities
jq = "latest"
ripgrep = "latest"

# C/C++ (for FFI)
[tools.cmake]
version = "latest"
[tools.ninja]
version = "latest"

[env]
RUST_BACKTRACE = "1"
CARGO_INCREMENTAL = "1"

[settings]
experimental = true
```

---

## Task Automation (tasks.toml)

Based on mise's task patterns:

```toml
[tasks.build]
alias = "b"
description = "Build all crates"
run = "cargo build --all-features"

[tasks.test]
alias = "t"
description = "Run all tests"
depends = ["test:unit", "test:integration"]

[tasks."test:unit"]
description = "Run unit tests"
run = "cargo nextest run --all-features"

[tasks."test:integration"]
description = "Run integration tests"
run = "cargo test --test '*' --all-features"

[tasks.lint]
description = "Run all linters"
depends = ["lint:rust", "lint:shell", "lint:toml"]

[tasks."lint:rust"]
description = "Lint Rust code"
run = """
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --check
"""

[tasks."lint:shell"]
description = "Lint shell scripts"
run = """
shellcheck scripts/**/*.sh
shfmt -d scripts/
"""

[tasks."lint:toml"]
description = "Lint TOML files"
run = "taplo check"

[tasks.fmt]
description = "Format all code"
run = """
cargo fmt
shfmt -w scripts/
taplo fmt
"""

[tasks.check]
description = "Quick compile check"
run = "cargo check --all-features --all-targets"

[tasks.deny]
description = "Check dependencies for issues"
run = "cargo deny check"

[tasks.audit]
description = "Security audit dependencies"
run = "cargo audit"

[tasks.machete]
description = "Find unused dependencies"
run = "cargo machete"

[tasks.clean]
description = "Clean build artifacts"
run = "cargo clean"

[tasks.ci]
description = "Run full CI pipeline locally"
depends = ["fmt", "lint", "deny", "test", "build"]

[tasks.release]
description = "Create a release"
run = "cargo release"

[tasks.changelog]
description = "Generate changelog"
run = "git-cliff -o CHANGELOG.md"

[tasks.coverage]
description = "Generate code coverage"
run = """
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
cargo llvm-cov report --html
"""

[tasks.bench]
description = "Run benchmarks"
run = "cargo bench"

[tasks.doc]
description = "Generate documentation"
run = "cargo doc --all-features --no-deps --open"

[tasks.install-dev]
description = "Install development binary"
run = "cargo install --path crates/rexos-launcher --debug"

[tasks.cross]
description = "Cross-compile for ARM64"
run = "cross build --target aarch64-unknown-linux-gnu --release"
```

---

## Pre-commit Configuration

Based on mise's setup:

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v5.0.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-toml
      - id: check-json
      - id: check-merge-conflict
      - id: check-added-large-files
        args: ['--maxkb=1000']

  - repo: https://github.com/koalaman/shellcheck-precommit
    rev: v0.10.0
    hooks:
      - id: shellcheck
        args: ['--severity=warning']

  - repo: https://github.com/scop/pre-commit-shfmt
    rev: v3.8.0-1
    hooks:
      - id: shfmt
        args: ['-i', '2', '-ci']

  - repo: https://github.com/rhysd/actionlint
    rev: v1.7.4
    hooks:
      - id: actionlint

  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-check
        name: cargo check
        entry: cargo check --all-features
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-features -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      - id: taplo-fmt
        name: taplo format
        entry: taplo fmt
        language: system
        types: [toml]
        pass_filenames: false
```

---

## Cargo Deny Configuration

Based on mise's deny.toml:

```toml
# deny.toml
[graph]
all-features = true

[advisories]
db-path = "~/.cargo/advisory-db"
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"

[licenses]
unlicensed = "deny"
confidence-threshold = 0.8
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "BSL-1.0",
    "ISC",
    "MPL-2.0",
    "Zlib",
    "Unicode-3.0",
]

[[licenses.clarify]]
crate = "ring"
expression = "MIT AND ISC AND OpenSSL"
license-files = [{ path = "LICENSE", hash = 0xbd0eed23 }]

[bans]
multiple-versions = "warn"
wildcards = "deny"
highlight = "all"
skip = []
skip-tree = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-git = []
```

---

## CI/CD Configuration

Based on mise, uv, and ripgrep workflows:

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --all-features --all-targets

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-features --all-targets -- -D warnings

  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run --all-features

  deny:
    name: Cargo Deny
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-llvm-cov
      - run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: true

  msrv:
    name: MSRV
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.85"  # Match rust-version in Cargo.toml
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --all-features

  cross:
    name: Cross Compile
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - aarch64-unknown-linux-gnu
          - armv7-unknown-linux-gnueabihf
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cross
      - run: cross build --target ${{ matrix.target }} --release
```

---

## Shell Script Standards

Based on shellcheck and shfmt best practices:

```bash
#!/usr/bin/env bash
# scripts/example.sh

# Strict mode
set -euo pipefail
IFS=$'\n\t'

# Constants
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Logging functions
log_info() { echo "[INFO] $*" >&2; }
log_warn() { echo "[WARN] $*" >&2; }
log_error() { echo "[ERROR] $*" >&2; }
die() { log_error "$*"; exit 1; }

# Dependency checks
require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"
}

# Main function
main() {
  require_cmd cargo
  log_info "Starting build..."
  # ...
}

main "$@"
```

---

## C/FFI Best Practices

Based on curl and Redis patterns:

### CMakeLists.txt for FFI Bridge

```cmake
cmake_minimum_required(VERSION 3.20)
project(rexos-ffi C)

# Compiler settings
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Warning flags
if(CMAKE_C_COMPILER_ID MATCHES "GNU|Clang")
    add_compile_options(
        -Wall -Wextra -Wpedantic
        -Wformat=2 -Wformat-security
        -Wconversion -Wsign-conversion
        -Wnull-dereference
        -fstack-protector-strong
    )
endif()

# Build type defaults
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Source files
file(GLOB_RECURSE SOURCES "src/*.c")
file(GLOB_RECURSE HEADERS "include/*.h")

# Library target
add_library(rexos_ffi STATIC ${SOURCES})
target_include_directories(rexos_ffi PUBLIC include)

# Testing
option(BUILD_TESTING "Build tests" ON)
if(BUILD_TESTING)
    enable_testing()
    add_subdirectory(tests)
endif()
```

---

## Documentation Standards

### README.md Template

```markdown
# RexOS

[![CI](https://github.com/user/rexos/workflows/CI/badge.svg)](https://github.com/user/rexos/actions)
[![codecov](https://codecov.io/gh/user/rexos/branch/main/graph/badge.svg)](https://codecov.io/gh/user/rexos)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

Brief description of the project.

## Features

- Feature 1
- Feature 2

## Installation

```bash
# Instructions
```

## Usage

```bash
# Examples
```

## Building from Source

```bash
git clone https://github.com/user/rexos
cd rexos
mise install
mise run build
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
```

### Changelog (git-cliff)

```toml
# cliff.toml
[changelog]
header = "# Changelog\n\n"
body = """
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
- {{ commit.message | upper_first }} ({{ commit.id | truncate(length=7, end="") }})\
{% endfor %}
{% endfor %}
"""
footer = ""
trim = true

[git]
conventional_commits = true
filter_unconventional = true
commit_parsers = [
    { message = "^feat", group = "Features" },
    { message = "^fix", group = "Bug Fixes" },
    { message = "^perf", group = "Performance" },
    { message = "^doc", group = "Documentation" },
    { message = "^refactor", group = "Refactor" },
    { message = "^style", group = "Style" },
    { message = "^test", group = "Testing" },
    { message = "^chore", group = "Miscellaneous" },
]
filter_commits = false
```

---

## Summary: Tools to Adopt

| Category | Tool | Purpose |
|----------|------|---------|
| **Testing** | cargo-nextest | Faster test runner with better output |
| **Snapshots** | cargo-insta | Snapshot testing for complex outputs |
| **Security** | cargo-deny | License and vulnerability checking |
| **Security** | cargo-audit | Security advisory scanning |
| **Unused** | cargo-machete | Find unused dependencies |
| **Coverage** | cargo-llvm-cov | Code coverage reports |
| **Release** | cargo-release | Automated version bumping and publishing |
| **Changelog** | git-cliff | Conventional commit changelog generation |
| **Formatting** | taplo | TOML formatting and validation |
| **Shell** | shellcheck + shfmt | Shell script linting and formatting |
| **CI** | actionlint | GitHub Actions workflow validation |
| **Hooks** | pre-commit | Git hook automation |

---

## Migration Checklist

- [ ] Update Cargo.toml with workspace lints from uv/mise
- [ ] Add build profiles (release-lto, profiling)
- [ ] Create deny.toml for dependency auditing
- [ ] Set up .pre-commit-config.yaml
- [ ] Create tasks.toml with mise tasks
- [ ] Add GitHub Actions CI workflow
- [ ] Configure git-cliff for changelogs
- [ ] Add shellcheck/shfmt for shell scripts
- [ ] Set up code coverage with codecov
- [ ] Add MSRV verification in CI
