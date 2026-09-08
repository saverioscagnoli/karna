#ifndef KARNA_PAYLOAD_H
#define KARNA_PAYLOAD_H

#include <stdbool.h>

#include <karna/types.h>

// A bundled game is the karna runtime with its game appended and a fixed-size
// footer at the very end:
//
//   [ runtime executable ][ payload ][ footer ]
//
// The loader reads the footer off the tail of its own executable; if the magic
// is there it plays what is attached, otherwise it behaves as the CLI. Nothing
// in the runtime's own bytes moves, so the executable still runs normally.
#define PKG_MAGIC "KARNAPKG"
#define PKG_MAGIC_LEN 8
#define PKG_VERSION 1
#define PKG_FOOTER_SIZE 32

typedef enum {
    PKG_MODULE = 0, // QuickJS bytecode, keyed by module name
    PKG_ASSET = 1,  // a verbatim file, keyed by its path under the root
} PkgKind;

typedef struct {
    char *name;
    u8 kind;
    u64 offset; // from the start of the blob region
    u64 size;
} PkgEntry;

typedef struct {
    PkgEntry *items;
    usize len, cap;
} PkgEntryList;

typedef struct {
    u8 *bytes; // the payload, as read off the executable
    usize size;

    char *entry; // module name to evaluate first
    PkgEntryList entries;
    const u8 *blobs; // into `bytes`, where entry offsets are measured from
} Pkg;

// Reads a payload off the running executable. False -- with nothing allocated
// -- when this is a plain runtime rather than a bundled game.
bool pkg_open_self(Pkg *pkg);
void pkg_close(Pkg *pkg);

// How much of a file is runtime rather than payload -- the whole file when it
// carries none.
u64 pkg_runtime_size(const char *path);

const PkgEntry *pkg_find(const Pkg *pkg, const char *name, PkgKind kind);
const u8 *pkg_data(const Pkg *pkg, const PkgEntry *entry);

typedef struct {
    u8 *items;
    usize len, cap;
} ByteList;

typedef struct {
    char *entry;
    PkgEntryList entries;
    ByteList blobs;
} PkgWriter;

void pkg_writer_set_entry(PkgWriter *writer, const char *name);
void pkg_writer_add(PkgWriter *writer, const char *name, PkgKind kind, const void *data,
                    usize size);

// Copies `runtime` -- minus any payload it already carries -- then appends this
// one, and makes the result executable.
bool pkg_writer_emit(PkgWriter *writer, const char *runtime, const char *out);

void pkg_writer_free(PkgWriter *writer);

#endif
