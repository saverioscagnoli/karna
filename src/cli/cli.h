#ifndef KARNA_CLI_H
#define KARNA_CLI_H

#include "bundle/payload.h"

// Runs a game from source. `entry` is a path to the entry module; its directory
// becomes the root that every other module and asset resolves against.
int cli_run(const char *entry);

// Runs the game attached to this executable.
int cli_play(const Pkg *pkg);

// Compiles the module graph reachable from `entry`, sweeps in the assets beside
// it, and writes a self-contained executable. `out` may be NULL.
int cli_bundle(const char *entry, const char *out);

#endif
