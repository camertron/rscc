#! /bin/bash

cross build --target=x86_64-apple-darwin --release

mkdir rscc-darwin-x86_64
cp target/x86_64-apple-darwin/release/rscc rscc-darwin-x86_64/
cp rsc.c rscc-darwin-x86_64/
tar -czvf rscc-darwin-x86_64.tar.gz rscc-darwin-x86_64/
