#!/bin/bash
# A script to build and pack an executable with README, LICENSE, etc. for
# distribution. 💚
DIST=./dist

cargo build --release

# Get default LLVM target triple and current cargo version
TARGET=$(rustc -vV | sed -n 's|host: ||p')
VERSION=$(awk '/version/' Cargo.toml | head -n1 | cut -d '"' -f2)
NAME=llux_${VERSION}_${TARGET}

# Copy contents
mkdir -p "${DIST}"
cp LICENSE README.md CHANGELOG.md preview.gif ./target/release/llux.exe "${DIST}"

# Time to pack :ayaya:
rm -f "${DIST}"/*.zip
zip -r -j "${NAME}".zip "${DIST}"/*
mv "${NAME}".zip "${DIST}"