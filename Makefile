BOOT_DIR = bootable_base/
BOOT_FILES = boot.s multiboot_header.s
LINKER_FILE = ${BOOT_DIR}/linker.ld

BOOT_SRC = $(addprefix $(BOOT_DIR), $(BOOT_FILES))

OBJS = ${BOOT_SRC:.s=.o}

NAME = kernel.bin

ISO = kfs.iso

ASM = nasm
ASM_FLAGS = -f elf32

LIB = target/kfs-1/debug/libkfs.a

GRUB = isofiles/boot/${NAME}

%.o: %.s		
		${ASM} ${ASM_FLAGS} $< -o $@

all: install ${NAME}

check-rust:
ifeq ($(shell command -v rustc && echo yes || echo no), no)
	@echo "rust is not installed. Installing rust..."
	@curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y;
else
	@echo "rust is already installed."
endif

check-rust-src:
ifeq ($(shell rustup show | grep -q 'rust-src' && echo yes || echo no), no)
	@echo "rust-src is not installed. Installing rust-src..."
	@rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
else
	@echo "rust-src is already installed."
endif

check-xorriso:
ifeq ($(shell command -v xorriso && echo yes || echo no), no)
	@echo "xorriso is not installed. Installing xorriso..."
	@sudo apt-get update && sudo apt-get install -y xorriso
else
	@echo "xorriso is already installed."
endif

check-qemu:
ifeq ($(shell command -v qemu-system-i386 && echo yes || echo no), no)
	@echo "qemu is not installed. Installing qemu..."
	@sudo apt-get update && sudo apt-get install -y qemu-system
else
	@echo "qemu is already installed."
endif

check-grub-mkrescue:
ifeq ($(shell command -v grub-mkrescue && echo yes || echo no), no)
	@echo "grub-mkrescue is not installed. Installing grub-mkrescue..."
	@sudo apt-get update && sudo apt-get install -y grub-mkrescue
else
	@echo "grub-mkrescue is already installed."
endif

check-grub-pc-bin:
ifeq ($(shell dpkg -s grub-pc-bin && echo yes || echo no), no)
	@echo "grub-pc-bin is not installed. Installing grub-pc-bin..."
	@sudo apt-get update && sudo apt-get install -y grub-pc-bin
else
	@echo "grub-pc-bin is already installed."
endif


${NAME}: ${OBJS} ${LIB}
	ld -m elf_i386 -n -o ${NAME} -T ${LINKER_FILE} ${OBJS} ${LIB}



${LIB}:
	cargo build

install: check-rust check-rust-src check-xorriso check-qemu check-grub-mkrescue check-grub-pc-bin all

${GRUB}: ${NAME}
	cp ${NAME} ${GRUB};

${ISO}: ${GRUB}
	grub-mkrescue -o ${ISO} ./isofiles

build: ${ISO}

run: build
	qemu-system-i386 -cdrom ${ISO}

clean:
	cargo clean; rm -f ${OBJS}

fclean: clean
	rm -f ${ISO} ${NAME} ${GRUB} Cargo.lock

re: fclean build

PHONY: all clean fclean re build run