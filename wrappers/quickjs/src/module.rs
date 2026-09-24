use core::ffi::CStr;
use core::ffi::c_char;
use core::ffi::c_void;
use core::ptr;

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::format;
use alloc::string::String;

use quickjs_sys::JS_EVAL_FLAG_COMPILE_ONLY;
use quickjs_sys::JS_EVAL_TYPE_MODULE;
use quickjs_sys::JS_Eval;
use quickjs_sys::JS_EvalFunction;
use quickjs_sys::JS_FreeValue;
use quickjs_sys::JS_GetModuleNamespace;
use quickjs_sys::JS_GetRuntime;
use quickjs_sys::JS_IsException;
use quickjs_sys::JS_PROMISE_REJECTED;
use quickjs_sys::JS_PromiseResult;
use quickjs_sys::JS_PromiseState;
use quickjs_sys::JS_SetModuleLoaderFunc;
use quickjs_sys::JS_VALUE_GET_PTR;
use quickjs_sys::JSContext;
use quickjs_sys::JSModuleDef;

use crate::Context;
use crate::Error;
use crate::Runtime;
use crate::Value;
use crate::context::describe;
use crate::function::throw;
use crate::runtime::run_jobs;
use crate::runtime::state;

/// Resolves an `import` to module source. Receives the normalized module
/// name: relative specifiers are already resolved against the importing
/// module's name, so with file paths as names it is a path to read.
pub type ModuleLoader = dyn Fn(&str) -> Result<String, Error>;

impl Runtime {
    /// Let modules `import` other modules, fetching their source with
    /// `loader`. Without one, every `import` fails.
    pub fn set_module_loader<F>(&self, loader: F)
    where
        F: Fn(&str) -> Result<String, Error> + 'static,
    {
        *unsafe { state(self.as_ptr()) }.loader.borrow_mut() = Some(Box::new(loader));

        unsafe { JS_SetModuleLoaderFunc(self.as_ptr(), None, Some(load_module), ptr::null_mut()) };
    }
}

impl<'rt> Context<'rt> {
    /// Evaluate `src` as an ES module named `name` and return its namespace
    /// object (`ns.default`, `ns.someExport`, ...).
    ///
    /// Runs the job queue, so a module with top-level `await` has settled by
    /// the time this returns unless it waits on something outside JS.
    pub fn eval_module(&self, src: &str, name: &str) -> Result<Value<'_>, Error> {
        let func = self.eval_with(src, name, JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY)?;

        // The context's module list keeps the definition alive past `func`.
        let module = unsafe { JS_VALUE_GET_PTR(func.as_raw()) }.cast::<JSModuleDef>();

        // Consumes `func`; module evaluation yields a promise.
        let promise = unsafe { self.wrap(JS_EvalFunction(self.as_ptr(), func.into_raw())) }?;
        unsafe { run_jobs(JS_GetRuntime(self.as_ptr())) }?;

        if unsafe { JS_PromiseState(self.as_ptr(), promise.as_raw()) } == JS_PROMISE_REJECTED {
            let reason = unsafe { JS_PromiseResult(self.as_ptr(), promise.as_raw()) };
            return Err(describe(unsafe { Value::from_raw(self, reason) }));
        }

        unsafe { self.wrap(JS_GetModuleNamespace(self.as_ptr(), module)) }
    }
}

unsafe extern "C" fn load_module(
    ctx: *mut JSContext,
    name: *const c_char,
    _opaque: *mut c_void,
) -> *mut JSModuleDef {
    let name = unsafe { CStr::from_ptr(name) };
    let loader = unsafe { state(JS_GetRuntime(ctx)) }.loader.borrow();

    let source = match loader.as_ref() {
        Some(loader) => loader(&name.to_string_lossy()),
        None => Err(Error::custom("no module loader is set")),
    };
    drop(loader);

    // QuickJS' parser expects a nul-terminated buffer.
    let source = source.and_then(|s| CString::new(s).map_err(|_| Error::InteriorNul));

    let source = match source {
        Ok(source) => source,
        Err(e) => {
            let e = Error::custom(format!(
                "could not load module '{}': {e}",
                name.to_string_lossy()
            ));
            unsafe { throw(ctx, e) };
            return ptr::null_mut();
        }
    };

    let func = unsafe {
        JS_Eval(
            ctx,
            source.as_ptr(),
            source.as_bytes().len(),
            name.as_ptr(),
            (JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY) as i32,
        )
    };

    if unsafe { JS_IsException(func) } {
        return ptr::null_mut();
    }

    // Already referenced by the context's module list (see quickjs-libc's
    // js_module_load), so our reference has to go.
    let module = unsafe { JS_VALUE_GET_PTR(func) }.cast::<JSModuleDef>();
    unsafe { JS_FreeValue(ctx, func) };

    module
}
