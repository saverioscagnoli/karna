// fileno() is POSIX, not ISO C: must be requested before any header is pulled
// in, or a strict -std=c17 build won't see it.
#if !defined(_WIN32) && !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif

#include "log.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <io.h>
#define isatty _isatty
#define fileno _fileno
#else
#include <unistd.h>
#endif

#define LOG_MSG_MAX 1024
#define LOG_MOD_MAX 64

static LogLevel s_level = LOG_TRACE;
static int s_color = -1; // -1 = not decided yet

static const char *const LEVEL_TAGS[] = {
    "TRCE",
    "DEBG",
    "INFO",
    "WARN",
    "ERRO",
    "FTAL",
};

static const char *const LEVEL_COLORS[] = {
    "\x1b[90m",   // trace: bright black
    "\x1b[36m",   // debug: cyan
    "\x1b[32m",   // info:  green
    "\x1b[33m",   // warn:  yellow
    "\x1b[31m",   // error: red
    "\x1b[1;31m", // fatal: bold red
};

static const char *const LEVEL_NAMES[] = {
    "trace", "debug", "info", "warn", "error", "fatal", "off",
};

void log_set_level(LogLevel level) { s_level = level; }

LogLevel log_get_level(void) { return s_level; }

void log_set_color(bool enabled) { s_color = enabled ? 1 : 0; }

static bool str_ieq(const char *a, const char *b) {
    for (; *a && *b; a++, b++) {
        int ca = (*a >= 'A' && *a <= 'Z') ? *a + 32 : *a;
        if (ca != *b) return false;
    }
    return *a == *b;
}

bool log_set_level_from_name(const char *name) {
    if (name == NULL) return false;

    for (size_t i = 0; i < sizeof(LEVEL_NAMES) / sizeof(*LEVEL_NAMES); i++) {
        if (str_ieq(name, LEVEL_NAMES[i])) {
            s_level = (LogLevel)i;
            return true;
        }
    }

    return false;
}

bool log_set_level_from_env(const char *var) {
    const char *value = getenv(var);
    return value != NULL && value[0] != '\0' && log_set_level_from_name(value);
}

static bool color_enabled(void) {
    if (s_color < 0) {
        const char *no_color = getenv("NO_COLOR");
        bool forbidden = no_color != NULL && no_color[0] != '\0';
        s_color = (!forbidden && isatty(fileno(stderr))) ? 1 : 0;
    }

    return s_color == 1;
}

// "engine::cursor" is passed through as-is; a __FILE__ path is reduced to its
// stem, so src/window.c becomes "window".
static const char *module_name(const char *module, char *buf, size_t buf_len) {
    if (strstr(module, "::") != NULL) return module;

    const char *base = module;
    for (const char *p = module; *p != '\0'; p++) {
        if (*p == '/' || *p == '\\') base = p + 1;
    }

    size_t len = strlen(base);
    const char *dot = strrchr(base, '.');
    if (dot != NULL && dot != base) len = (size_t)(dot - base);
    if (len >= buf_len) len = buf_len - 1;

    memcpy(buf, base, len);
    buf[len] = '\0';

    return buf;
}

void log_log(LogLevel level, const char *module, const char *fmt, ...) {
    if (level < s_level || level > LOG_FATAL) return;

    char msg[LOG_MSG_MAX];
    va_list args;
    va_start(args, fmt);
    int written = vsnprintf(msg, sizeof(msg), fmt, args);
    va_end(args);

    if (written < 0) return;
    if ((size_t)written >= sizeof(msg)) memcpy(msg + sizeof(msg) - 4, "...", 4);

    char mod_buf[LOG_MOD_MAX];
    const char *mod = module_name(module, mod_buf, sizeof(mod_buf));

    // Formatted into one buffer so a single fputs emits the whole line: stdio
    // locks the stream per call, so lines from other threads can't interleave.
    char line[LOG_MSG_MAX + 128];
    if (color_enabled()) {
        snprintf(line, sizeof(line), "%s[%s]\x1b[0m \x1b[2m[%s]\x1b[0m %s\n",
                 LEVEL_COLORS[level], LEVEL_TAGS[level], mod, msg);
    } else {
        snprintf(line, sizeof(line), "[%s] [%s] %s\n", LEVEL_TAGS[level], mod, msg);
    }

    fputs(line, stderr);
}
