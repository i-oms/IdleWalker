#!/bin/sh
# Turns a release tag into the crate's version; the tag is the only place it is written.
#   stamp-version.sh --check v1.2.3   prints 1.2.3, or fails if the tag is malformed
#   stamp-version.sh v1.2.3           also writes it into Cargo.toml and Cargo.lock
set -e

check=false
if [ "$1" = "--check" ]; then
    check=true
    shift
fi
tag=$1

n='(0|[1-9][0-9]*)'
if ! printf '%s\n' "$tag" | grep -Eq "^v$n\.$n\.$n(-[0-9A-Za-z.-]+)?$"; then
    echo "Tag '$tag' is not vMAJOR.MINOR.PATCH[-prerelease]" >&2
    exit 1
fi
version=${tag#v}

if [ "$check" = false ]; then
    # Only the [package] version, which is the first version line of the file.
    awk -v v="$version" '!done && /^version = / { print "version = \"" v "\""; done = 1; next } { print }' Cargo.toml > Cargo.toml.new
    mv Cargo.toml.new Cargo.toml
    grep -q "^version = \"$version\"$" Cargo.toml
    # Any cargo command brings Cargo.lock's own entry into line, and nothing else in it.
    cargo metadata --format-version 1 --offline > /dev/null 2>&1 || cargo metadata --format-version 1 > /dev/null
fi
printf '%s\n' "$version"
