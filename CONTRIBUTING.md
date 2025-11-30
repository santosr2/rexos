# Contributing to RexOS

Thank you for considering contributing to RexOS! We welcome contributions from everyone.

## Code of Conduct

- Be respectful and inclusive
- Help create a welcoming environment
- Accept constructive criticism gracefully
- Focus on what's best for the community

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues. When creating a bug report, include:

- **Description**: Clear description of the bug
- **Steps to Reproduce**: Step-by-step instructions
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happens
- **Environment**: Device model, OS version, etc.
- **Logs**: Relevant log output if available

### Suggesting Enhancements

Enhancement suggestions are welcome! Please include:

- **Use case**: Why is this enhancement useful?
- **Description**: Detailed description of the feature
- **Mockups**: UI mockups if applicable
- **Alternatives**: Alternative solutions considered

### Pull Requests

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Make** your changes
4. **Test** your changes thoroughly
5. **Format** your code: `cargo fmt`
6. **Lint** your code: `cargo clippy`
7. **Commit** with clear messages: `git commit -am 'Add amazing feature'`
8. **Push** to your fork: `git push origin feature/amazing-feature`
9. **Open** a Pull Request

### Commit Message Guidelines

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding tests
- `chore`: Maintenance tasks

**Example:**
```
feat(hal): add support for RG353V analog sticks

Implement analog stick reading for RG353V using GPIO.
Includes calibration and dead zone support.

Closes #123
```

## Development Setup

See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for detailed setup instructions.

## Project Structure

```
rexos/
├── core/           # Core system components (Rust)
├── services/       # System services (Rust)
├── scripts/        # Shell scripts
├── docs/           # Documentation
└── tests/          # Tests
```

## Coding Standards

### Rust Code

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Add documentation comments (`///`) for public APIs
- Write unit tests for new functionality
- Keep functions small and focused

### Shell Scripts

- Use shellcheck for validation
- Add comments for complex logic
- Use functions for reusable code
- Handle errors explicitly
- Test on target devices

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run on specific target
cross test --target aarch64-unknown-linux-gnu
```

### Hardware Testing

- Test on actual devices when possible
- Document device-specific behavior
- Include device model in test reports

## Documentation

- Update README.md for user-facing changes
- Update docs/ for technical changes
- Add inline comments for complex code
- Update CHANGELOG.md (if applicable)

## Areas Needing Help

We especially welcome contributions in:

- **Hardware Support**: Adding new device support
- **Emulator Integration**: Optimizing emulators
- **UI/UX**: Frontend improvements
- **Documentation**: Guides and tutorials
- **Testing**: Hardware testing on various devices
- **Performance**: Optimization work
- **Localization**: Translations

## Questions?

- Open a discussion on GitHub
- Join our Discord/Matrix (links coming soon)
- Email: (coming soon)

## Attribution

Contributors will be recognized in:
- README.md Contributors section
- Release notes
- About page in the UI

Thank you for contributing! 🎮
