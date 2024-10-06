#! /bin/bash

cross build --target=x86_64-pc-windows-gnu --release

docker run \
    --platform linux/amd64 \
    --rm \
    -v $PWD:/rscc \
    -t ghcr.io/camertron/rscc-windows-install-builder:latest \
    /bin/bash -c 'cd /rscc/windows && wine /root/.wine/drive_c/Program\ Files/Inno\ Setup\ 6/ISCC.exe /DMyAppVersion=$1 rscc.iss'

mkdir rscc-windows
cp target/x86_64-pc-windows-gnu/release/rscc.exe rscc-windows/
cp rsc.c rscc-windows/
cp -R windows/mingw64_rsc/ rscc-windows/mingw64_rsc/
tar -czvf rscc-windows.tar.gz rscc-windows/
