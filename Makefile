.PHONY: publish
publish:
	@echo "Checking if crate can be published..."
	@cargo publish --dry-run -p swc_core
	@cargo publish --dry-run -p swc_cli
	@cargo publish --dry-run -p swc_web

	$(eval VERSION := $(shell grep -E '^version = "[0-9.]+"' Cargo.toml | cut -d'"' -f2))
	$(eval TAG := v$(VERSION))
	@echo "Publishing version '$(VERSION)' with tag '$(TAG)'"
	@echo "- Please sign the tag..."
	git tag -s -a $(TAG) -m "Version $(VERSION)"
	@echo "- Pushing tag to GitHub..."
	git push origin $(TAG)
	@echo "- Publishing crate to crates.io..."
	@echo " - Publishing core..."
	@cargo publish -p swc_core
	@echo " - Publishing cli..."
	@cargo publish -p swc_cli
	@echo " - Publishing web..."
	@cargo publish -p swc_web
	@echo "Done!"
