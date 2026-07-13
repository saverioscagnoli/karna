// Minimal WASI shim.
//
// dear imgui's static destructors reference __cxa_atexit, which drags
// musl's exit-time stdio flushing (fd_write & friends) into the module as
// imports. None of this can ever run in the browser — there is no process
// exit — so these stubs only exist to satisfy instantiation.
//
// Errno 8 = WASI EBADF ("bad file descriptor").

export function fd_write() {
  return 8;
}

export function fd_close() {
  return 8;
}

export function fd_seek() {
  return 8;
}

export function fd_fdstat_get() {
  return 8;
}

export function environ_get() {
  return 0;
}

export function environ_sizes_get() {
  return 0;
}

export function proc_exit(code) {
  throw new Error(`WASI proc_exit(${code}) called in the browser`);
}
