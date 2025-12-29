################################################################################
#
# libretro-cores
#
# Pre-built libretro cores for RexOS
# Downloads cores from libretro buildbot for target architecture
#
################################################################################

LIBRETRO_CORES_VERSION = 1.0.0
LIBRETRO_CORES_LICENSE = Various (GPL-2.0+, LGPL-2.1+, BSD)
LIBRETRO_CORES_LICENSE_FILES = LICENSE
LIBRETRO_CORES_SITE_METHOD = local
LIBRETRO_CORES_SITE = $(BR2_EXTERNAL_REXOS_PATH)/..

LIBRETRO_CORES_INSTALL_TARGET = YES
LIBRETRO_CORES_DEPENDENCIES = retroarch

# Determine architecture for buildbot downloads
ifeq ($(BR2_aarch64),y)
LIBRETRO_CORES_ARCH = aarch64
else ifeq ($(BR2_arm),y)
LIBRETRO_CORES_ARCH = armv7-neon-hf
else
LIBRETRO_CORES_ARCH = x86_64
endif

# Buildbot base URL
LIBRETRO_CORES_BUILDBOT = https://buildbot.libretro.com/nightly/linux/$(LIBRETRO_CORES_ARCH)/latest

# Essential cores to download
LIBRETRO_CORES_LIST = \
	mgba_libretro.so.zip \
	gambatte_libretro.so.zip \
	snes9x_libretro.so.zip \
	nestopia_libretro.so.zip \
	genesis_plus_gx_libretro.so.zip \
	pcsx_rearmed_libretro.so.zip \
	mupen64plus_next_libretro.so.zip \
	desmume_libretro.so.zip \
	mednafen_pce_libretro.so.zip \
	fbneo_libretro.so.zip \
	mame2003_plus_libretro.so.zip \
	picodrive_libretro.so.zip \
	fceumm_libretro.so.zip \
	ppsspp_libretro.so.zip \
	stella_libretro.so.zip \
	prosystem_libretro.so.zip

# Optional cores (enabled via BR2_PACKAGE_LIBRETRO_CORES_FLYCAST)
ifeq ($(BR2_PACKAGE_LIBRETRO_CORES_FLYCAST),y)
LIBRETRO_CORES_LIST += flycast_libretro.so.zip
endif

# Install directory
LIBRETRO_CORES_INSTALL_DIR = $(TARGET_DIR)/rexos/lib/libretro

# Download and install cores
define LIBRETRO_CORES_INSTALL_TARGET_CMDS
	@echo "Installing libretro cores for $(LIBRETRO_CORES_ARCH)..."
	mkdir -p $(LIBRETRO_CORES_INSTALL_DIR)
	mkdir -p $(LIBRETRO_CORES_INSTALL_DIR)/info

	@# Check if pre-downloaded cores exist in build/cores
	@if [ -d "$(@D)/build/cores" ] && ls $(@D)/build/cores/*.so >/dev/null 2>&1; then \
		echo "Using pre-downloaded cores from build/cores/"; \
		cp $(@D)/build/cores/*.so $(LIBRETRO_CORES_INSTALL_DIR)/; \
		if [ -d "$(@D)/build/cores/info" ]; then \
			cp -r $(@D)/build/cores/info/* $(LIBRETRO_CORES_INSTALL_DIR)/info/ 2>/dev/null || true; \
		fi; \
	else \
		echo "Downloading cores from libretro buildbot..."; \
		mkdir -p $(DL_DIR)/libretro-cores; \
		for core in $(LIBRETRO_CORES_LIST); do \
			so_name=$${core%.zip}; \
			if [ ! -f "$(LIBRETRO_CORES_INSTALL_DIR)/$$so_name" ]; then \
				echo "  Downloading $$core..."; \
				if wget -q --timeout=30 -O "$(DL_DIR)/libretro-cores/$$core" \
					"$(LIBRETRO_CORES_BUILDBOT)/$$core" 2>/dev/null; then \
					unzip -q -o "$(DL_DIR)/libretro-cores/$$core" -d $(LIBRETRO_CORES_INSTALL_DIR) || true; \
					rm -f "$(DL_DIR)/libretro-cores/$$core"; \
					echo "    ✓ $$so_name"; \
				else \
					echo "    ✗ Failed to download $$core (may not exist for $(LIBRETRO_CORES_ARCH))"; \
				fi; \
			fi; \
		done; \
	fi

	@# Download core info files
	@if [ ! -f "$(LIBRETRO_CORES_INSTALL_DIR)/info/mgba_libretro.info" ]; then \
		echo "Downloading core info files..."; \
		wget -q --timeout=30 -O "$(DL_DIR)/libretro-cores/info.zip" \
			"https://buildbot.libretro.com/assets/frontend/info.zip" 2>/dev/null && \
		unzip -q -o "$(DL_DIR)/libretro-cores/info.zip" -d $(LIBRETRO_CORES_INSTALL_DIR)/info || \
		echo "Warning: Could not download core info files"; \
		rm -f "$(DL_DIR)/libretro-cores/info.zip"; \
	fi

	@# Set permissions
	chmod 644 $(LIBRETRO_CORES_INSTALL_DIR)/*.so 2>/dev/null || true

	@echo "Libretro cores installation complete"
endef

$(eval $(generic-package))
