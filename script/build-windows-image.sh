#! /bin/bash

docker build \
    --cache-from ghcr.io/camertron/rscc-windows-install-builder:latest \
    --label "org.opencontainers.image.source=https://github.com/camertron/rscc" \
    --label "org.opencontainers.image.description=RSCC Windows installer builder" \
    --label "org.opencontainers.image.licenses=MIT" \
    --file windows/Dockerfile \
    --platform linux/amd64 \
    -t ghcr.io/camertron/rscc-windows-install-builder:latest .

docker push ghcr.io/camertron/rscc-windows-install-builder:latest
