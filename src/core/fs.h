#ifndef KARNA_FS_H
#define KARNA_FS_H

#include <stdbool.h>

#include <karna/types.h>

typedef struct {
    char **items;
    usize len, cap;
} StrList;

void strlist_free(StrList *list);

// Reads the whole file, NUL-terminated so the result doubles as a C string for
// source text. Caller frees. `out_size` excludes the terminator.
u8 *fs_read(const char *path, usize *out_size);
bool fs_write(const char *path, const void *data, usize size);

bool fs_exists(const char *path);
bool fs_is_dir(const char *path);

// The running executable's own path, which is what `bundle` copies and what an
// embedded game reads its payload out of. Caller frees.
char *fs_exe_path(void);

char *fs_dirname(const char *path);       // caller frees
const char *fs_basename(const char *path); // borrows from `path`
char *fs_join(const char *a, const char *b); // caller frees

// Collapses `.`/`..` segments and switches separators to '/', in place. Module
// names travel into a bundle, so they have to look the same on every platform.
void fs_normalize(char *path);

// Every file under `root`, as paths relative to it. Used to sweep a project's
// assets into a bundle.
bool fs_walk(const char *root, StrList *out);

#endif
