#!/bin/bash
# A script to build and pack an executable with README, LICENSE, etc. for
# distribution. 💚

cargo build --release

# Get default LLVM target triple and current cargo version
TARGET=$(rustc -vV | sed -n 's|host: ||p')
VERSION=$(awk '/version/' Cargo.toml | head -n1 | cut -d '"' -f2)
NAME=llux_${VERSION}_${TARGET}

# Copy contents
mkdir -p ./dist
cp LICENSE ./dist
cp README.md ./dist
cp CHANGELOG.md ./dist
cp preview.gif ./dist
cp ./target/release/llux.exe ./dist

# Time to pack :ayaya:
rm -f ./dist/*.zip
zip -r -j "${NAME}".zip ./dist/*
mv "${NAME}".zip ./dist