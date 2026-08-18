//! User-defined functions written in Java.
//!
//! Statements step on the thread that called into JNI, so a callback always
//! runs on a thread the JVM already knows about; `attach_current_thread` finds
//! that attachment and leaves it in place when the guard drops.

use crate::errors::TURSO_ETC;
use crate::turso_connection::to_turso_connection;
use crate::utils::{set_err_msg_and_throw_exception, utf8_byte_arr_to_str};
use jni::objects::{GlobalRef, JByteArray, JObject, JObjectArray, JString, JValue};
use jni::sys::{jboolean, jint, jlong};
use jni::{AttachGuard, JNIEnv, JavaVM};
use std::sync::Arc;
use turso_core::{
    AggregateFunction, AggregateState, FunctionContext, FunctionFlags, LimboError, Numeric,
    ScalarFunction, Value, ValueRef,
};

/// Local references besides one per argument: the array, the returned object,
/// and room for what the conversions create.
const LOCAL_REFS_BESIDES_ARGS: i32 = 8;

/// Kept in step with the engine's wording in `core/udf.rs`.
const NOT_A_WINDOW_FUNCTION: &str = "aggregate may not be used as a window function";

enum UdfError {
    Jni(jni::errors::Error),
    Message(String),
}

impl From<jni::errors::Error> for UdfError {
    fn from(value: jni::errors::Error) -> Self {
        UdfError::Jni(value)
    }
}

impl From<UdfError> for LimboError {
    fn from(value: UdfError) -> Self {
        let message = match value {
            UdfError::Jni(err) => format!("JNI error while calling user-defined function: {err}"),
            UdfError::Message(msg) => msg,
        };
        LimboError::UserFunction { code: 1, message }
    }
}

type UdfResult<T> = std::result::Result<T, UdfError>;

/// Hands a callback's outcome to the engine. Any JNI failure may have left an exception
/// pending, and the engine must never get the thread back in that state.
fn settle<T>(env: &mut JNIEnv<'_>, result: UdfResult<T>) -> turso_core::Result<T> {
    if result.is_err() && env.exception_check().unwrap_or(true) {
        let _ = env.exception_clear();
    }
    Ok(result?)
}

fn attach(vm: &JavaVM) -> turso_core::Result<AttachGuard<'_>> {
    vm.attach_current_thread()
        .map_err(|err| LimboError::from(UdfError::Jni(err)))
}

fn local_refs_needed(arg_count: usize) -> i32 {
    arg_count as i32 + LOCAL_REFS_BESIDES_ARGS
}

/// The engine must never be handed back a thread with a pending exception, so
/// this clears it even when reading the message fails.
fn take_exception_message(env: &mut JNIEnv<'_>) -> String {
    let throwable = match env.exception_occurred() {
        Ok(throwable) if !throwable.is_null() => throwable,
        _ => {
            let _ = env.exception_clear();
            return "user-defined function raised exception: unknown error".to_string();
        }
    };
    let _ = env.exception_clear();

    let mut message = String::new();
    if let Ok(value) = env.call_method(&throwable, "getMessage", "()Ljava/lang/String;", &[]) {
        if let Ok(text) = value.l() {
            if !text.is_null() {
                if let Ok(text) = env.get_string(&JString::from(text)) {
                    message = text.into();
                }
            }
        }
    }
    if message.is_empty() {
        message = java_class_name(env, &throwable).unwrap_or_else(|| "unknown error".to_string());
    }
    // Reading the message may itself have thrown.
    let _ = env.exception_clear();

    format!("user-defined function raised exception: {message}")
}

fn java_class_name(env: &mut JNIEnv<'_>, obj: &JObject<'_>) -> Option<String> {
    let class = env.get_object_class(obj).ok()?;
    let name = env
        .call_method(&class, "getName", "()Ljava/lang/String;", &[])
        .ok()?
        .l()
        .ok()?;
    Some(env.get_string(&JString::from(name)).ok()?.into())
}

