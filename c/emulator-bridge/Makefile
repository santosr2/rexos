# Compiler and flags
CC := gcc
CFLAGS := -Wall -Wextra -std=c11 -O2 -fPIC
LDFLAGS := -shared

# Directories
SRC_DIR := .
BUILD_DIR := ../../build/c/emulator-bridge
OBJ_DIR := $(BUILD_DIR)/obj

# Target library
TARGET := $(BUILD_DIR)/librexos_emulator.so

# Source files
SOURCES := $(wildcard $(SRC_DIR)/*.c)
OBJECTS := $(patsubst $(SRC_DIR)/%.c,$(OBJ_DIR)/%.o,$(SOURCES))

# Default target
all: $(TARGET)

# Create build directories
$(BUILD_DIR) $(OBJ_DIR):
	mkdir -p $@

# Build shared library
$(TARGET): $(OBJECTS) | $(BUILD_DIR)
	$(CC) $(LDFLAGS) -o $@ $^
	@echo "Built: $@"

# Compile source files
$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)

# Install (placeholder)
install: $(TARGET)
	@echo "Install not yet implemented"

# Phony targets
.PHONY: all clean install

# Help
help:
	@echo "RexOS Emulator Bridge Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all     - Build the emulator bridge library"
	@echo "  clean   - Remove build artifacts"
	@echo "  install - Install library (not implemented)"
	@echo "  help    - Show this help message"
