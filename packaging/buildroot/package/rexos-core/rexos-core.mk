################################################################################
#
# rexos-core
#
################################################################################

REXOS_CORE_VERSION = 0.1.0
REXOS_CORE_SITE = $(BR2_EXTERNAL_REXOS_PATH)/..
REXOS_CORE_SITE_METHOD = local
REXOS_CORE_LICENSE = MIT
REXOS_CORE_LICENSE_FILES = LICENSE

REXOS_CORE_DEPENDENCIES = host-rustc

# Rust target based on architecture
ifeq ($(BR2_aarch64),y)
REXOS_CORE_RUST_TARGET = aarch64-unknown-linux-gnu
else ifeq ($(BR2_arm),y)
REXOS_CORE_RUST_TARGET = armv7-unknown-linux-gnueabihf
endif

define REXOS_CORE_BUILD_CMDS
	cd $(@D) && \
	CARGO_HOME=$(HOST_DIR)/share/cargo \
	RUSTFLAGS="-C linker=$(TARGET_CC)" \
	$(HOST_DIR)/bin/cargo build \
		--release \
		--target $(REXOS_CORE_RUST_TARGET) \
		--manifest-path $(@D)/Cargo.toml
endef

define REXOS_CORE_INSTALL_TARGET_CMDS
	# Install binaries
	$(INSTALL) -D -m 0755 $(@D)/target/$(REXOS_CORE_RUST_TARGET)/release/rexos-init \
		$(TARGET_DIR)/usr/bin/rexos-init
	$(INSTALL) -D -m 0755 $(@D)/target/$(REXOS_CORE_RUST_TARGET)/release/rexos-launcher \
		$(TARGET_DIR)/usr/bin/rexos-launcher

	# Install scripts
	$(INSTALL) -d $(TARGET_DIR)/rexos/scripts
	cp -r $(@D)/scripts/* $(TARGET_DIR)/rexos/scripts/
	find $(TARGET_DIR)/rexos/scripts -name "*.sh" -exec chmod +x {} \;

	# Create directory structure
	$(INSTALL) -d $(TARGET_DIR)/rexos/config
	$(INSTALL) -d $(TARGET_DIR)/rexos/data
	$(INSTALL) -d $(TARGET_DIR)/rexos/lib/libretro
endef

$(eval $(generic-package))