fn call_java<'local>(
    env: &mut JNIEnv<'local>,
    receiver: &JObject<'_>,
    method: &str,
    signature: &str,
    args: &[JValue<'_, '_>],
) -> UdfResult<JObject<'local>> {
    match env.call_method(receiver, method, signature, args) {
        Ok(value) => Ok(value.l()?),
        Err(jni::errors::Error::JavaException) => {
            Err(UdfError::Message(take_exception_message(env)))
        }
        Err(err) => Err(UdfError::Jni(err)),
    }
}

fn args_to_java<'local>(
    env: &mut JNIEnv<'local>,
    args: &[ValueRef<'_>],
) -> UdfResult<JObject<'local>> {
    let array: JObjectArray<'local> =
        env.new_object_array(args.len() as i32, "java/lang/Object", JObject::null())?;
    for (index, arg) in args.iter().enumerate() {
        let obj = match arg {
            ValueRef::Null => JObject::null(),
            ValueRef::Numeric(Numeric::Integer(value)) => {
                env.new_object("java/lang/Long", "(J)V", &[JValue::Long(*value)])?
            }
            ValueRef::Numeric(Numeric::Float(value)) => env.new_object(
                "java/lang/Double",
                "(D)V",
                &[JValue::Double(f64::from(*value))],
            )?,
            ValueRef::Text(text) => env.new_string(text.as_str())?.into(),
            ValueRef::Blob(blob) => env.byte_array_from_slice(blob)?.into(),
        };
        env.set_object_array_element(&array, index as i32, obj)?;
    }
    Ok(array.into())
}

fn java_to_value<'local>(env: &mut JNIEnv<'local>, obj: JObject<'local>) -> UdfResult<Value> {
    if obj.is_null() {
        return Ok(Value::Null);
    }
    if env.is_instance_of(&obj, "java/lang/Long")?
        || env.is_instance_of(&obj, "java/lang/Integer")?
        || env.is_instance_of(&obj, "java/lang/Short")?
        || env.is_instance_of(&obj, "java/lang/Byte")?
    {
        let value = env.call_method(&obj, "longValue", "()J", &[])?.j()?;
        return Ok(Value::from_i64(value));
    }
    if env.is_instance_of(&obj, "java/lang/String")? {
        let text: String = env.get_string(&JString::from(obj))?.into();
        return Ok(Value::build_text(text));
    }
    if env.is_instance_of(&obj, "java/lang/Double")?
        || env.is_instance_of(&obj, "java/lang/Float")?
    {
        let value = env.call_method(&obj, "doubleValue", "()D", &[])?.d()?;
        return Ok(Value::from_f64(value));
    }
    if env.is_instance_of(&obj, "java/lang/Boolean")? {
        let value = env.call_method(&obj, "booleanValue", "()Z", &[])?.z()?;
        return Ok(Value::from_i64(i64::from(value)));
    }
    if env.is_instance_of(&obj, "[B")? {
        let blob = env.convert_byte_array(JByteArray::from(obj))?;
        return Ok(Value::Blob(blob));
    }

    let class_name = java_class_name(env, &obj).unwrap_or_else(|| "unknown".to_string());
    Err(UdfError::Message(format!(
        "unsupported return type from user-defined function: {class_name}"
    )))
}

/// A `null` accumulator is a legitimate state, so it is kept as `None`.
fn pin_state(env: &mut JNIEnv<'_>, obj: JObject<'_>) -> UdfResult<Option<GlobalRef>> {
    if obj.is_null() {
        return Ok(None);
    }
    Ok(Some(env.new_global_ref(&obj)?))
}

struct JavaScalarFunction {
    vm: JavaVM,
    callback: GlobalRef,
}

impl ScalarFunction for JavaScalarFunction {
    fn call(
        &self,
        _ctx: &mut FunctionContext<'_>,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<Value> {
        let mut guard = attach(&self.vm)?;
        let env: &mut JNIEnv<'_> = &mut guard;
        let result: UdfResult<Value> = env.with_local_frame(local_refs_needed(args.len()), |env| {
            let array = args_to_java(env, args)?;
            let returned = call_java(
                env,
                self.callback.as_obj(),
                "call",
                "([Ljava/lang/Object;)Ljava/lang/Object;",
                &[JValue::Object(&array)],
            )?;
            java_to_value(env, returned)
        });
        settle(env, result)
    }
}

