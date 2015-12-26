arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso
target ?= $(arch)-unknown-none-gnu
rust_os := target/$(target)/debug/libsos_kernel.a

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))


.PHONY: all clean run iso cargo

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -hda $(iso)

iso: $(iso)

cargo:
	@echo CARGO
	@cargo build --target $(target)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): cargo $(assembly_object_files) $(linker_script)
	@echo LD $(kernel)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)

# compile assembly files
# build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
# 	@mkdir -p $(shell dirname $@)
# 	@nasm -felf64 -Isrc/arch/$(arch)/ $< -o $@
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm $(assembly_header_files)
	@echo NASM $<
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 -Isrc/arch/$(arch)/ $< -o $@

#==========================================================================
# Building the Rust runtime for our bare-metal target

# Where to put our compiled runtime libraries for this platform.
installed_target_libs := \
	$(shell multirust which rustc | \
		sed s,bin/rustc,lib/rustlib/$(target)/lib,)

runtime_rlibs := \
	$(installed_target_libs)/libcore.rlib \
	$(installed_target_libs)/liballoc.rlib \
	$(installed_target_libs)/librustc_unicode.rlib \
	$(installed_target_libs)/libcollections.rlib

RUSTC := \
	rustc --verbose --target $(target) \
		-Z no-landing-pads \
		--cfg disable_float \
		--out-dir $(installed_target_libs)

.PHONY: runtime

runtime: $(runtime_rlibs)

$(installed_target_libs):
	@mkdir -p $(installed_target_libs)

$(installed_target_libs)/%.rlib: lib/rust/src/%/lib.rs $(installed_target_libs)
	@echo RUSTC $<
	@$(RUSTC) $<
	@echo Check $(installed_target_libs)
