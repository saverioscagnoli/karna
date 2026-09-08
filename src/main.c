#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bundle/payload.h"
#include "cli/cli.h"
#include "core/log.h"

static void usage(void) {
    fprintf(stderr,
            "karna -- a scene-based game framework\n"
            "\n"
            "usage:\n"
            "  karna run <file.js>              run a game from source\n"
            "  karna bundle <file.js> [-o out]  build a self-contained executable\n"
            "  karna <file.js>                  shorthand for run\n"
            "\n"
            "options:\n"
            "  -v, --verbose   log at debug level\n"
            "  -q, --quiet     log errors only\n"
            "\n"
            "The entry module default-exports either a scene or an app config:\n"
            "\n"
            "  export default {\n"
            "    title: \"my game\", width: 1280, height: 720,\n"
            "    scenes: { demo }, scene: \"demo\",\n"
            "  };\n");
}

static void configure_logging(int *argc, char **argv) {
    const char *env = getenv("KARNA_LOG");

    if (env) {
        if (strcmp(env, "trace") == 0)
            log_set_level(LOG_TRACE);
        else if (strcmp(env, "debug") == 0)
            log_set_level(LOG_DEBUG);
        else if (strcmp(env, "warn") == 0)
            log_set_level(LOG_WARN);
        else if (strcmp(env, "error") == 0)
            log_set_level(LOG_ERROR);
    }

    // The flags are pulled out of argv so the rest of the parsing only ever
    // sees positional arguments.
    int out = 1;

    for (int i = 1; i < *argc; i++) {
        if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--verbose") == 0)
            log_set_level(LOG_DEBUG);
        else if (strcmp(argv[i], "-q") == 0 || strcmp(argv[i], "--quiet") == 0)
            log_set_level(LOG_ERROR);
        else
            argv[out++] = argv[i];
    }

    *argc = out;
}

static bool has_js_suffix(const char *path) {
    usize len = strlen(path);

    return (len > 3 && strcmp(path + len - 3, ".js") == 0) ||
           (len > 4 && strcmp(path + len - 4, ".mjs") == 0);
}

int main(int argc, char **argv) {
    // Before anything else, so a shipped game still answers to -v and
    // KARNA_LOG when someone needs to see why it will not start.
    configure_logging(&argc, argv);

    // A bundled game is this same executable with a payload on the end, so the
    // next question is always which of the two is running.
    Pkg pkg;

    if (pkg_open_self(&pkg)) {
        int status = cli_play(&pkg);
        pkg_close(&pkg);

        return status;
    }

    if (argc < 2) {
        usage();
        return 1;
    }

    const char *command = argv[1];

    if (strcmp(command, "help") == 0 || strcmp(command, "--help") == 0 ||
        strcmp(command, "-h") == 0) {
        usage();
        return 0;
    }

    if (strcmp(command, "run") == 0) {
        if (argc < 3) {
            usage();
            return 1;
        }

        return cli_run(argv[2]);
    }

    if (strcmp(command, "bundle") == 0) {
        if (argc < 3) {
            usage();
            return 1;
        }

        const char *out = NULL;

        for (int i = 3; i < argc; i++) {
            if ((strcmp(argv[i], "-o") == 0 || strcmp(argv[i], "--output") == 0) && i + 1 < argc)
                out = argv[++i];
            else {
                log_error("unexpected argument: %s", argv[i]);
                return 1;
            }
        }

        return cli_bundle(argv[2], out);
    }

    if (has_js_suffix(command))
        return cli_run(command);

    log_error("unknown command: %s", command);
    usage();

    return 1;
}