struct JavaAggregateFunction {
    /// Shared with every accumulator: `JavaVM` cannot be cloned.
    vm: Arc<JavaVM>,
    callback: GlobalRef,
    /// Registered as a `WindowFunction`, so it also has `value` and `inverse`.
    window: bool,
}

impl AggregateFunction for JavaAggregateFunction {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> turso_core::Result<Box<dyn AggregateState>> {
        let mut guard = attach(&self.vm)?;
        let env: &mut JNIEnv<'_> = &mut guard;
        let state: UdfResult<Option<GlobalRef>> =
            env.with_local_frame(local_refs_needed(0), |env| {
                let returned = call_java(
                    env,
                    self.callback.as_obj(),
                    "init",
                    "()Ljava/lang/Object;",
                    &[],
                )?;
                pin_state(env, returned)
            });
        let state = settle(env, state)?;

        Ok(Box::new(JavaAggregateState {
            vm: self.vm.clone(),
            callback: self.callback.clone(),
            window: self.window,
            state,
        }))
    }

    fn supports_window(&self) -> bool {
        self.window
    }
}

struct JavaAggregateState {
    vm: Arc<JavaVM>,
    callback: GlobalRef,
    window: bool,
    state: Option<GlobalRef>,
}

impl JavaAggregateState {
    /// `method` is `step` or `inverse`.
    fn accumulate(&mut self, method: &str, args: &[ValueRef<'_>]) -> turso_core::Result<()> {
        let mut guard = attach(&self.vm)?;
        let env: &mut JNIEnv<'_> = &mut guard;
        let null_state = JObject::null();
        let state = self.state.as_ref().map_or(&null_state, |it| it.as_obj());
        let new_state: UdfResult<Option<GlobalRef>> =
            env.with_local_frame(local_refs_needed(args.len()), |env| {
                let array = args_to_java(env, args)?;
                let returned = call_java(
                    env,
                    self.callback.as_obj(),
                    method,
                    "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
                    &[JValue::Object(state), JValue::Object(&array)],
                )?;
                pin_state(env, returned)
            });
        self.state = settle(env, new_state)?;
        Ok(())
    }

    /// `method` is `result` or `value`.
    fn read(&self, method: &str) -> turso_core::Result<Value> {
        let mut guard = attach(&self.vm)?;
        let env: &mut JNIEnv<'_> = &mut guard;
        let null_state = JObject::null();
        let state = self.state.as_ref().map_or(&null_state, |it| it.as_obj());
        let result: UdfResult<Value> = env.with_local_frame(local_refs_needed(0), |env| {
            let returned = call_java(
                env,
                self.callback.as_obj(),
                method,
                "(Ljava/lang/Object;)Ljava/lang/Object;",
                &[JValue::Object(state)],
            )?;
            java_to_value(env, returned)
        });
        settle(env, result)
    }
}

impl AggregateState for JavaAggregateState {
    fn step(
        &mut self,
        _ctx: &mut FunctionContext<'_>,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<()> {
        self.accumulate("step", args)
    }

    fn finalize(self: Box<Self>, _ctx: &mut FunctionContext<'_>) -> turso_core::Result<Value> {
        self.read("result")
    }

    fn value(&self, _ctx: &mut FunctionContext<'_>) -> turso_core::Result<Value> {
        if !self.window {
            return Err(LimboError::ParseError(NOT_A_WINDOW_FUNCTION.to_string()));
        }
        self.read("value")
    }

