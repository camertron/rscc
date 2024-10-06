#! /bin/bash

set -x
set -eo pipefail

cross build --target x86_64-unknown-linux-gnu --release

mkdir rscc-linux-x86_64
cp target/x86_64-unknown-linux-gnu/release/rscc rscc-linux-x86_64/
cp rsc.c rscc-linux-x86_64/
tar -czvf rscc-linux-x86_64.tar.gz rscc-linux-x86_64/
