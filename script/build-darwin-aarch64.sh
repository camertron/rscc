#! /bin/bash

cross build --target=aarch64-apple-darwin --release

mkdir rscc-darwin-aarch64
cp target/x86_64-apple-darwin/release/rscc rscc-darwin-aarch64/
cp rsc.c rscc-darwin-aarch64/
tar -czvf rscc-darwin-aarch64.tar.gz rscc-darwin-aarch64/
