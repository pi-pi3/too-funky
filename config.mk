
IMAGE:=funky.img
ARCH?=x86
OS:=funky

SRCDIR:=src
INCLUDE:=-Iinclude/ -I$(SRCDIR)/
LIBDIR:=lib
LINKER:=linker.ld
GRUB:=grub.cfg

CC:=gcc
CXX:=g++
AS:=nasm
LD:=ld

ifeq ($(ARCH),x86_64)
EXEFORMAT:=elf64
BITS:=64
LDEMU:=elf_x86_64
QEMU:=qemu-system-x86_64
CPU:=x86_64
else ifeq ($(ARCH),x86)
EXEFORMAT:=elf32
BITS:=32
LDEMU:=elf_i386
QEMU:=qemu-system-i386
CPU:=i686
else
$(error invalid arch \"$(ARCH)\")
endif


CFLAGS+=-D__funky_libk -D__funky_$(ARCH) -D__funky_arch=$(ARCH) \
		-Wall -Wextra -m$(BITS) --std=c99 -nostdlib \
		-fno-builtin -ffreestanding -fno-stack-protector \
		-nostartfiles -nodefaultlibs
CXXFLAGS+=
ASFLAGS+=-f $(EXEFORMAT) 
LDFLAGS+=-n -T $(LINKER) -m $(LDEMU) --gc-sections

ifdef NDEBUG
CFLAGS+=-Os -DNDEBUG
CXXFLAGS+=$(CFLAGS)
ASFLAGS+=
LDFLAGS+=
RELEASE:=release
CARGO_FLAGS+=--release
else
CFLAGS+=-O0 -DDEBUG
CXXFLAGS+=$(CFLAGS)
ASFLAGS+=
LDFLAGS+=
RELEASE:=debug
CARGO_FLAGS+=
endif

ifdef DRYRUN
RUN:=@echo
else
RUN:=
endif

TARGET:=$(CPU)-unknown-$(OS)
BUILD:=target/$(TARGET)/$(RELEASE)
OBJDIR:=$(BUILD)/obj
KERNEL:=$(BUILD)/kernel.bin
