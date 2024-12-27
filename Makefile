BOOT_DIR = bootable_base/
BOOT_FILES = boot.s multiboot_header.s
LINKER_FILE = ${BOOT_DIR}/linker.ld
BOOT_SRC = $(addprefix $(BOOT_DIR), $(BOOT_FILES))

RUST_DIR = src/
RUST_FILES = lib.rs io.rs utils.rs key_handlers.rs commands.rs screens.rs gdt.rs
RUST_SRC = $(addprefix $(RUST_DIR), $(RUST_FILES))

GRUB_CFG = isofiles/boot/grub/grub.cfg

OBJS = ${BOOT_SRC:.s=.o}

NAME = kernel.bin

ISO = kfs.iso

ASM = nasm
ASM_FLAGS = -f elf32

LIB = target/kfs-1/debug/libkfs.a

GRUB = isofiles/boot/${NAME}

RUST_PATH = $$HOME/.cargo/bin/

%.o: %.s		
		${ASM} ${ASM_FLAGS} $< -o $@

all: install ${NAME}

install: check-rust-nightly check-xorriso check-qemu check-grub-mkrescue check-grub-pc-bin

check-rust:
ifeq ($(shell command -v ${RUST_PATH}rustc && echo yes || echo no), no)
	@echo "rust is not installed. Installing rust..."
	@curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y;
endif

check-rust-nightly: check-rust
ifeq ($(shell ${RUST_PATH}rustup show | grep -q 'nightly' && echo yes || echo no), no)
	@echo "rust-nightly is not installed. Installing rust-nightly..."
	@${RUST_PATH}rustup install nightly
	@${RUST_PATH}rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
endif

check-xorriso:
ifeq ($(shell command -v xorriso && echo yes || echo no), no)
	@echo "xorriso is not installed. Installing xorriso..."
	@sudo apt-get update && sudo apt-get install -y xorriso
endif

check-qemu:
ifeq ($(shell command -v qemu-system-i386 && echo yes || echo no), no)
	@echo "qemu is not installed. Installing qemu..."
	@sudo apt-get update && sudo apt-get install -y qemu-system
endif

check-grub-mkrescue:
ifeq ($(shell command -v grub-mkrescue && echo yes || echo no), no)
	@echo "grub-mkrescue is not installed. Installing grub-mkrescue..."
	@sudo apt-get update && sudo apt-get install -y grub-mkrescue
endif

check-grub-pc-bin:
ifeq ($(shell dpkg -s grub-pc-bin && echo yes || echo no), no)
	@echo "grub-pc-bin is not installed. Installing grub-pc-bin..."
	@sudo apt-get update && sudo DEBIAN_FRONTEND=noninteractive apt-get install -y grub-pc-bin
endif


${LIB}: ${RUST_SRC} Cargo.toml .cargo/config.toml
	${RUST_PATH}cargo build

${NAME}: ${OBJS} ${LIB}
	ld -m elf_i386 -n -o ${NAME} -T ${LINKER_FILE} ${OBJS} ${LIB}

${GRUB}: ${NAME}
	cp ${NAME} ${GRUB};

${ISO}: ${GRUB_CFG} ${GRUB}
	grub-mkrescue -o ${ISO} ./isofiles

build: ${ISO}

run: all build
	qemu-system-i386 -cdrom ${ISO}

clean:
	${RUST_PATH}cargo clean; rm -f ${OBJS}

fclean: clean
	rm -f ${ISO} ${NAME} ${GRUB} Cargo.lock

re: fclean build

PHONY: all clean fclean re build run install