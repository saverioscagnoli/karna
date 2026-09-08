#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#include "bundle/payload.h"
#include "core/da.h"
#include "core/fs.h"
#include "core/log.h"

// Fixed little-endian on the wire: a bundle built on one machine should keep
// working if it is ever copied to another.

static void put_u32(ByteList *out, u32 value) {
    u8 bytes[4] = {(u8)value, (u8)(value >> 8), (u8)(value >> 16), (u8)(value >> 24)};
    da_append(out, bytes, 4);
}

static void put_u64(ByteList *out, u64 value) {
    u8 bytes[8];

    for (int i = 0; i < 8; i++)
        bytes[i] = (u8)(value >> (i * 8));

    da_append(out, bytes, 8);
}

static u32 get_u32(const u8 *p) {
    return (u32)p[0] | ((u32)p[1] << 8) | ((u32)p[2] << 16) | ((u32)p[3] << 24);
}

static u64 get_u64(const u8 *p) {
    u64 value = 0;

    for (int i = 0; i < 8; i++)
        value |= (u64)p[i] << (i * 8);

    return value;
}

// A cursor over the payload that refuses to read past the end, so a truncated
// or corrupt bundle is rejected instead of walking off the buffer.
typedef struct {
    const u8 *p;
    const u8 *end;
    bool bad;
} Reader;

static const u8 *reader_take(Reader *r, usize count) {
    if (r->bad || (usize)(r->end - r->p) < count) {
        r->bad = true;
        return NULL;
    }

    const u8 *at = r->p;
    r->p += count;

    return at;
}

static u32 reader_u32(Reader *r) {
    const u8 *at = reader_take(r, 4);
    return at ? get_u32(at) : 0;
}

static u64 reader_u64(Reader *r) {
    const u8 *at = reader_take(r, 8);
    return at ? get_u64(at) : 0;
}

static char *reader_string(Reader *r) {
    u32 len = reader_u32(r);
    const u8 *at = reader_take(r, len);

    if (!at)
        return NULL;

    char *str = malloc(len + 1);
    memcpy(str, at, len);
    str[len] = '\0';

    return str;
}

// The offset at which a file's appended payload begins, or the file's size when
// there is none. `bundle` copies exactly this many bytes, which is what lets a
// bundled game be used as the runtime for the next bundle.
static bool read_footer(const char *path, u64 *payload_off, u64 *payload_size) {
    FILE *f = fopen(path, "rb");

    if (!f)
        return false;

    fseek(f, 0, SEEK_END);
    long size = ftell(f);

    if (size < PKG_FOOTER_SIZE) {
        fclose(f);
        return false;
    }

    u8 footer[PKG_FOOTER_SIZE];

    fseek(f, size - PKG_FOOTER_SIZE, SEEK_SET);
    bool ok = fread(footer, 1, PKG_FOOTER_SIZE, f) == PKG_FOOTER_SIZE;
    fclose(f);

    if (!ok || memcmp(footer, PKG_MAGIC, PKG_MAGIC_LEN) != 0)
        return false;

    u32 version = get_u32(footer + 8);

    if (version != PKG_VERSION) {
        log_error("bundle format v%u, this runtime speaks v%u", version, PKG_VERSION);
        return false;
    }

    u64 off = get_u64(footer + 16);
    u64 len = get_u64(footer + 24);

    if (off + len + PKG_FOOTER_SIZE != (u64)size)
        return false;

    *payload_off = off;
    *payload_size = len;

    return true;
}

u64 pkg_runtime_size(const char *path) {
    u64 off, size;

    if (read_footer(path, &off, &size))
        return off;

    FILE *f = fopen(path, "rb");

    if (!f)
        return 0;

    fseek(f, 0, SEEK_END);
    long end = ftell(f);
    fclose(f);

    return end > 0 ? (u64)end : 0;
}

static bool pkg_parse(Pkg *pkg) {
    Reader r = {.p = pkg->bytes, .end = pkg->bytes + pkg->size};

    u32 count = reader_u32(&r);
    pkg->entry = reader_string(&r);

    for (u32 i = 0; i < count && !r.bad; i++) {
        PkgEntry entry = {0};

        entry.name = reader_string(&r);

        const u8 *kind = reader_take(&r, 1);
        entry.kind = kind ? *kind : 0;

        entry.offset = reader_u64(&r);
        entry.size = reader_u64(&r);

        if (r.bad) {
            free(entry.name);
            break;
        }

        da_push(&pkg->entries, entry);
    }

    if (r.bad || !pkg->entry)
        return false;

    pkg->blobs = r.p;

    // Every entry has to land inside the blob region, or a lookup would hand
    // the JS engine bytes from outside the bundle.
    usize available = (usize)(r.end - r.p);

    for (usize i = 0; i < pkg->entries.len; i++) {
        const PkgEntry *entry = &pkg->entries.items[i];

        if (entry->offset > available || entry->size > available - entry->offset)
            return false;
    }

    return true;
}

