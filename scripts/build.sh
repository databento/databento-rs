#! /usr/bin/env bash
echo build all
cargo build --all-features
echo build historical
cargo build --no-default-features --features historical
echo build live
cargo build --no-default-features --features live
