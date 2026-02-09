TESTS_INIT=tests/minimal_init.lua
TESTS_DIR=tests/

.PHONY: test build install clean help

help:
	@echo "snapshot.nvim - Makefile commands"
	@echo ""
	@echo "  make build     Build the Rust image generator"
	@echo "  make install   Build and install the plugin"
	@echo "  make test      Run the test suite"
	@echo "  make clean     Clean build artifacts"
	@echo "  make help      Show this help message"

build:
	@echo "Building Rust image generator..."
	cd generator && cargo build --release
	@echo "Build complete! Generator: generator/target/release/snapshot-generator"

install: build
	@echo "Plugin installed successfully!"
	@echo "Add this to your config: require('snapshot').setup()"

clean:
	@echo "Cleaning build artifacts..."
	cd generator && cargo clean
	rm -f test_*.lua test_*.json test_init.vim *.png
	@echo "Clean complete!"

test:
	@nvim \
		--headless \
		--noplugin \
		-u ${TESTS_INIT} \
		-c "PlenaryBustedDirectory ${TESTS_DIR} { minimal_init = '${TESTS_INIT}' }"
