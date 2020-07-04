#!/usr/bin/env bash

if [ $TRAVIS_RUST_VERSION == "nightly" ]; then
    env RUSTFLAGS="-Z sanitizer=address" cargo +nightly test
else
    cargo test
fi

# test again, but with MSVC (GNU is by default on Windows)
if [ $TRAVIS_OS_NAME == "windows" ]; then
    rustup default stable-x86_64-pc-windows-msvc
    cargo test
fi
