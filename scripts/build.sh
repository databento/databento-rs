#! /usr/bin/env bash
set -e

cargo --version
echo build all
cargo build --all-features
echo build historical
cargo build --no-default-features --features historical
echo build live
cargo build --no-default-features --features live
echo build examples
cargo build --examples
