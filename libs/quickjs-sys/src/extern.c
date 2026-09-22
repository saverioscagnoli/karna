#include "wrapper.h"

// Static wrappers

bool JS_VALUE_IS_NAN__extern(JSValue v) { return JS_VALUE_IS_NAN(v); }
const char * JS_AtomToCString__extern(JSContext *ctx, JSAtom atom) { return JS_AtomToCString(ctx, atom); }
JSValue JS_NewBool__extern(JSContext *ctx, bool val) { return JS_NewBool(ctx, val); }
JSValue JS_NewInt32__extern(JSContext *ctx, int32_t val) { return JS_NewInt32(ctx, val); }
JSValue JS_NewFloat64__extern(JSContext *ctx, double val) { return JS_NewFloat64(ctx, val); }
JSValue JS_NewCatchOffset__extern(JSContext *ctx, int32_t val) { return JS_NewCatchOffset(ctx, val); }
JSValue JS_NewInt64__extern(JSContext *ctx, int64_t val) { return JS_NewInt64(ctx, val); }
JSValue JS_NewUint32__extern(JSContext *ctx, uint32_t val) { return JS_NewUint32(ctx, val); }
JSValue JS_NewUint64__extern(JSContext *ctx, uint64_t val) { return JS_NewUint64(ctx, val); }
bool JS_IsNumber__extern(JSValue v) { return JS_IsNumber(v); }
bool JS_IsBigInt__extern(JSValue v) { return JS_IsBigInt(v); }
bool JS_IsBool__extern(JSValue v) { return JS_IsBool(v); }
bool JS_IsNull__extern(JSValue v) { return JS_IsNull(v); }
bool JS_IsUndefined__extern(JSValue v) { return JS_IsUndefined(v); }
bool JS_IsException__extern(JSValue v) { return JS_IsException(v); }
bool JS_IsUninitialized__extern(JSValue v) { return JS_IsUninitialized(v); }
bool JS_IsString__extern(JSValue v) { return JS_IsString(v); }
bool JS_IsSymbol__extern(JSValue v) { return JS_IsSymbol(v); }
bool JS_IsObject__extern(JSValue v) { return JS_IsObject(v); }
bool JS_IsModule__extern(JSValue v) { return JS_IsModule(v); }
JSValue JS_ToBoolean__extern(JSContext *ctx, JSValue val) { return JS_ToBoolean(ctx, val); }
int JS_ToUint32__extern(JSContext *ctx, uint32_t *pres, JSValue val) { return JS_ToUint32(ctx, pres, val); }
JSValue JS_NewString__extern(JSContext *ctx, const char *str) { return JS_NewString(ctx, str); }
const char * JS_ToCStringLen__extern(JSContext *ctx, size_t *plen, JSValue val1) { return JS_ToCStringLen(ctx, plen, val1); }
const char * JS_ToCString__extern(JSContext *ctx, JSValue val1) { return JS_ToCString(ctx, val1); }
const uint16_t * JS_ToCStringUTF16__extern(JSContext *ctx, JSValue val1) { return JS_ToCStringUTF16(ctx, val1); }
JSValue JS_NewCFunction__extern(JSContext *ctx, JSCFunction *func, const char *name, int length) { return JS_NewCFunction(ctx, func, name, length); }
JSValue JS_NewCFunctionMagic__extern(JSContext *ctx, JSCFunctionMagic *func, const char *name, int length, JSCFunctionEnum cproto, int magic) { return JS_NewCFunctionMagic(ctx, func, name, length, cproto, magic); }
