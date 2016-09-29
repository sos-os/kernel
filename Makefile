arch ?= x86_64
target ?= $(arch)-unknown-sos-gnu
iso := target/$(target)/release/sos-$(arch).iso
kernel := target/$(target)/release/libsos_kernel.a
isofiles := target/$(target)/release/isofiles

grub_cfg := src/arch/$(arch)/grub.cfg

.PHONY: all clean run iso cargo

all: $(kernel)

clean:
	@rm -r build
	@cargo clean

run: $(iso)
	@qemu-system-x86_64 -hda $(iso)

iso: $(iso)

cargo:
	@xargo build --release --target $(target)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p $(isofiles)/boot/grub
	@cp $(kernel) $(isofiles)/boot/
	@cp $(grub_cfg) $(isofiles)/boot/grub
	@grub-mkrescue -o $(iso) $(isofiles)/
	@rm -r $(isofiles)

$(kernel): cargo
