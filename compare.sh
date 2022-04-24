#!/usr/bin/env bash

set -e

git submodule update --init

# Compare two commits
OLD="$1"
if [ -z "$OLD" ]; then
    echo "Usage: $0 <old commit> <new commit>"
	exit 1
fi

NEW="$2"
if [ -z "$NEW" ]; then
	echo "Usage: $0 <old commit> <new commit>"
	exit 1
fi

SUBFOLDER="runtime/polkadot/src/weights/"
ROOT="$(git rev-parse --show-toplevel)/test_data/"
echo "Comparing $OLD to $NEW"

# Checkout old
cd "$ROOT/polkadot_old" && git checkout "$OLD" && cd -
# Checkout new
cd "$ROOT/polkadot_new" && git checkout "$NEW" && cd -

echo "Compiling…"
# This also includes the mod.rs, but that is skipped by the CLI.
# It runs in debug mode, but its fast enough anyway ¯\_(ツ)_/¯.
cargo r --quiet --bin cli -- --new `find $ROOT/polkadot_new/$SUBFOLDER -type f` --old `find $ROOT/polkadot_old/$SUBFOLDER -type f` "${@:3}"
