.PHONY: publish
publish:
	@echo "Checking if crate can be published..."
	@cargo publish --dry-run -p subweight-core
	@cargo publish --dry-run -p subweight
	@cargo publish --dry-run -p subweight-web

	$(eval VERSION := $(shell grep -E '^version = "[0-9.]+"' Cargo.toml | cut -d'"' -f2))
	$(eval TAG := v$(VERSION))
	@echo "Publishing version '$(VERSION)' with tag '$(TAG)'"
	@echo "- Please sign the tag..."
	git tag -s -a $(TAG) -m "Version $(VERSION)"
	@echo "- Pushing tag to GitHub..."
	git push origin $(TAG)
	@echo "- Publishing crate to crates.io..."
	@echo " - Publishing core..."
	@cargo publish -p subweight-core
	@echo " - Publishing cli..."
	@cargo publish -p subweight
	@echo " - Publishing web..."
	@cargo publish -p subweight-web
	@echo "Done!"
