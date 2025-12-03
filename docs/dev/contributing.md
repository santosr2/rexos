# Contributing to RexOS

Thank you for your interest in contributing to RexOS! This guide will help you get started.

## Getting Started

### Prerequisites

1. **Rust Toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add aarch64-unknown-linux-gnu
   rustup target add armv7-unknown-linux-gnueabihf
   ```

2. **Cross-compilation Tools**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf

   # Or use cross
   cargo install cross
   ```

3. **Development Tools**
   ```bash
   rustup component add rustfmt clippy
   cargo install cargo-audit cargo-llvm-cov
   ```

### Building

```bash
# Clone the repository
git clone https://github.com/santosr2/rexos.git
cd rexos

# Build all components
cargo build

# Run tests
cargo test --all

# Format code
cargo fmt --all

# Run lints
cargo clippy --all -- -D warnings
```

## Development Workflow

### Branch Strategy

- `main` - Stable release branch
- `develop` - Integration branch for features
- `feature/*` - Feature branches
- `fix/*` - Bug fix branches
- `release/*` - Release preparation branches

### Creating a Pull Request

1. Fork the repository
2. Create a feature branch from `develop`
3. Make your changes
4. Ensure all tests pass
5. Format your code with `cargo fmt`
6. Run `cargo clippy` and fix any warnings
7. Submit a pull request to `develop`

### Commit Message Convention

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code formatting
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `test` - Adding tests
- `chore` - Maintenance tasks

**Examples:**
```
feat(hal): add support for RG353V analog sticks

Adds analog stick support for the RG353V device including
deadzone configuration and axis mapping.

Closes #123
```

```
fix(library): correct ROM extension detection

Fixes an issue where .sfc files were not recognized as
SNES ROMs.
```

## Code Style

### Rust Guidelines

1. **Use `rustfmt`** - All code must be formatted with `rustfmt`
2. **No warnings** - Code must compile without warnings
3. **Documentation** - Public APIs must be documented
4. **Tests** - New features should include tests

### Naming Conventions

```rust
// Modules: snake_case
mod game_library;

// Types: PascalCase
struct GameEntry { }
enum GameState { }

// Functions: snake_case
fn load_game() { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_GAMES: usize = 1000;

// Variables: snake_case
let game_count = 0;
```

### Error Handling

```rust
// Use thiserror for library errors
#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("game not found: {0}")]
    GameNotFound(String),
}

// Use anyhow for application errors
use anyhow::{Context, Result};

pub fn scan_games() -> Result<()> {
    library.scan()
        .context("Failed to scan game library")?;
    Ok(())
}
```

### Logging

```rust
use tracing::{info, warn, error, debug, trace};

// Use structured logging
info!(game_id = %id, "Loading game");
error!(?error, "Failed to load game");
debug!(path = ?path, "Scanning directory");
```

## Adding a New Feature

### 1. Create a New Module

```bash
# For a core module
cargo new --lib core/my-module
```

Update `Cargo.toml`:
```toml
[workspace]
members = [
    # ... existing members
    "core/my-module",
]
```

### 2. Define the Public API

```rust
// core/my-module/src/lib.rs

//! My Module - Description of what it does
//!
//! # Example
//!
//! ```rust
//! use rexos_my_module::MyFeature;
//!
//! let feature = MyFeature::new()?;
//! feature.do_something()?;
//! ```

mod error;
mod implementation;

pub use error::MyModuleError;
pub use implementation::MyFeature;
```

### 3. Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        let feature = MyFeature::new().unwrap();
        assert!(feature.is_ready());
    }
}
```

### 4. Update Documentation

- Add module documentation in the code
- Update `docs/dev/architecture.md` if needed
- Add usage examples to README.md

## Testing

### Unit Tests

```bash
# Run all tests
cargo test --all

# Run tests with output
cargo test --all -- --nocapture

# Run specific test
cargo test test_name
```

### Integration Tests

```bash
# Run integration tests (marked with #[ignore])
cargo test --all -- --ignored
```

### Code Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features
```

## Security

### Reporting Vulnerabilities

Please report security vulnerabilities privately to security@rexos.dev.

### Security Guidelines

1. Never store secrets in code
2. Validate all external input
3. Use safe Rust patterns
4. Run `cargo audit` regularly
5. Keep dependencies updated

## Getting Help

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - Questions and discussions
- **Discord** - Real-time chat (coming soon)

## Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- GitHub releases

Thank you for contributing to RexOS!
