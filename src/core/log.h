#ifndef KARNA_LOG_H
#define KARNA_LOG_H

#include <karna/types.h>

typedef enum {
    LOG_TRACE,
    LOG_DEBUG,
    LOG_INFO,
    LOG_WARN,
    LOG_ERROR,
    LOG_OFF,
} LogLevel;

void log_set_level(LogLevel level);
LogLevel log_level(void);

// `tag` names the source: "karna" for the engine, the script's path for
// anything coming out of `console`.
void log_write(LogLevel level, const char *tag, const char *fmt, ...)
    __attribute__((format(printf, 3, 4)));

#define log_trace(...) log_write(LOG_TRACE, "karna", __VA_ARGS__)
#define log_debug(...) log_write(LOG_DEBUG, "karna", __VA_ARGS__)
#define log_info(...) log_write(LOG_INFO, "karna", __VA_ARGS__)
#define log_warn(...) log_write(LOG_WARN, "karna", __VA_ARGS__)
#define log_error(...) log_write(LOG_ERROR, "karna", __VA_ARGS__)

// An error the process cannot continue past.
#define fatal(...)                                                                                 \
    do {                                                                                           \
        log_write(LOG_ERROR, "karna", __VA_ARGS__);                                                \
        exit(1);                                                                                   \
    } while (0)

#endif
