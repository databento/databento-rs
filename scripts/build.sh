#! /usr/bin/env bash
cargo build --all-features
cargo build --no-default-features --features historical
cargo build --no-default-features --features live
