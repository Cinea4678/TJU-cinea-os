.EXPORT_ALL_VARIABLES:

mode = release


bin = target/x86_64-cinea_os/$(mode)/bootimage-cinea-os.bin
img = disk.img

$(img):
	qemu-img create $(img) 32M

cargo-opts = --release --bin cinea_os
ifeq ($(mode),release)
	cargo-opts += --release
endif

image: $(img)
	touch src/lib.rs
	cargo bootimage
	dd conv=notrunc if=$(bin) of=$(img)