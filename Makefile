
include config.mk

ASM_SRC:=$(wildcard $(SRCDIR)/arch/$(ARCH)/boot/*.asm)
RS_SRC:=$(wildcard $(SRCDIR)/*.rs) $(wildcard $(SRCDIR)/**/*.rs) $(wildcard $(SRCDIR)/**/**/*.rs)

SRC+=$(ASM_SRC) $(RS_SRC)
OBJ+=$(patsubst $(SRCDIR)/arch/$(ARCH)/boot/%.asm,$(OBJDIR)/%.asm.o,$(ASM_SRC))
LIBK:=$(BUILD)/libk.a
LIB+=$(LIBK)
IMAGE:=funky.iso
ISO:=isofiles

.PHONY: all run cargo clean mrproper

all: $(IMAGE)

run: $(IMAGE)
	$(RUN) $(QEMU) -cdrom $(IMAGE)

$(LIBK): $(RS_SRC) Cargo.toml
	$(RUN) RUST_TARGET_PATH=$(shell pwd) xargo build --target $(TARGET) $(CARGO_FLAGS)

$(IMAGE): $(BUILD) $(OBJDIR) $(ISO) $(GRUB) $(KERNEL)
	$(RUN) cp grub.cfg $(ISO)/boot/grub/
	$(RUN) cp $(KERNEL) $(ISO)/boot/
	$(RUN) grub-mkrescue -o $(IMAGE) $(ISO)

$(OBJDIR)/%.asm.o: $(SRCDIR)/arch/$(ARCH)/boot/%.asm
	$(RUN) $(AS) $(ASFLAGS) $(INCLUDE) -o $@ $^

$(KERNEL): $(LINKER) $(OBJ) $(LIB)
	$(RUN) $(LD) $(LDFLAGS) -o $@ $(OBJ) $(LIB)

$(BUILD):
	$(RUN) mkdir -p $(BUILD)

$(OBJDIR):
	$(RUN) mkdir -p $(OBJDIR)

$(ISO):
	$(RUN) mkdir -p $(ISO)/boot/grub

clean:
	$(RUN) rm -rf target
	$(RUN) rm -rf $(ISO)

mrproper: clean
	$(RUN) rm -f $(IMAGE)
