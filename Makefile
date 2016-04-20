RUSTC=/home/fklock/Dev/Mozilla/rust-gc/objdir-dbgopt/x86_64-unknown-linux-gnu/stage1/bin/rustc

default: run-foo

run-foo: foo
	./foo

foo: $(shell find . -name '*.rs') $(RUSTC)
	$(RUSTC) $(RUSTFLAGS) foo.rs -Z orbit -C link-dead-code

cfoo: foo.c
	$(CC) $(CFLAGS) -lunwind -lunwind-x86_64 foo.c -o $@

run-cfoo: cfoo
	./cfoo

