#! /bin/bash

set -x
set -eo pipefail

if [ ! -d vendor/cross ]; then
    git clone https://github.com/cross-rs/cross.git vendor/cross
fi

cd vendor/cross
git submodule update --init --remote

cargo \
    build-docker-image x86_64-apple-darwin-cross \
    --tag latest \
    --build-arg 'MACOS_SDK_URL=https://github.com/joseluisq/macosx-sdks/releases/download/12.3/MacOSX12.3.sdk.tar.xz'

docker image tag \
    ghcr.io/cross-rs/x86_64-apple-darwin-cross:latest \
    ghcr.io/camertron/rscc-darwin-x64-builder:latest

docker push ghcr.io/camertron/rscc-darwin-x64-builder:latest
