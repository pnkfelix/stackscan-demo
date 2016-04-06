RUSTC=/home/fklock/Dev/Mozilla/rust-gc/objdir-dbgopt/x86_64-unknown-linux-gnu/stage1/bin/rustc

default: run-foo

run-foo: foo
	./foo

foo: $(shell find . -name '*.rs')
	$(RUSTC) foo.rs -Z orbit -C link-dead-code
