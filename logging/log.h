#ifndef KARNA_LOG_H
#define KARNA_LOG_H

#include <stdbool.h>

typedef enum {
    LOG_TRACE,
    LOG_DEBUG,
    LOG_INFO,
    LOG_WARN,
    LOG_ERROR,
    LOG_FATAL,
    LOG_OFF, // set as the level to silence everything
} LogLevel;

void log_set_level(LogLevel level);
LogLevel log_get_level(void);

// "trace".."fatal"/"off", case-insensitive. Returns false on an unknown name.
bool log_set_level_from_name(const char *name);

// Reads an env var (e.g. log_set_level_from_env("KARNA_LOG")) and applies it.
// Returns false if unset or unrecognized; the level is left untouched.
bool log_set_level_from_env(const char *var);

// Force ANSI colors on/off. Default: auto (on iff stderr is a tty and
// NO_COLOR is unset).
void log_set_color(bool enabled);

#if defined(__GNUC__) || defined(__clang__)
#define KARNA_PRINTF(fmt_idx, arg_idx) __attribute__((format(printf, fmt_idx, arg_idx)))
#else
#define KARNA_PRINTF(fmt_idx, arg_idx)
#endif

void log_log(LogLevel level, const char *module, const char *fmt, ...) KARNA_PRINTF(3, 4);

// The module tag printed in the second bracket. Define it before including
// this header to spell a path out by hand:
//
//     #define LOG_MODULE "engine::cursor"
//     #include "log.h"
//
// Otherwise it falls back to the file stem: src/window.c -> [window].
#ifndef LOG_MODULE
#define LOG_MODULE __FILE__
#endif

// NOTE: on Windows, <windows.h> defines ERROR; include it before this header
// and #undef ERROR, or rename these macros.
#define TRACE(...) log_log(LOG_TRACE, LOG_MODULE, __VA_ARGS__)
#define DEBUG(...) log_log(LOG_DEBUG, LOG_MODULE, __VA_ARGS__)
#define INFO(...)  log_log(LOG_INFO,  LOG_MODULE, __VA_ARGS__)
#define WARN(...)  log_log(LOG_WARN,  LOG_MODULE, __VA_ARGS__)
#define ERROR(...) log_log(LOG_ERROR, LOG_MODULE, __VA_ARGS__)
#define FATAL(...) log_log(LOG_FATAL, LOG_MODULE, __VA_ARGS__)

#endif
