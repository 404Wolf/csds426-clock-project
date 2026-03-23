CC = gcc
CFLAGS += -I$(ZMAP_SRC) -I$(ZMAP_SRC)/probe_modules -I$(ZMAP_SRC)/../lib
LDFLAGS = -shared

ZMAP_DIR = module
BUILD_DIR = build

SOURCES = $(ZMAP_DIR)/module_icmp_timestamp.c
TARGET = $(BUILD_DIR)/module_icmp_timestamp.so

all: $(BUILD_DIR) $(TARGET)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(TARGET): $(SOURCES) $(BUILD_DIR)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<

clean:
	rm -rf $(BUILD_DIR)

.PHONY: all clean install check
