#!/bin/bash
# RexOS Test Runner Script
# Runs all tests with configurable options

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
COVERAGE_DIR="$PROJECT_ROOT/coverage"

log() {
    echo "[TEST] $1"
}

error() {
    echo "[ERROR] $1" >&2
    exit 1
}

# Run unit tests
run_unit_tests() {
    log "Running unit tests..."

    cargo test \
        --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        --all \
        --lib \
        -- --test-threads=4

    log "Unit tests complete"
}

# Run integration tests
run_integration_tests() {
    log "Running integration tests..."

    cargo test \
        --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        --all \
        --test '*' \
        -- --test-threads=2

    log "Integration tests complete"
}

# Run doc tests
run_doc_tests() {
    log "Running documentation tests..."

    cargo test \
        --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        --all \
        --doc

    log "Doc tests complete"
}

# Run ignored tests (hardware tests, etc.)
run_ignored_tests() {
    log "Running ignored tests..."

    cargo test \
        --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        --all \
        -- --ignored

    log "Ignored tests complete"
}

# Run tests with coverage
run_coverage() {
    log "Running tests with coverage..."

    if ! command -v cargo-llvm-cov &>/dev/null; then
        log "Installing cargo-llvm-cov..."
        cargo install cargo-llvm-cov
    fi

    mkdir -p "$COVERAGE_DIR"

    cargo llvm-cov \
        --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        --all-features \
        --workspace \
        --html \
        --output-dir "$COVERAGE_DIR"

    log "Coverage report: $COVERAGE_DIR/html/index.html"
}

# Run lints
run_lint() {
    log "Running lints..."

    cargo fmt \
        --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        --all \
        -- --check

    cargo clippy \
        --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        --all-targets \
        --all-features \
        -- -D warnings

    log "Lints complete"
}

# Run security audit
run_audit() {
    log "Running security audit..."

    if ! command -v cargo-audit &>/dev/null; then
        log "Installing cargo-audit..."
        cargo install cargo-audit
    fi

    cargo audit

    log "Security audit complete"
}

# Run shell script tests
run_shell_tests() {
    log "Running shell script tests..."

    # Check shell scripts with shellcheck if available
    if command -v shellcheck &>/dev/null; then
        find "$PROJECT_ROOT/scripts" -name "*.sh" -exec shellcheck {} \;
        log "Shellcheck passed"
    else
        log "Warning: shellcheck not installed, skipping shell script validation"
    fi

    # Basic syntax check
    find "$PROJECT_ROOT/scripts" -name "*.sh" -exec bash -n {} \;
    log "Shell syntax check passed"
}

# Run all tests
run_all() {
    run_lint
    run_unit_tests
    run_integration_tests
    run_doc_tests
    run_shell_tests

    log ""
    log "=========================================="
    log "All tests passed!"
    log "=========================================="
}

# Usage
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  all           Run all tests (default)"
    echo "  unit          Run unit tests only"
    echo "  integration   Run integration tests only"
    echo "  doc           Run doc tests only"
    echo "  ignored       Run ignored tests"
    echo "  coverage      Run tests with coverage report"
    echo "  lint          Run lints only"
    echo "  audit         Run security audit"
    echo "  shell         Run shell script tests"
    echo ""
    echo "Options:"
    echo "  --verbose     Show verbose output"
    echo "  --release     Run tests in release mode"
    echo ""
}

main() {
    local command="${1:-all}"

    case "$command" in
        all)
            run_all
            ;;
        unit)
            run_unit_tests
            ;;
        integration)
            run_integration_tests
            ;;
        doc)
            run_doc_tests
            ;;
        ignored)
            run_ignored_tests
            ;;
        coverage)
            run_coverage
            ;;
        lint)
            run_lint
            ;;
        audit)
            run_audit
            ;;
        shell)
            run_shell_tests
            ;;
        -h|--help|help)
            usage
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
