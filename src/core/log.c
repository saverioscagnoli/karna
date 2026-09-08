#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "log.h"

static LogLevel g_level = LOG_INFO;
static int g_color = -1; // -1 until probed

void log_set_level(LogLevel level) {
    g_level = level;
}

LogLevel log_level(void) {
    return g_level;
}

static const char *level_name(LogLevel level) {
    switch (level) {
    case LOG_TRACE: return "trace";
    case LOG_DEBUG: return "debug";
    case LOG_INFO: return "info";
    case LOG_WARN: return "warn";
    case LOG_ERROR: return "error";
    default: return "off";
    }
}

static const char *level_color(LogLevel level) {
    switch (level) {
    case LOG_TRACE: return "\x1b[90m";
    case LOG_DEBUG: return "\x1b[36m";
    case LOG_INFO: return "\x1b[32m";
    case LOG_WARN: return "\x1b[33m";
    case LOG_ERROR: return "\x1b[31m";
    default: return "";
    }
}

void log_write(LogLevel level, const char *tag, const char *fmt, ...) {
    if (level < g_level)
        return;

    if (g_color < 0)
        g_color = isatty(fileno(stderr)) && !getenv("NO_COLOR");

    if (g_color)
        fprintf(stderr, "%s%-5s\x1b[0m \x1b[90m%s\x1b[0m ", level_color(level), level_name(level),
                tag);
    else
        fprintf(stderr, "%-5s %s ", level_name(level), tag);

    va_list args;
    va_start(args, fmt);
    vfprintf(stderr, fmt, args);
    va_end(args);

    fputc('\n', stderr);
}
