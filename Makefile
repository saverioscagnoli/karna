BUILD   ?= build
TARGET  := karna
TYPE    ?= Debug
EXAMPLE ?= examples/js/main.js

.PHONY: all run bundle clean rebuild release

# `make`         -> compile
all: | $(BUILD)
	@cmake --build $(BUILD) -j -- --no-print-directory

# `make run`     -> compile, then run the javascript example
run: all
	@./$(BUILD)/$(TARGET) run $(EXAMPLE)

# `make bundle`  -> compile, then build the example into a standalone binary
bundle: all
	@./$(BUILD)/$(TARGET) bundle $(EXAMPLE) -o $(BUILD)/demo -v

# `make release` -> optimized build in build-release/
release:
	@cmake -S . -B build-release -DCMAKE_BUILD_TYPE=Release
	@cmake --build build-release -j -- --no-print-directory

# `make clean`   -> nuke build dirs
clean:
	@rm -rf $(BUILD) build-release

rebuild: clean all

# configure once; cmake re-runs itself when CMakeLists.txt changes
$(BUILD):
	@cmake -S . -B $(BUILD) -DCMAKE_BUILD_TYPE=$(TYPE)
