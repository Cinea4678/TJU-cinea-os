rust:
	basename -s .rs src/bin/*.rs | xargs -I {} \
		touch ../../dsk/bin/{}
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cargo rustc --release --bin {}
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cp target/x86_64-cinea_os/release/{} dsk/bin/{}
	if [ `arch` = "x86_64" ]; then \
		strip dsk/bin/*; \
	fi