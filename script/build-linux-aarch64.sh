#! /bin/bash

set -x
set -eo pipefail

CROSS_CONTAINER_OPTS="--env RSCC_VERSION=\"$1\"" cross build --target aarch64-unknown-linux-gnu --release

mkdir rscc-linux-aarch64
cp target/aarch64-unknown-linux-gnu/release/rscc rscc-linux-aarch64/
cp rsc.c rscc-linux-aarch64/
tar -czvf rscc-linux-aarch64.tar.gz rscc-linux-aarch64/
