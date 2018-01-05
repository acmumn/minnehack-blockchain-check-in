name = "minnehack-check-in"
log = "{{name}}.log"

all: check doc build-debug test-debug
build: build-debug build-release
build-debug:
	cargo build
build-release:
	cargo build --release
check:
	cargo check
clean:
	cargo clean
doc:
	cargo doc
run-debug: build-debug
	cargo run
run-forever: build-release
	until cargo run --release |& tee -a {{log}}; do true; done
run-release: build-release
	cargo run --release
test: test-debug test-release
test-debug:
	cargo test
test-release:
	cargo test --release
watch TARGET="all":
	watchexec -cre rs,toml "just {{TARGET}}"
