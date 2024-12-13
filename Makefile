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

${LIB}:
	cargo build

${NAME}: ${OBJS} ${LIB}
	ld -m elf_i386 -n -o ${NAME} -T ${LINKER_FILE} ${OBJS} ${LIB}

all: ${NAME}

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