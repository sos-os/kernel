arch ?= x86_64
build_dir := build
iso_dir := $(build_dir)/isofiles
kernel := $(build_dir)/kernel-$(arch).bin
iso := $(build_dir)/os-$(arch).iso

target ?= $(arch)-unknown-linux-gnu
rust_os := target/$(target)/debug/libsos_kernel.a
linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
images_dir := images
assembly_source_files := $(wildcard src/arch/$(arch)/boot/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/boot/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))


.PHONY: all clean run iso

all: $(kernel)

clean:
	@rm -r build
	@cargo clean

run: $(iso)
	@qemu-system-x86_64 -hda $(iso)

iso: $(iso)

cargo:
	@cargo rustc --target $(target) -- -L /lib/rust/src/libcore -Z no-landing-pads

# Make the RustDoc documentation
doc:
	@cargo doc

$(iso): $(kernel) $(grub_cfg)
	@echo "[ISO ]" $@
	@cp $(images_dir)/grub/stage2_eltorito-x86 $(iso_dir)/stage2_eltorito
	@cp $(kernel) $(iso_dir)/boot/kernel.bin
	@$(MKISOFS) -D -joliet -quiet -input-charset iso8859-1 -R \
	-b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 4 \
	-boot-info-table -o $@ -V 'SOS' -graft-points \
	boot/grub/stage2_eltorito=$(iso_dir)/stage2_eltorito \
	boot/grub/menu.lst=$(iso_dir)/grub/menu.lst \
	boot/kernel.bin=$(iso_dir)/boot/kernel.bin
	@rm -r build/isofiles

$(kernel): cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@x86_64-elf-ld -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
