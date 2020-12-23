
GLSLANG = glslangValidator

OUTPUT = $(patsubst %.glsl,%.spv,$(wildcard dotrix_core/src/renderer/shaders/*.glsl))

all: $(OUTPUT)

$(OUTPUT): %.spv: %.glsl
	$(GLSLANG) -V $< -o $@

.PHONY: clean
clean:
	rm -f $(OUTPUT)