    fn inverse(
        &mut self,
        _ctx: &mut FunctionContext<'_>,
        args: &[ValueRef<'_>],
    ) -> turso_core::Result<()> {
        if !self.window {
            return Err(LimboError::ParseError(NOT_A_WINDOW_FUNCTION.to_string()));
        }
        self.accumulate("inverse", args)
    }
}

/// SQLite ignores flag bits it does not know, and so do we.
fn function_flags(flags: jint) -> FunctionFlags {
    FunctionFlags::from_bits_truncate(flags as u32)
}

fn register_scalar(
    env: &mut JNIEnv<'_>,
    connection_ptr: jlong,
    name_bytes: JByteArray<'_>,
    n_args: jint,
    flags: jint,
    function: JObject<'_>,
) -> std::result::Result<(), String> {
    let connection = to_turso_connection(connection_ptr).map_err(|err| err.to_string())?;
    let name = utf8_byte_arr_to_str(env, name_bytes).map_err(|err| err.to_string())?;
    let vm = env.get_java_vm().map_err(|err| err.to_string())?;
    let callback = env
        .new_global_ref(&function)
        .map_err(|err| err.to_string())?;

    connection
        .conn
        .create_scalar_function(
            &name,
            n_args,
            function_flags(flags),
            JavaScalarFunction { vm, callback },
        )
        .map_err(|err| err.to_string())
}

fn register_aggregate(
    env: &mut JNIEnv<'_>,
    connection_ptr: jlong,
    name_bytes: JByteArray<'_>,
    n_args: jint,
    flags: jint,
    aggregate: JObject<'_>,
    window: bool,
) -> std::result::Result<(), String> {
    let connection = to_turso_connection(connection_ptr).map_err(|err| err.to_string())?;
    let name = utf8_byte_arr_to_str(env, name_bytes).map_err(|err| err.to_string())?;
    let vm = env.get_java_vm().map_err(|err| err.to_string())?;
    let callback = env
        .new_global_ref(&aggregate)
        .map_err(|err| err.to_string())?;

    connection
        .conn
        .create_aggregate_function(
            &name,
            n_args,
            function_flags(flags),
            JavaAggregateFunction {
                vm: Arc::new(vm),
                callback,
                window,
            },
        )
        .map_err(|err| err.to_string())
}

fn unregister(
    env: &mut JNIEnv<'_>,
    connection_ptr: jlong,
    name_bytes: JByteArray<'_>,
    n_args: jint,
) -> std::result::Result<(), String> {
    let connection = to_turso_connection(connection_ptr).map_err(|err| err.to_string())?;
    let name = utf8_byte_arr_to_str(env, name_bytes).map_err(|err| err.to_string())?;
    connection
        .conn
        .remove_function(&name, n_args)
        .map_err(|err| err.to_string())
}

#[no_mangle]
pub extern "system" fn Java_tech_turso_core_TursoConnection_createScalarFunctionUtf8<'local>(
    mut env: JNIEnv<'local>,
    obj: JObject<'local>,
    connection_ptr: jlong,
    name_bytes: JByteArray<'local>,
    n_args: jint,
    flags: jint,
    function: JObject<'local>,
) {
    if let Err(msg) = register_scalar(
        &mut env,
        connection_ptr,
        name_bytes,
        n_args,
        flags,
        function,
    ) {
        set_err_msg_and_throw_exception(&mut env, obj, TURSO_ETC, msg);
    }
}

#[no_mangle]
pub extern "system" fn Java_tech_turso_core_TursoConnection_createAggregateFunctionUtf8<'local>(
    mut env: JNIEnv<'local>,
    obj: JObject<'local>,
    connection_ptr: jlong,
    name_bytes: JByteArray<'local>,
    n_args: jint,
    flags: jint,
    aggregate: JObject<'local>,
    window: jboolean,
) {
    if let Err(msg) = register_aggregate(
        &mut env,
        connection_ptr,
        name_bytes,
        n_args,
        flags,
        aggregate,
        window != 0,
    ) {
        set_err_msg_and_throw_exception(&mut env, obj, TURSO_ETC, msg);
    }
}

#[no_mangle]
pub extern "system" fn Java_tech_turso_core_TursoConnection_removeFunctionUtf8<'local>(
    mut env: JNIEnv<'local>,
    obj: JObject<'local>,
    connection_ptr: jlong,
    name_bytes: JByteArray<'local>,
    n_args: jint,
) {
    if let Err(msg) = unregister(&mut env, connection_ptr, name_bytes, n_args) {
        set_err_msg_and_throw_exception(&mut env, obj, TURSO_ETC, msg);
    }
}
