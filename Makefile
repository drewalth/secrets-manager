install: build-release
	sudo cp target/release/secrets-manager /usr/local/bin/

uninstall:
	sudo rm /usr/local/bin/secrets-manager

clean:
	cargo clean

test:
	cargo test

build-release: clean
	cargo build --release

build-debug: clean
	cargo build

create-alias:
	@echo "Adding alias to shell configuration files..."
	@if ! grep -q "alias secrets=" ~/.bashrc 2>/dev/null; then \
		echo "" >> ~/.bashrc; \
		echo "alias secrets='secrets-manager'" >> ~/.bashrc; \
		echo "Added alias to ~/.bashrc"; \
	else \
		echo "Alias already exists in ~/.bashrc"; \
	fi
	@if ! grep -q "alias secrets=" ~/.zshrc 2>/dev/null; then \
		echo "" >> ~/.zshrc; \
		echo "alias secrets='secrets-manager'" >> ~/.zshrc; \
		echo "Added alias to ~/.zshrc"; \
	else \
		echo "Alias already exists in ~/.zshrc"; \
	fi
	@echo "Please run 'source ~/.zshrc' or restart your terminal to use the alias"

.PHONY: install uninstall clean test build-release build-debug create-alias