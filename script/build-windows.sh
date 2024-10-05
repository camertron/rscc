#! /bin/bash

cross build --target=x86_64-pc-windows-gnu --release

docker pull ghcr.io/camertron/rscc-windows-install-builder:latest

docker build \
    --cache-from ghcr.io/camertron/rscc-windows-install-builder:latest \
    --label "org.opencontainers.image.source=https://github.com/camertron/rscc" \
    --label "org.opencontainers.image.description=RSCC Windows installer builder" \
    --label "org.opencontainers.image.licenses=MIT" \
    --file windows/Dockerfile \
    --platform linux/amd64 \
    -t ghcr.io/camertron/rscc-windows-install-builder:latest .

docker run \
    --platform linux/amd64 \
    --rm \
    -it \
    -v $PWD:/rscc \
    -t rscc-windows-install-builder:latest \
    /bin/bash -c 'cd /rscc/windows && wine /root/.wine/drive_c/Program\ Files/Inno\ Setup\ 6/ISCC.exe rscc.iss'
