set windows-powershell := true

check:
    cargo check --no-default-features
    cargo check --no-default-features --features wav
    cargo check --no-default-features --features ogg
    cargo check --no-default-features --features flac
    cargo check --no-default-features --features mp3
    cargo check --all-features

test:
    cargo test --no-default-features
    cargo test --no-default-features --features wav
    cargo test --no-default-features --features ogg
    cargo test --no-default-features --features flac
    cargo test --no-default-features --features mp3
    cargo test --all-features

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy

ci:
    just fmt
    just clippy
    just check
    just test