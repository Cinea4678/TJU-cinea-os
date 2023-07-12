user-rust:
	basename -s .rs src/usrbin/*.rs | xargs -I {} \
		touch dsk/bin/{}
	basename -s .rs src/usrbin/*.rs | xargs -I {} \
		cargo rustc --release --bin {}
	basename -s .rs src/usrbin/*.rs | xargs -I {} \
		cp target/x86_64-moros/release/{} dsk/bin/{}
	strip dsk/bin/*