#!/usr/bin/env bash

set -e

TARGET="x86_64-unknown-redox"
export CC="$TARGET-gcc"
if ! which "$CC" > /dev/null
then
    echo "Please run this script inside of a Redox cross-compiling environment"
    exit 1
fi

EXAMPLE="simple"
xargo build --target "$TARGET" --example "$EXAMPLE"

unset CC

export RUST_LOG=rine=debug
cargo run "target/$TARGET/debug/examples/$EXAMPLE"
