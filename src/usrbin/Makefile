.PHONY: bin

mode = release
strip = false

RUSTC = cargo rustc
RUSTFLAGS = 
STRIP := $(strip)

SD = src/bin/  # Src dir

ifeq ($(mode),release)
	RUSTFLAGS += -r
else
	STRIP = false
endif

hello: src/bin/hello.rs
	$(RUSTC) $(RUSTFLAGS) --bin hello
	touch target/hello

nothing: src/bin/nothing.rs
	$(RUSTC) $(RUSTFLAGS) --bin nothing
	touch target/nothing

shell: src/bin/shell.rs
	$(RUSTC) $(RUSTFLAGS) --bin shell
	touch target/shell

infprint: src/bin/infprint.rs
	$(RUSTC) $(RUSTFLAGS) --bin infprint
	touch target/infprint

echo: src/bin/echo.rs
	$(RUSTC) $(RUSTFLAGS) --bin echo
	touch target/echo

taffy: src/bin/taffy.rs
	$(RUSTC) $(RUSTFLAGS) --bin taffy
	touch target/echo

bin: hello nothing shell infprint echo taffy
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cp target/x86_64-cinea_os/$(mode)/{} ../../dsk/bin/{}
	if [ "$(STRIP)" = "true" ] && [ `arch` = "x86_64" ]; then \
		strip ../../dsk/bin/*; \
	fi