bool pkg_open_self(Pkg *pkg) {
    memset(pkg, 0, sizeof(*pkg));

    char *exe = fs_exe_path();

    if (!exe)
        return false;

    u64 off, size;

    if (!read_footer(exe, &off, &size)) {
        free(exe);
        return false;
    }

    FILE *f = fopen(exe, "rb");
    free(exe);

    if (!f)
        return false;

    pkg->bytes = malloc(size);
    pkg->size = size;

    fseek(f, (long)off, SEEK_SET);
    bool ok = fread(pkg->bytes, 1, size, f) == size;
    fclose(f);

    if (!ok || !pkg_parse(pkg)) {
        log_error("this game's bundle is corrupt");
        pkg_close(pkg);
        return false;
    }

    return true;
}

void pkg_close(Pkg *pkg) {
    for (usize i = 0; i < pkg->entries.len; i++)
        free(pkg->entries.items[i].name);

    da_free(&pkg->entries);

    free(pkg->entry);
    free(pkg->bytes);

    memset(pkg, 0, sizeof(*pkg));
}

const PkgEntry *pkg_find(const Pkg *pkg, const char *name, PkgKind kind) {
    for (usize i = 0; i < pkg->entries.len; i++) {
        const PkgEntry *entry = &pkg->entries.items[i];

        if (entry->kind == kind && strcmp(entry->name, name) == 0)
            return entry;
    }

    return NULL;
}

const u8 *pkg_data(const Pkg *pkg, const PkgEntry *entry) {
    return pkg->blobs + entry->offset;
}

void pkg_writer_set_entry(PkgWriter *writer, const char *name) {
    free(writer->entry);
    writer->entry = strdup(name);
}

void pkg_writer_add(PkgWriter *writer, const char *name, PkgKind kind, const void *data,
                    usize size) {
    PkgEntry entry = {
        .name = strdup(name),
        .kind = (u8)kind,
        .offset = writer->blobs.len,
        .size = size,
    };

    da_append(&writer->blobs, (const u8 *)data, size);
    da_push(&writer->entries, entry);
}

bool pkg_writer_emit(PkgWriter *writer, const char *runtime, const char *out) {
    u64 runtime_size = pkg_runtime_size(runtime);

    if (runtime_size == 0) {
        log_error("could not read the karna runtime at %s", runtime);
        return false;
    }

    usize file_size = 0;
    u8 *runtime_bytes = fs_read(runtime, &file_size);

    if (!runtime_bytes) {
        log_error("could not read the karna runtime at %s", runtime);
        return false;
    }

    ByteList index = {0};

    put_u32(&index, (u32)writer->entries.len);

    u32 entry_len = (u32)strlen(writer->entry);
    put_u32(&index, entry_len);
    da_append(&index, (const u8 *)writer->entry, entry_len);

    for (usize i = 0; i < writer->entries.len; i++) {
        const PkgEntry *entry = &writer->entries.items[i];

        u32 name_len = (u32)strlen(entry->name);
        put_u32(&index, name_len);
        da_append(&index, (const u8 *)entry->name, name_len);

        da_push(&index, entry->kind);

        put_u64(&index, entry->offset);
        put_u64(&index, entry->size);
    }

    u64 payload_size = index.len + writer->blobs.len;

    ByteList footer = {0};
    da_append(&footer, (const u8 *)PKG_MAGIC, PKG_MAGIC_LEN);
    put_u32(&footer, PKG_VERSION);
    put_u32(&footer, 0); // reserved
    put_u64(&footer, runtime_size);
    put_u64(&footer, payload_size);

    FILE *f = fopen(out, "wb");

    if (!f) {
        log_error("could not write %s", out);
        free(runtime_bytes);
        da_free(&index);
        da_free(&footer);
        return false;
    }

    bool ok = fwrite(runtime_bytes, 1, runtime_size, f) == runtime_size &&
              fwrite(index.items, 1, index.len, f) == index.len &&
              (writer->blobs.len == 0 ||
               fwrite(writer->blobs.items, 1, writer->blobs.len, f) == writer->blobs.len) &&
              fwrite(footer.items, 1, footer.len, f) == footer.len;

    fclose(f);

    free(runtime_bytes);
    da_free(&index);
    da_free(&footer);

    if (!ok) {
        log_error("could not write %s", out);
        return false;
    }

    chmod(out, 0755);

    return true;
}

void pkg_writer_free(PkgWriter *writer) {
    for (usize i = 0; i < writer->entries.len; i++)
        free(writer->entries.items[i].name);

    da_free(&writer->entries);
    da_free(&writer->blobs);

    free(writer->entry);

    memset(writer, 0, sizeof(*writer));
}
