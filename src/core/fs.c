#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#if defined(_WIN32)
#include <windows.h>
#elif defined(__APPLE__)
#include <mach-o/dyld.h>
#else
#include <unistd.h>
#endif

#include "core/da.h"
#include "core/fs.h"

void strlist_free(StrList *list) {
    for (usize i = 0; i < list->len; i++)
        free(list->items[i]);
    da_free(list);
}

u8 *fs_read(const char *path, usize *out_size) {
    FILE *f = fopen(path, "rb");

    if (!f)
        return NULL;

    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);

    if (size < 0) {
        fclose(f);
        return NULL;
    }

    u8 *buf = malloc((usize)size + 1);

    if (fread(buf, 1, (usize)size, f) != (usize)size) {
        free(buf);
        fclose(f);
        return NULL;
    }

    fclose(f);

    buf[size] = '\0';

    if (out_size)
        *out_size = (usize)size;

    return buf;
}

bool fs_write(const char *path, const void *data, usize size) {
    FILE *f = fopen(path, "wb");

    if (!f)
        return false;

    bool ok = size == 0 || fwrite(data, 1, size, f) == size;
    fclose(f);

    return ok;
}

bool fs_exists(const char *path) {
    struct stat st;
    return stat(path, &st) == 0;
}

bool fs_is_dir(const char *path) {
    struct stat st;
    return stat(path, &st) == 0 && S_ISDIR(st.st_mode);
}

char *fs_exe_path(void) {
#if defined(_WIN32)
    char buf[MAX_PATH];
    DWORD n = GetModuleFileNameA(NULL, buf, sizeof(buf));

    return n > 0 && n < sizeof(buf) ? strdup(buf) : NULL;
#elif defined(__APPLE__)
    u32 size = 0;
    _NSGetExecutablePath(NULL, &size);

    char *buf = malloc(size);

    if (_NSGetExecutablePath(buf, &size) != 0) {
        free(buf);
        return NULL;
    }

    char *real = realpath(buf, NULL);
    free(buf);

    return real;
#else
    // Sized up front rather than probed: /proc/self/exe does not report a size
    // through stat, so the only way to know it fit is to leave room to spare.
    char buf[4096];
    ssize_t n = readlink("/proc/self/exe", buf, sizeof(buf) - 1);

    if (n <= 0)
        return NULL;

    buf[n] = '\0';

    return strdup(buf);
#endif
}

char *fs_dirname(const char *path) {
    const char *slash = strrchr(path, '/');

#ifdef _WIN32
    const char *back = strrchr(path, '\\');
    if (back > slash)
        slash = back;
#endif

    if (!slash)
        return strdup(".");

    usize len = (usize)(slash - path);

    if (len == 0)
        return strdup("/");

    char *dir = malloc(len + 1);
    memcpy(dir, path, len);
    dir[len] = '\0';

    return dir;
}

const char *fs_basename(const char *path) {
    const char *slash = strrchr(path, '/');

#ifdef _WIN32
    const char *back = strrchr(path, '\\');
    if (back > slash)
        slash = back;
#endif

    return slash ? slash + 1 : path;
}

char *fs_join(const char *a, const char *b) {
    if (!a || !*a || strcmp(a, ".") == 0)
        return strdup(b);

    if (!b || !*b)
        return strdup(a);

    usize alen = strlen(a);
    bool sep = a[alen - 1] != '/';

    char *out = malloc(alen + sep + strlen(b) + 1);
    sprintf(out, "%s%s%s", a, sep ? "/" : "", b);

    return out;
}

void fs_normalize(char *path) {
    for (char *p = path; *p; p++)
        if (*p == '\\')
            *p = '/';

    // Rebuilt in place from a stack of segment starts, so `a/b/../c` collapses
    // without a second buffer. Leading `..` is kept: a module may legitimately
    // sit above the project root, and the name only has to be stable.
    char *starts[256];
    int depth = 0;

    char *out = path;
    const char *in = path;

    bool absolute = *in == '/';

    if (absolute)
        *out++ = *in++;

    while (*in) {
        const char *seg = in;

        while (*in && *in != '/')
            in++;

        usize len = (usize)(in - seg);

        while (*in == '/')
            in++;

        if (len == 0 || (len == 1 && seg[0] == '.'))
            continue;

        if (len == 2 && seg[0] == '.' && seg[1] == '.' && depth > 0) {
            out = starts[--depth];
            continue;
        }

        if (out != path && out[-1] != '/')
            *out++ = '/';

        if (depth < (int)(sizeof(starts) / sizeof(*starts)))
            starts[depth++] = out;

        memmove(out, seg, len);
        out += len;
    }

    if (out == path || (absolute && out == path + 1))
        *out++ = absolute ? '\0' : '.';

    *out = '\0';
}

static bool walk_into(const char *root, const char *rel, StrList *out) {
    char *dir = rel ? fs_join(root, rel) : strdup(root);
    DIR *d = opendir(dir);

    if (!d) {
        free(dir);
        return false;
    }

    struct dirent *entry;

    while ((entry = readdir(d))) {
        if (entry->d_name[0] == '.')
            continue;

        char *child_rel = rel ? fs_join(rel, entry->d_name) : strdup(entry->d_name);
        char *child_abs = fs_join(dir, entry->d_name);

        if (fs_is_dir(child_abs)) {
            walk_into(root, child_rel, out);
            free(child_rel);
        } else {
            da_push(out, child_rel);
        }

        free(child_abs);
    }

    closedir(d);
    free(dir);

    return true;
}

bool fs_walk(const char *root, StrList *out) {
    return walk_into(root, NULL, out);
}
