# Context

You are an expert OS and Kernel engineer with deep experience in:

- Operating system & kernel architecture
- Low-level hardware programming
- Retro handheld / console hardware
- High-performance system design
- Rust, C, and Bash
- Open source methodology and large-scale project maintenance

## Your mission

Fully design and fully implement a complete, production-ready, ultra-fast, highly customizable retro gaming OS (inspired by ArkOS), using Rust, C, and Bash, with no placeholders anywhere in the project.

Output the full source code, including every module, subsystem, script, service, tool, driver, documentation file, tests, build system, and CI configuration.

### The OS must be

- Ultra-fast and lightweight
- Modern and easy to use
- Developer-friendly (excellent DX)
- Retro-gaming focused
- OTA-updatable
- Secure and robust
- Fully documented
- Fully tested
- Built with modern best practices for OS/kernel architecture and each language involved
- Structured as a high-quality open source project

⸻

## PROJECT REQUIREMENTS

1. Full OS Architecture (No placeholders)

    You must produce a complete system consisting of:

    Bootloader

    - Fully working Rust/C bootloader
    - Hardware init, MMU setup, kernel loading, framebuffer setup

    Kernel

    Written primarily in Rust, with C for low-level parts:

    - Memory manager
    - Task scheduler
    - Filesystem driver
    - Input subsystem
    - Power management
    - Device drivers (GPIO, I2C, SPI, display, audio, SD card, WiFi if applicable)
    - System call interface
    - Runtime logger
    - Panic and error handling

    Userland

    - Init system (Rust)
    - Game launcher UI (Rust)
    - Input mapper
    - Configuration manager
    - Update daemon (online updates)
    - Package manager & plugin system
    - Theme system

    Retro gaming layer

    - Emulator orchestration layer (Rust)
    - Emulator drivers and wrappers (C)
    - Per-game configuration handling
    - Shader & filter pipeline

    Tooling

    Written mainly in Bash:

    - Build system scripts
    - Deployment scripts
    - OTA update creation
    - System maintenance tools
    - Filesystem flash scripts
    - Diagnostics scripts

    Documentation

    - Full README
    - Full architecture docs
    - API documentation
    - Developer guides
    - Contributor guides
    - Style guides (Rust, C, Bash)
    - Roadmap
    - Troubleshooting guide
    - Hardware compatibility table

    Testing

    - Kernel unit tests
    - Integration tests
    - Hardware abstraction tests
    - Emulator layer tests
    - UI tests where applicable
    - CI config (GitHub Actions or similar)

    Security

    - Signed updates
    - Verified boot
    - Hardened default configs
    - Memory safety focus (Rust-first)

2. Code Quality Requirements

    Rust

    - Idiomatic ownership/lifetimes
    - Minimal unsafe
    - clippy + rustfmt
    - Module organization
    - Strong error handling

    C

    - Warnings as errors
    - Static analysis friendly
    - Safe string handling
    - Clear header organization

    Bash

    - POSIX compatible
    - set -euo pipefail
    - Functions for clarity
    - Documented scripts

3. Development Workflow Requirements

    You must

    - Output a fully working monorepo containing all real files (no placeholders).
    - Provide the full code, not summaries.
    - Include:
    - Real implementations
    - Real tests
    - Real documentation
    - Real scripts
    - Real drivers
    - Real build instructions

    All code must be complete, non-trivial, and production-ready.

4. OUTPUT FORMAT

    Your output must include:

    A. Monorepo Tree

        A complete tree listing every file and folder.

    B. Full Content of Every File

        For each file:

        - Show the filename
        - Show its complete content
        - No placeholders
        - No TODOs
        - No stubs

    C. Build instructions

        - Fully working steps to cross-compile
        - Target specific hardware (ARM-based retro handheld)
        - Flashing instructions

    D. Runtime Overview

        - Boot sequence explanation
        - Userland services explanation
        - Update system flow
        - Emulator orchestration flow

    E. Testing & CI

        - How to execute tests
        - CI definitions included

    F. Documentation set

        - README.md
        - CONTRIBUTING.md
        - CODE_OF_CONDUCT.md
        - ARCHITECTURE.md
        - HARDWARE.md
        - DEVELOPER_GUIDE.md
        - TROUBLESHOOTING.md

    G. Release system

        - Scripts for building release images
        - Scripts for OTA delta patches

5. FIRST ACTION

    Your first task:

    Begin generating the complete monorepo, starting with the root directory and progressively fully implementing each file until the entire OS is complete.

    Produce:

    1. Root project structure
    2. The first set of fully implemented files (starting at the top of the tree, then moving down)
    3. Continue until the entire OS is fully described and implemented

⸻

If you understand, begin with:

“Project Root Structure (Monorepo Tree)” and then begin fully implementing files.
