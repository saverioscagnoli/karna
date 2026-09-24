use core::any::TypeId;
use core::any::type_name;

use alloc::boxed::Box;
use alloc::ffi::CString;

use quickjs_sys::JS_GetAnyOpaque;
use quickjs_sys::JS_GetOpaque;
use quickjs_sys::JS_NewClass;
use quickjs_sys::JS_NewClassID;
use quickjs_sys::JSClassDef;
use quickjs_sys::JSClassID;
use quickjs_sys::JSRuntime;
use quickjs_sys::JSValue;

use crate::Error;
use crate::function::RawFn;
use crate::runtime::state;

/// # Safety
/// `rt` must be a live runtime.
pub(crate) unsafe fn register_closure_class(rt: *mut JSRuntime) -> Result<JSClassID, Error> {
    unsafe { new_class(rt, c"RustClosure".into(), Some(finalize_closure)) }
}

/// The class id backing `T`, registering the class on first use.
///
/// # Safety
/// `rt` must be a runtime created by [`Runtime::new`](crate::Runtime::new).
pub(crate) unsafe fn class_id<T: 'static>(rt: *mut JSRuntime) -> Result<JSClassID, Error> {
    let classes = &unsafe { state(rt) }.classes;

    if let Some(&id) = classes.borrow().get(&TypeId::of::<T>()) {
        return Ok(id);
    }

    let name = type_name::<T>();
    let name = name.rsplit("::").next().unwrap_or(name);
    let name = CString::new(name).map_err(|_| Error::InteriorNul)?;

    let id = unsafe { new_class(rt, name, Some(finalize::<T>)) }?;
    classes.borrow_mut().insert(TypeId::of::<T>(), id);

    Ok(id)
}

unsafe fn new_class(
    rt: *mut JSRuntime,
    name: CString,
    finalizer: quickjs_sys::JSClassFinalizer,
) -> Result<JSClassID, Error> {
    let mut id = 0;
    unsafe { JS_NewClassID(rt, &mut id) };

    let def = JSClassDef {
        class_name: name.as_ptr(),
        finalizer,
        ..Default::default()
    };

    // The name is interned as an atom, so `name` only has to live this long.
    if unsafe { JS_NewClass(rt, id, &def) } < 0 {
        return Err(Error::OutOfMemory);
    }

    Ok(id)
}

unsafe extern "C" fn finalize_closure(rt: *mut JSRuntime, val: JSValue) {
    let id = unsafe { state(rt) }.closure_class.get();
    let ptr = unsafe { JS_GetOpaque(val, id) }.cast::<Box<RawFn>>();

    if !ptr.is_null() {
        drop(unsafe { Box::from_raw(ptr) });
    }
}

unsafe extern "C" fn finalize<T>(_rt: *mut JSRuntime, val: JSValue) {
    let mut id = 0;
    let ptr = unsafe { JS_GetAnyOpaque(val, &mut id) }.cast::<T>();

    if !ptr.is_null() {
        drop(unsafe { Box::from_raw(ptr) });
    }
}
