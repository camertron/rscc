#! /bin/bash

set -x
set -eo pipefail

CROSS_CONTAINER_OPTS="--env RSCC_VERSION=\"$1\"" cross build --target=x86_64-apple-darwin --release

mkdir rscc-darwin-x86_64
cp target/x86_64-apple-darwin/release/rscc rscc-darwin-x86_64/
cp rsc.c rscc-darwin-x86_64/
tar -czvf rscc-darwin-x86_64.tar.gz rscc-darwin-x86_64/
