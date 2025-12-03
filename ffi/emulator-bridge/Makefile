# RexOS Emulator Bridge Makefile

CC ?= gcc
AR ?= ar
CFLAGS = -Wall -Wextra -Werror -O2 -fPIC -std=c11
CFLAGS += -D_GNU_SOURCE -D_POSIX_C_SOURCE=200809L
LDFLAGS = -shared -lpthread

# Cross-compilation support
ifdef CROSS_COMPILE
    CC = $(CROSS_COMPILE)gcc
    AR = $(CROSS_COMPILE)ar
endif

# Target architecture
ARCH ?= $(shell uname -m)
ifeq ($(ARCH),aarch64)
    CFLAGS += -march=armv8-a
endif
ifeq ($(ARCH),armv7l)
    CFLAGS += -march=armv7-a -mfpu=neon
endif

# Directories
SRC_DIR := src
INC_DIR := include
BUILD_DIR := ../../target/ffi/emulator-bridge
OBJ_DIR := $(BUILD_DIR)/obj

# Source files
SOURCES := $(wildcard $(SRC_DIR)/*.c)
OBJECTS := $(patsubst $(SRC_DIR)/%.c,$(OBJ_DIR)/%.o,$(SOURCES))

# Include path
CFLAGS += -I$(INC_DIR)

# Output
LIB_SHARED := $(BUILD_DIR)/librexos_bridge.so
LIB_STATIC := $(BUILD_DIR)/librexos_bridge.a

# Install paths
PREFIX ?= /usr
LIBDIR = $(PREFIX)/lib
INCLUDEDIR = $(PREFIX)/include/rexos

.PHONY: all clean install uninstall debug help

all: $(LIB_SHARED) $(LIB_STATIC)

# Create directories
$(BUILD_DIR) $(OBJ_DIR):
	mkdir -p $@

# Shared library
$(LIB_SHARED): $(OBJECTS) | $(BUILD_DIR)
	$(CC) $(LDFLAGS) -o $@ $^
	@echo "Built: $@"

# Static library
$(LIB_STATIC): $(OBJECTS) | $(BUILD_DIR)
	$(AR) rcs $@ $^
	@echo "Built: $@"

# Object files (with header dependencies)
$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c $(INC_DIR)/emulator_bridge.h | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

install: all
	install -d $(DESTDIR)$(LIBDIR)
	install -d $(DESTDIR)$(INCLUDEDIR)
	install -m 755 $(LIB_SHARED) $(DESTDIR)$(LIBDIR)/
	install -m 644 $(LIB_STATIC) $(DESTDIR)$(LIBDIR)/
	install -m 644 $(INC_DIR)/*.h $(DESTDIR)$(INCLUDEDIR)/
	ldconfig 2>/dev/null || true
	@echo "Installed to $(DESTDIR)$(LIBDIR)"

uninstall:
	rm -f $(DESTDIR)$(LIBDIR)/librexos_bridge.so
	rm -f $(DESTDIR)$(LIBDIR)/librexos_bridge.a
	rm -f $(DESTDIR)$(INCLUDEDIR)/emulator_bridge.h
	rmdir $(DESTDIR)$(INCLUDEDIR) 2>/dev/null || true

clean:
	rm -rf $(BUILD_DIR)

debug: CFLAGS += -g -O0 -DDEBUG
debug: all

help:
	@echo "RexOS Emulator Bridge Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all      - Build shared and static libraries"
	@echo "  clean    - Remove build artifacts"
	@echo "  install  - Install libraries and headers"
	@echo "  debug    - Build with debug symbols"
	@echo "  help     - Show this help message"
	@echo ""
	@echo "Cross-compilation:"
	@echo "  make CROSS_COMPILE=aarch64-linux-gnu-"
