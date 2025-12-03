################################################################################
#
# rexos-bridge
#
################################################################################

REXOS_BRIDGE_VERSION = 0.1.0
REXOS_BRIDGE_SITE = $(BR2_EXTERNAL_REXOS_PATH)/../c/emulator-bridge
REXOS_BRIDGE_SITE_METHOD = local
REXOS_BRIDGE_LICENSE = MIT
REXOS_BRIDGE_LICENSE_FILES = ../../LICENSE
REXOS_BRIDGE_INSTALL_STAGING = YES

define REXOS_BRIDGE_BUILD_CMDS
	$(MAKE) $(TARGET_CONFIGURE_OPTS) -C $(@D) \
		CC="$(TARGET_CC)" \
		AR="$(TARGET_AR)" \
		CFLAGS="$(TARGET_CFLAGS)" \
		LDFLAGS="$(TARGET_LDFLAGS)" \
		BUILD_DIR=$(@D)/build
endef

define REXOS_BRIDGE_INSTALL_STAGING_CMDS
	$(INSTALL) -D -m 0644 $(@D)/build/librexos_bridge.a \
		$(STAGING_DIR)/usr/lib/librexos_bridge.a
	$(INSTALL) -D -m 0644 $(@D)/emulator_bridge.h \
		$(STAGING_DIR)/usr/include/rexos/emulator_bridge.h
endef

define REXOS_BRIDGE_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 0755 $(@D)/build/librexos_bridge.so \
		$(TARGET_DIR)/usr/lib/librexos_bridge.so
endef

$(eval $(generic-package))
