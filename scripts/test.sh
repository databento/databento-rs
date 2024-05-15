#! /usr/bin/env bash
set -e

cargo --version
cargo test --all-features
