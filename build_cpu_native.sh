#!/usr/bin/sh
set -e
RUSTFLAGS="-C target-cpu=native" cargo build --release