
include config.mk

SRC+=$(shell find $(SRCDIR) -name '*.rs')
IMAGE:=funky.iso
ISO:=isofiles

.PHONY: all run cargo clean mrproper

all: $(IMAGE)

run: $(IMAGE)
	$(RUN) $(QEMU) $(QEMU_FLAGS) -cdrom $(IMAGE)

$(IMAGE): $(BUILD) $(ISO) $(GRUB) $(KERNEL)
	$(RUN) cp $(GRUB) $(ISO)/boot/grub/
	$(RUN) cp $(KERNEL) $(ISO)/boot/kernel.bin
	$(RUN) grub-mkrescue -o $(IMAGE) $(ISO)

$(KERNEL): $(SRC) $(LINKER) Cargo.toml $(TARGET).json
	$(RUN) RUST_TARGET_PATH=$(shell pwd) xargo build --target $(TARGET) $(CARGO_FLAGS)

$(BUILD):
	$(RUN) mkdir -p $(BUILD)

$(ISO):
	$(RUN) mkdir -p $(ISO)/boot/grub

clean:
	$(RUN) rm -rf target
	$(RUN) rm -rf $(ISO)

mrproper: clean
	$(RUN) rm -f $(IMAGE)
