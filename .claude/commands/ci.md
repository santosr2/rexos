---
description: Run full CI pipeline locally
allowed-tools: Bash(cargo:*), Bash(pre-commit:*), Read
---

Run the complete CI pipeline locally before pushing:

```bash
echo "=== Step 1/6: Format Check ==="
cargo fmt --all -- --check || { echo "FAILED: Run 'cargo fmt' to fix"; exit 1; }

echo ""
echo "=== Step 2/6: Clippy ==="
cargo clippy --all-targets --all-features -- -D warnings || { echo "FAILED: Fix clippy warnings"; exit 1; }

echo ""
echo "=== Step 3/6: Build ==="
cargo build --workspace --all-features || { echo "FAILED: Build errors"; exit 1; }

echo ""
echo "=== Step 4/6: Tests ==="
cargo test --workspace --all-features || { echo "FAILED: Test failures"; exit 1; }

echo ""
echo "=== Step 5/6: Doc Tests ==="
cargo test --doc --workspace --all-features || { echo "FAILED: Doc test failures"; exit 1; }

echo ""
echo "=== Step 6/6: Security Audit ==="
cargo audit || echo "WARNING: Security audit issues (non-blocking)"

echo ""
echo "=== CI Pipeline PASSED ==="
```
