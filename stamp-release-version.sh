#!/bin/bash

set -euo pipefail

RELEASE_VERSION="${1:-}"
PACKAGE_NAME="circle-to-search-pc"
SEMVER_PATTERN='^[0-9]+\.[0-9]+\.[0-9]+$'

if [[ ! "$RELEASE_VERSION" =~ $SEMVER_PATTERN ]]; then
    echo "ERROR: expected a release version like 1.2.3, got '${RELEASE_VERSION}'" >&2
    exit 1
fi

export RELEASE_VERSION PACKAGE_NAME

perl -0pi -e 's/(\[package\]\r?\nname = "\Q$ENV{PACKAGE_NAME}\E"\r?\nversion = ")[^"]+"/${1}$ENV{RELEASE_VERSION}"/' Cargo.toml
perl -0pi -e 's/(\[package\.metadata\.bundle\]\r?\n(?:[^\[\r\n][^\r\n]*\r?\n)*?version = ")[^"]+"/${1}$ENV{RELEASE_VERSION}"/' Cargo.toml
perl -0pi -e 's/(\[\[package\]\]\r?\nname = "\Q$ENV{PACKAGE_NAME}\E"\r?\nversion = ")[^"]+"/${1}$ENV{RELEASE_VERSION}"/' Cargo.lock
perl -0pi -e 's/(<key>CFBundle(?:Short)?Version(?:String)?<\/key>\s*<string>)[^<]+(<\/string>)/${1}$ENV{RELEASE_VERSION}${2}/g' Info.plist

STAMPED_COUNT=$(grep -c "\"${RELEASE_VERSION}\"" Cargo.toml || true)
LOCK_STAMPED=$(grep -A1 "^name = \"${PACKAGE_NAME}\"" Cargo.lock | grep -c "^version = \"${RELEASE_VERSION}\"" || true)
PLIST_STAMPED=$(grep -c "<string>${RELEASE_VERSION}</string>" Info.plist || true)

if [ "$STAMPED_COUNT" -lt 2 ] || [ "$LOCK_STAMPED" -ne 1 ] || [ "$PLIST_STAMPED" -ne 2 ]; then
    echo "ERROR: version stamp incomplete (Cargo.toml=${STAMPED_COUNT}/2, Cargo.lock=${LOCK_STAMPED}/1, Info.plist=${PLIST_STAMPED}/2)" >&2
    exit 1
fi

echo "Stamped version ${RELEASE_VERSION} into Cargo.toml, Cargo.lock and Info.plist"
