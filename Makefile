RUSTC=/home/fklock/Dev/Mozilla/rust-gc/objdir-dbgopt/x86_64-unknown-linux-gnu/stage1/bin/rustc

# UW_PATH=/home/fklock/Dev/Mozilla/libunwind/src
UW_PATH=/home/fklock/opt/libunwind-dbg/lib
LIBRARY_PATH=/home/fklock/opt/libunwind-dbg/lib
# UW_PATH=/home/fklock/Dev/Mozilla/libunwind/src/.libs

default: run-foo

run-foo: foo
	./foo

SUBMODS=$(foreach path,$(wildcard */mod.rs),$(subst /mod.rs,,$(path)))

$(info RUSTC $(RUSTC))
$(info RUSTFLAGS $(RUSTFLAGS))
$(info UW_PATH $(UW_PATH))
$(info LIBRARY_PATH $(LIBRARY_PATH))

# $(info SUBMODS $(SUBMODS))

libutil.rlib: util.rs $(find $(SUBMODS) -name '*.rs') $(RUSTC) Makefile
#	LIBRARY_PATH=$(LIBRARY_PATH) $(RUSTC) $(RUSTFLAGS) -g util.rs -Z orbit -C link-dead-code -C "link-args=-Wl,-rpath=$(UW_PATH),--export-dynamic -lunwind" -Z print-link-args
	LIBRARY_PATH=$(LIBRARY_PATH) $(RUSTC) $(RUSTFLAGS) -g util.rs -Z orbit -C link-dead-code -C "link-args=-Wl,-rpath=$(UW_PATH),--export-dynamic -lunwind"

foo: foo.rs libutil.rlib $(RUSTC) Makefile
#	$(RUSTC) $(RUSTFLAGS) -g foo.rs -Z orbit -C link-dead-code -C link-args=-Wl,-rpath=$(UW_PATH) -lstatic=unwind -lstatic=unwind-x86_64 -Z print-link-args
#	LIBRARY_PATH=$(LIBRARY_PATH) $(RUSTC) $(RUSTFLAGS) -g foo.rs -Z orbit -C link-dead-code -C "link-args=-Wl,-rpath=$(UW_PATH) -lunwind -lunwind-x86_64" -Z print-link-args
#	LIBRARY_PATH=$(LIBRARY_PATH) $(RUSTC) $(RUSTFLAGS) -L . -g foo.rs -Z orbit -C link-dead-code -C "link-args=-Wl,-rpath=$(UW_PATH),--export-dynamic -lunwind" -Z print-link-args
	LIBRARY_PATH=$(LIBRARY_PATH) $(RUSTC) $(RUSTFLAGS) -L . -g foo.rs -Z orbit -C link-dead-code -C "link-args=-Wl,-rpath=$(UW_PATH),--export-dynamic -lunwind"

cfoo: foo.c Makefile
	$(CC) $(CFLAGS) foo.c -o $@

run-cfoo: cfoo
	LIBRARY_PATH=$(LIBRARY_PATH) ./cfoo

clean:
	rm -f foo cfoo
