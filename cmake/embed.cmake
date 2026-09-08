# Turns a binary file into a C array, so a compiled shader is linked into the
# executable rather than shipped beside it -- a bundled game is one file, and
# that has to include its shaders.
#
#   cmake -DIN=x.spv -DOUT=x.h -DNAME=karna_shader_x -P cmake/embed.cmake

file(READ "${IN}" HEXDATA HEX)
string(LENGTH "${HEXDATA}" HEXLEN)
math(EXPR BYTELEN "${HEXLEN} / 2")

# "0a0b" -> "0x0a,0x0b," then a newline every 12 bytes to keep the header readable.
string(REGEX REPLACE "(..)" "0x\\1," BYTES "${HEXDATA}")
string(REGEX REPLACE "((0x..,){12})" "\\1\n    " BYTES "${BYTES}")

file(WRITE "${OUT}"
"// Generated from ${IN} -- do not edit.\n"
"#ifndef KARNA_EMBED_${NAME}\n"
"#define KARNA_EMBED_${NAME}\n"
"\n"
"#include <stddef.h>\n"
"#include <stdint.h>\n"
"\n"
"static const uint8_t ${NAME}[] = {\n"
"    ${BYTES}\n"
"};\n"
"\n"
"static const size_t ${NAME}_len = ${BYTELEN};\n"
"\n"
"#endif\n")
