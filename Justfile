all: check doc build-debug test-debug
build: build-debug build-release
build-debug:
	cargo build --all
build-release:
	cargo build --all --release
check:
	cargo check --all
clean:
	cargo clean
doc:
	cargo doc --all
test: test-debug test-release
test-debug:
	cargo test --all
test-release:
	cargo test --all --release
watch TARGET="all":
	watchexec -cre rs,toml "just {{TARGET}}"
