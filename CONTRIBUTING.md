# Contributing to Canary

Thank you for your interest in contributing to Canary! This document provides guidelines and instructions for contributing.

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Keep discussions technical and professional

## How to Contribute

### 1. Reporting Issues

Before creating an issue:
- Search existing issues to avoid duplicates
- Provide clear reproduction steps
- Include relevant system information
- Attach error messages and logs

### 2. Suggesting Features

When suggesting features:
- Explain the use case and benefits
- Consider existing patterns in the codebase
- Provide examples if possible
- Be open to discussion and alternatives

### 3. Submitting Code

#### Fork and Clone

```bash
git clone https://github.com/your-username/canary.git
cd canary
```

#### Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

#### Make Changes

Follow these guidelines:

**Code Style:**
- Use declarative Rust patterns (`.filter()`, `.map()`, `.collect()`)
- Avoid imperative loops and mutations
- Prefer pattern matching over if-else chains
- Keep functions small and focused
- Use meaningful variable names

**SOLID Principles:**
- Single Responsibility: Each module does one thing
- Open/Closed: Extend via traits, not modification
- Liskov Substitution: Implementations must be interchangeable
- Interface Segregation: Small, focused traits
- Dependency Inversion: Depend on abstractions

**Rust Patterns:**
- Use Newtype pattern for type safety
- Apply Builder pattern for complex construction
- Use RAII for resource management
- Apply Strategy pattern via traits
- Keep crates small and focused

#### Adding Data

**New Pinouts:**
```bash
# Add TOML file
crates/canary-data/data/pinouts/manufacturer_name/vehicle_model.toml

# Update loader in crates/canary-data/src/lib.rs
static NEW_PINOUT_TOML: &str = include_str!("../data/pinouts/...");

# Add to PINOUTS HashMap
```

**New DTC Codes:**
```bash
# Edit existing or create new TOML
crates/canary-data/data/dtc/system_name.toml

# Add codes following existing format
[[dtc_codes]]
code = "P0123"
system = "Powertrain"
description = "..."
```

**New Service Procedures:**
```bash
# Create TOML file
crates/canary-data/data/service_procedures/procedure_name.toml

# Follow existing format with steps and warnings
```

**New Protocols:**
```rust
// Implement ProtocolDecoder trait
pub struct NewProtocolDecoder {
    spec: &'static ProtocolSpec,
}

impl ProtocolDecoder for NewProtocolDecoder {
    type Frame = NewFrame;
    fn decode(&self, raw: &[u8]) -> Result<Self::Frame> { /* ... */ }
    fn encode(&self, frame: &Self::Frame) -> Result<Vec<u8>> { /* ... */ }
}
```

#### Testing

**Run tests before committing:**

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p canary-dtc

# With output
cargo test --workspace -- --nocapture

# Doc tests
cargo test --doc
```

**Write tests for new features:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Arrange
        let input = /* ... */;

        // Act
        let result = your_function(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

**Test coverage requirements:**
- All public APIs must have tests
- Edge cases should be covered
- Error conditions should be tested
- Encode/decode symmetry for protocols

#### Documentation

**Add rustdoc comments:**

```rust
/// Brief description of what this does
///
/// # Examples
///
/// ```
/// use canary_core::YourType;
///
/// let result = YourType::new();
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns `CanaryError::NotFound` if the item doesn't exist
pub fn your_function() -> Result<()> {
    // ...
}
```

**Update documentation files:**
- `README.md` for major features
- `FEATURES.md` for detailed feature docs
- `PROJECT_SUMMARY.md` for implementation details

#### Build and Verify

```bash
# Clean build
cargo clean
cargo build --workspace

# Release build
cargo build --workspace --release

# Check for warnings
cargo clippy --workspace

# Format code
cargo fmt --all

# Run examples
cargo run --example basic_usage
cargo run --example dtc_analysis
```

#### Commit Messages

Follow conventional commits format:

```
type(scope): brief description

Longer description if needed

- Additional details
- Breaking changes noted

Fixes #123
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

**Examples:**
```
feat(dtc): add body system DTC codes

Added 15 new body system (B-codes) diagnostic codes
covering door, window, and seat systems.

- B0001-B0015 added to dtc/body.toml
- Tests updated
- Documentation updated
```

```
fix(protocol): correct CAN frame length validation

Fixed issue where frames shorter than 4 bytes caused
panic instead of returning proper error.

Fixes #42
```

### 4. Submit Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a PR on GitHub with:
- Clear title describing the change
- Description of what changed and why
- Link to related issues
- Screenshots/examples if applicable

#### PR Checklist

- [ ] Tests pass (`cargo test --workspace`)
- [ ] Code formatted (`cargo fmt --all`)
- [ ] No clippy warnings (`cargo clippy --workspace`)
- [ ] Documentation updated
- [ ] Examples still work
- [ ] Commit messages follow convention
- [ ] Breaking changes noted

## Development Setup

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# PostgreSQL (optional, for testing migrations)
brew install postgresql

# SQLite (optional, for testing)
brew install sqlite
```

### Environment Setup

```bash
# Clone repository
git clone https://github.com/your-username/canary.git
cd canary

# Build
cargo build --workspace

# Run tests
cargo test --workspace

# Run example
cargo run --example basic_usage
```

### Database Testing

```bash
# SQLite
export DATABASE_URL="sqlite://test.db"
./run_migration.sh

# PostgreSQL
export DATABASE_URL="postgresql://localhost/canary_test"
./run_migration.sh
```

## Project Structure

```
canary/
├── crates/
│   ├── canary-core/         # Main library facade
│   ├── canary-models/       # Data structures
│   ├── canary-database/     # DB connection
│   ├── canary-data/         # Embedded data
│   ├── canary-pinout/       # Pin mapping
│   ├── canary-protocol/     # Protocol decoders
│   ├── canary-dtc/          # DTC database
│   └── canary-service-proc/ # Service procedures
├── migration/               # Database migrations
├── examples/                # Usage examples
└── docs/                    # Additional documentation
```

## Common Tasks

### Adding a New DTC Code

1. Edit `crates/canary-data/data/dtc/system.toml`
2. Add code entry following format
3. Run `cargo test -p canary-data`
4. Update `FEATURES.md` with new count

### Adding a New Service Procedure

1. Create `crates/canary-data/data/service_procedures/name.toml`
2. Define steps with order, instruction, warnings
3. Update `crates/canary-data/src/lib.rs` to include it
4. Add test in `canary-service-proc`
5. Update examples if relevant

### Adding a New Protocol Decoder

1. Define protocol spec in `crates/canary-data/data/protocols/`
2. Create decoder in `crates/canary-protocol/src/`
3. Implement `ProtocolDecoder` trait
4. Add to `ProtocolFactory`
5. Write encode/decode tests
6. Add example usage

### Running Benchmarks

```bash
# Coming soon - performance benchmarks
cargo bench
```

## Getting Help

- **Issues**: Report bugs or ask questions
- **Discussions**: General discussion about the project
- **Discord**: [Coming soon]

## License

By contributing, you agree that your contributions will be licensed under both MIT and Apache-2.0 licenses.

---

Thank you for contributing to Canary! 🐤
