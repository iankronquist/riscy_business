NAME=overhyped
PLATFORM=rv64
ARCH=riscv
SUBARCH=64
LINKER_SCRIPT=src/link.ld
TYPE=debug
RUST_TARGET=riscv64gc-unknown-none-elf

RUST_LIB=target/$(RUST_TARGET)/$(TYPE)/lib$(NAME).a
LIBS=-L$(RUST_LIB)
SOURCES_ASM=$(wildcard src/asm/*.S)
OBJ_ASM=$(SOURCES_ASM:.S=.S.o)
KERNEL=$(NAME).elf
BINUTILS_TOOLCHAIN=riscv64-elf
LD=$(BINUTILS_TOOLCHAIN)-ld
AS=$(BINUTILS_TOOLCHAIN)-as
# C preprocessor, NOT c++
CPP=cpp

ASFLAGS=-g --noexecstack
ARCHASFLAGS=-march=rv64ima

QEMU=qemu-system-riscv64
QEMU_MACH=virt
QEMU_CPU=rv64
QEMU_CPUS=4
QEMU_MEM=128M

HARD_DRIVE=hdd.dsk
HARD_DRIVE_MB=32
HARD_DRIVE_ID=$(NAME)_drive


all: $(KERNEL)

$(HARD_DRIVE):
	dd if=/dev/zero of=$@ count=$(HARD_DRIVE_MB) bs=1m

%.S.o: %.S
	$(CPP) $< | $(AS) -o $@ $(ASFLAGS) $(ARCHASFLAGS)


$(RUST_LIB): $(wildcard src/*.rs)
	cargo build --target=$(RUST_TARGET)

$(KERNEL): $(LINKER_SCRIPT) $(OBJ_ASM) $(RUST_LIB)
	$(LD) -T $(LINKER_SCRIPT) $(OBJ_ASM) $(RUST_LIB) -o $@ $(LDFLAGS)

run: $(KERNEL) $(HARD_DRIVE)
	$(QEMU) -machine $(QEMU_MACH) -cpu $(QEMU_CPU) -smp $(QEMU_CPUS) -m $(QEMU_MEM)  -serial mon:stdio -bios none -kernel $(KERNEL) -drive if=none,format=raw,file=$(HARD_DRIVE),id=$(HARD_DRIVE_ID) -device virtio-blk-device,drive=$(HARD_DRIVE_ID)


.PHONY: clean run
clean:
	cargo clean
	rm -f $(OUT)	
