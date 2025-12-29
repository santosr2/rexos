---
description: Create a new RexOS crate in the workspace
allowed-tools: Bash(cargo:*), Write, Read, Edit
argument-hint: <crate-name>
---

Create a new crate in the RexOS workspace following project conventions.

The crate name should follow the pattern: `rexos-<name>`

Steps:
1. Create the crate with cargo
2. Update root Cargo.toml workspace members
3. Set up standard dependencies
4. Create initial module structure

```bash
CRATE_NAME="$1"
if [ -z "$CRATE_NAME" ]; then
  echo "Usage: /new-crate <crate-name>"
  echo "Example: /new-crate rexos-input"
  exit 1
fi

echo "Creating crate: $CRATE_NAME in crates/"
cargo new --lib "crates/$CRATE_NAME" 2>&1
```

After running, I will:
1. Add the crate to workspace members in root Cargo.toml
2. Update the new crate's Cargo.toml with workspace dependencies
3. Set up proper module structure with lib.rs
