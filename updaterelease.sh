#!/bin/sh
set -e
cargo test

CURRENT_VERSION="v$(cargo run -- --version 2> /dev/null | cut -d' ' -f2-)"

git commit -am "${CURRENT_VERSION}"
git tag "${CURRENT_VERSION}"
git push --tags

URL="https://github.com/mattiaslundberg/fastjump/archive/${CURRENT_VERSION}.tar.gz"
curl -O ${URL} -L

CHECKSUM=$(sha256sum ${CURRENT_VERSION}.tar.gz | cut -d' ' -f1)

rm ${CURRENT_VERSION}.tar.gz

sed -i '' "s|url .*|url \"${URL}\"|" ../homebrew-fastjump/fastjump.rb
sed -i '' "s/sha256 .*/sha256 \"${CHECKSUM}\"/" ../homebrew-fastjump/fastjump.rb
sed -i '' "s/version .*/version \"${CURRENT_VERSION}\"/" ../homebrew-fastjump/fastjump.rb

cd ../homebrew-fastjump/ && git commit -am "${CURRENT_VERSION}"
cd ../homebrew-fastjump/ && git push
