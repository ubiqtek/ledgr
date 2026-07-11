# Common development commands for ledgr.
# Run `just` with no arguments to list them.

default:
    @just --list

build:
    cargo build

run:
    cargo run

test:
    cargo test

fmt:
    cargo fmt

lint:
    cargo clippy --all-targets --all-features -- -D warnings

check: fmt lint test
