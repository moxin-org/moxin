#
# Maintenance Makefile
#

# Enforce bash with fatal errors.
SHELL			:= /bin/bash -eo pipefail

# Keep intermediates around on failures for better caching.
.SECONDARY:

# Default build and source directories.
BUILDDIR		?= ./build
SRCDIR			?= .

#
# Target: help
#

.PHONY: help
help:
	@# 80-width marker:
	@#     01234567012345670123456701234567012345670123456701234567012345670123456701234567
	@echo "make [TARGETS...]"
	@echo
	@echo "The following targets are provided by this maintenance makefile:"
	@echo
	@echo "    help:               Print this usage information"
	@echo
	@echo "    flatpak-build:      Build the flatpak repository"
	@echo "    flatpak-run:        Run the flatpak repository"
	@echo
	@echo "    macos-build:        Build the macOS bundle"
	@echo "    macos-run:          Run the macOS bundle"
	@echo
	@echo "    windows-build:      Build the Windows application"
	@echo "    windows-run:        Run the Windows application"

#
# Target: BUILDDIR
#

$(BUILDDIR)/:
	mkdir -p "$@"

$(BUILDDIR)/%/:
	mkdir -p "$@"

#
# Target: FORCE
#
# Used as alternative to `.PHONY` if the target is not fixed.
#

.PHONY: FORCE
FORCE:

#
# Target: cargo-*
#
# The `cargo-build` target simply runs `cargo build --release` with the
# target-directory set to the default, and then copies all files that are
# needed by other parts of the makefile (for now: the main binary).
#
# XXX: We should just use `cargo metadata` to find the target-directory,
#      rather than enforcing the default. But we want to avoid calling
#      `cargo metadata` more than once, so we likely want some cached
#      variable.
#

.PHONY: cargo-build
cargo-build: export CARGO_TARGET_DIR=$(SRCDIR)/target/
cargo-build: | $(BUILDDIR)/target/
	cd "$(SRCDIR)" && cargo build --release
	cp "$(SRCDIR)/target/release/moxin" "$(BUILDDIR)/target/"

#
# Target: flatpak-*
#

.PHONY: flatpak-build
flatpak-build: cargo-build resources-build
flatpak-build: $(BUILDDIR)/wasmedge/linux-x86_64/core.archive
flatpak-build: $(BUILDDIR)/wasmedge/linux-x86_64/wasinn.archive
flatpak-build: FORCE | $(BUILDDIR)/flatpak/
	cp "$(SRCDIR)/pkg/flatpak-rs.robius.moxin.json" "$(BUILDDIR)/"
	flatpak-builder \
		--force-clean \
		--install \
		--user \
		"$(BUILDDIR)/flatpak/" \
		"$(BUILDDIR)/rs.robius.moxin.json"

.PHONY: flatpak-run
flatpak-run:
	flatpak run rs.robius.moxin

#
# Target: macos-*
#

MACOS_ARCH		?= arm64

$(BUILDDIR)/macos-arm64/Moxin.app: $(BUILDDIR)/wasmedge/macos-arm64/core.archive
$(BUILDDIR)/macos-arm64/Moxin.app: $(BUILDDIR)/wasmedge/macos-arm64/wasinn.archive
$(BUILDDIR)/macos-x86_64/Moxin.app: $(BUILDDIR)/wasmedge/macos-x86_64/core.archive
$(BUILDDIR)/macos-x86_64/Moxin.app: $(BUILDDIR)/wasmedge/macos-x86_64/wasinn.archive

$(BUILDDIR)/macos-%/Moxin.app: cargo-build resources-build FORCE | $(BUILDDIR)/macos-%/
	rm -rf "$@"
	mkdir "$@/" "$@/Contents"
	mkdir "$@/Contents/MacOS"
	mkdir "$@/Contents/Resources"
	cat "$(SRCDIR)/pkg/macos-Info.plist" >"$@/Contents/Info.plist"
	cat "$(SRCDIR)/pkg/macos-PkgInfo" >"$@/Contents/PkgInfo"
	install -Dm755 "$(BUILDDIR)/target/moxin" -t "$@/Contents/MacOS/"
	cp -R "$(BUILDDIR)/resources/." "$@/Contents/Resources"
	cp -R "$(BUILDDIR)/wasmedge/macos-$*/core/lib/." "$@/Contents/Frameworks"
	cp -R "$(BUILDDIR)/wasmedge/macos-$*/wasinn/"*.dylib "$@/Contents/Frameworks"

