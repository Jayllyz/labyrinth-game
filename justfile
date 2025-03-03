# Install required development tools
install-tools:
    curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
    cargo binstall -y cargo-watch cargo-nextest

# Format code using rustfmt
format:
    cargo fmt --all

# Run clippy on all targets
clippy:
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings

# Run tests using nextest
test:
    cargo nextest run && cargo test --doc

# Run clippy and tests
check: clippy test

# Update changelog
changelog:
    npx --yes git-cliff@latest -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: update changelog"

# Default recipe
default:
    @just --list
