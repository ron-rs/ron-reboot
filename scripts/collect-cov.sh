#!/bin/sh

set -e

if ! cargo install --list | grep -q "grcov"; then
  echo "Installing grcvo because it was not found..."
  cargo install grcov
fi

cargo clean
CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile" cargo +nightly build --all-features
CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile" cargo +nightly test --all-features
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