.PHONY: macos-build
macos-build: $(BUILDDIR)/macos-$(MACOS_ARCH)/Moxin.app

.PHONY: macos-run
macos-run:
	open $(BUILDDIR)/macos-$(MACOS_ARCH)/Moxin.app

#
# Target: resources-*
#
# The `resources-build` target populates `$(BUILDDIR)/resources` with the
# resources used by the application as well as its dependencies.
#

RESOURCES_WIDGETS_JQ	= .packages | map(select(.name == "makepad-widgets")) | .[0].manifest_path

.PHONY: resources-moxin
resources-moxin: | $(BUILDDIR)/resources/moxin/resources/
	cp -r "$(SRCDIR)/resources/." "$(BUILDDIR)/resources/moxin/resources"

.PHONY: resources-widgets
resources-widgets: | $(BUILDDIR)/resources/makepad_widgets/resources/
	$(eval RESOURCES_WIDGETS=$(shell dirname "$$(cargo metadata --format-version 1 | jq -er '$(RESOURCES_WIDGETS_JQ)')"))
	cp -r "$(RESOURCES_WIDGETS)/resources/." "$(BUILDDIR)/resources/makepad_widgets/resources"

.PHONY: resources-build
resources-build: resources-moxin resources-widgets

#
# Target: wasmedge-*
#
# The different `$(BUILDDIR)/wasmedge/*` targets fetch the selected WasmEdge
# release archive and extract it into the build-directory. It is ready to be
# bundled into application builds.
#

WASMEDGE_DL		?= https://github.com/WasmEdge/WasmEdge/releases/download
WASMEDGE_VERSION	?= 0.13.5

$(BUILDDIR)/wasmedge/linux-arm64/core.archive: WASMEDGE_PKG=-manylinux2014_aarch64.tar.gz
$(BUILDDIR)/wasmedge/linux-arm64/wasinn.archive: WASMEDGE_PKG=-manylinux2014_aarch64.tar.gz
$(BUILDDIR)/wasmedge/linux-x86_64/core.archive: WASMEDGE_PKG=-manylinux2014_x86_64.tar.gz
$(BUILDDIR)/wasmedge/linux-x86_64/wasinn.archive: WASMEDGE_PKG=-manylinux2014_x86_64.tar.gz
$(BUILDDIR)/wasmedge/macos-arm64/core.archive: WASMEDGE_PKG=-darwin_arm64.tar.gz
$(BUILDDIR)/wasmedge/macos-arm64/wasinn.archive: WASMEDGE_PKG=-darwin_arm64.tar.gz
$(BUILDDIR)/wasmedge/macos-x86_64/core.archive: WASMEDGE_PKG=-darwin_x86_64.tar.gz
$(BUILDDIR)/wasmedge/macos-x86_64/wasinn.archive: WASMEDGE_PKG=-darwin_x86_64.tar.gz
$(BUILDDIR)/wasmedge/windows-x86_64/core.archive: WASMEDGE_PKG=-windows-msvc.zip
$(BUILDDIR)/wasmedge/windows-x86_64/wasinn.archive: WASMEDGE_PKG=-windows_x86_64.zip

$(BUILDDIR)/wasmedge/%/core.archive: | $(BUILDDIR)/wasmedge/%/
	curl \
		--fail \
		--location \
		--output "$@" \
		--progress-bar \
		--show-error \
		"$(WASMEDGE_DL)/$(WASMEDGE_VERSION)/WasmEdge-$(WASMEDGE_VERSION)$(WASMEDGE_PKG)"
	rm -rf "$(dir $@)/core"
	mkdir "$(dir $@)/core"
	bsdtar \
		-x \
		--strip-components 1 \
		-C "$(dir $@)/core" \
		-f "$@"

$(BUILDDIR)/wasmedge/%/wasinn.archive: | $(BUILDDIR)/wasmedge/%/
	curl \
		--fail \
		--location \
		--output "$@" \
		--progress-bar \
		--show-error \
		"$(WASMEDGE_DL)/$(WASMEDGE_VERSION)/WasmEdge-plugin-wasi_nn-ggml-$(WASMEDGE_VERSION)$(WASMEDGE_PKG)"
	rm -rf "$(dir $@)/wasinn"
	mkdir "$(dir $@)/wasinn"
	bsdtar \
		-x \
		-C "$(dir $@)/wasinn" \
		-f "$@"

#
# Target: windows-*
#

.PHONY: windows-build
windows-build:
	@echo "<TBD>"

.PHONY: windows-run
windows-run:
	@echo "<TBD>"
