default: fmt test

fmt:
	cargo +nightly-2024-09-11 fmt --all

test:
	cargo test --release --all-targets --all-features

dev:
	cargo r --release --bin subweight-web -- --endpoint 127.0.0.1 --repos polkadot-sdk --root repos --port 4000

publish:
	#!/usr/bin/env bash
	echo "Checking if crate can be published..."

	VERSION=$(shell grep -E '^version = "[0-9.]+"' Cargo.toml | cut -d'"' -f2)
	TAG=v$(VERSION)
	echo "Publishing version '$(VERSION)' with tag '$(TAG)'"
	echo "- Publishing crate to crates.io..."
	echo " - Publishing core..."
	cargo publish -p subweight-core --allow-dirty -q
	echo " - Publishing cli..."
	cargo publish -p subweight  --allow-dirty -q
	echo " - Publishing web..."
	cargo publish -p subweight-web  --allow-dirty -q
	echo "- Please sign the tag..."
	git tag -s -a $(TAG) -m "Version $(VERSION)"
	echo "- Pushing tag to GitHub..."
	git push -f origin $(TAG)
	echo "Done!"
