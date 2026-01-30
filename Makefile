# Makefile for building ZMap ICMP Timestamp Module

CC = gcc
CFLAGS = -Wall -Wextra -O2 -fPIC -std=gnu11
ifdef ZMAP_SRC
CFLAGS += -I$(ZMAP_SRC) -I$(ZMAP_SRC)/probe_modules -I$(ZMAP_SRC)/../lib
endif
LDFLAGS = -shared

# Directories
ZMAP_DIR = zmap
BUILD_DIR = build

# Source files
SOURCES = $(ZMAP_DIR)/module_icmp_timestamp.c
TARGET = $(BUILD_DIR)/module_icmp_timestamp.so

# Default target
all: $(BUILD_DIR) $(TARGET)

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Build shared library directly from source
$(TARGET): $(SOURCES) $(BUILD_DIR)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)

# Install (copy to a location where zmap can find it)
install: $(TARGET)
	@echo "Installing module to /usr/local/lib/zmap/"
	@sudo mkdir -p /usr/local/lib/zmap
	@sudo cp $(TARGET) /usr/local/lib/zmap/
	@echo "Module installed successfully"

# Check syntax without linking
check:
	$(CC) $(CFLAGS) -fsyntax-only $(SOURCES)

.PHONY: all clean install check
