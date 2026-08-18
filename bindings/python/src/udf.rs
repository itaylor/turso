//! User-defined functions written in Python.
//!
//! The engine calls these from inside [`Python::detach`], so every callback
//! takes the GIL back with [`Python::attach`] first. A raised exception is
//! dropped and reported with the message CPython's `sqlite3` uses, which
//! `turso/lib.py` maps back to `sqlite3.OperationalError`.

use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyTuple;
use turso_sdk_kit::rsapi::{
    self,
    udf::{
        AggregateFunction, AggregateState, CoreResult, FunctionContext, FunctionFlags,
        ScalarFunction,
    },
    Numeric, Value, ValueRef,
};

use crate::turso::{py_to_db_value, turso_error_to_py_err};

/// Part of the API: `turso/lib.py` and user code match on these messages.
const SCALAR_RAISED: &str = "user-defined function raised exception";
const INIT_RAISED: &str = "user-defined aggregate's '__init__' method raised error";
const STEP_RAISED: &str = "user-defined aggregate's 'step' method raised error";
const FINALIZE_RAISED: &str = "user-defined aggregate's 'finalize' method raised error";
const VALUE_RAISED: &str = "user-defined aggregate's 'value' method raised error";
const INVERSE_RAISED: &str = "user-defined aggregate's 'inverse' method raised error";

thread_local! {
    /// The exception the last failing callback raised, attached as `__cause__` to the
    /// error the statement reports (see `turso_error_to_py_err`).
    static PENDING_PY_ERROR: std::cell::RefCell<Option<PyErr>> = const { std::cell::RefCell::new(None) };
}

pub(crate) fn take_pending_py_error() -> Option<PyErr> {
    PENDING_PY_ERROR.with(|slot| slot.borrow_mut().take())
}

/// Reports `message` like CPython does, keeping the raised exception for `__cause__`.
fn call_python<T>(
    ctx: &FunctionContext<'_>,
    message: &str,
    f: impl FnOnce() -> PyResult<T>,
) -> CoreResult<T> {
    f().map_err(|err| {
        PENDING_PY_ERROR.with(|slot| *slot.borrow_mut() = Some(err));
        ctx.error(message)
    })
}

fn value_ref_to_py(py: Python<'_>, value: &ValueRef<'_>) -> PyResult<Py<PyAny>> {
    match value {
        ValueRef::Null => Ok(py.None()),
        ValueRef::Numeric(Numeric::Integer(i)) => Ok(i.into_pyobject(py)?.into()),
        ValueRef::Numeric(Numeric::Float(f)) => Ok(f64::from(*f).into_pyobject(py)?.into()),
        ValueRef::Text(text) => Ok(text.as_str().into_pyobject(py)?.into()),
        ValueRef::Blob(blob) => Ok(pyo3::types::PyBytes::new(py, blob).into()),
    }
}

fn args_to_py<'py>(py: Python<'py>, args: &[ValueRef<'_>]) -> PyResult<Bound<'py, PyTuple>> {
    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        values.push(value_ref_to_py(py, arg)?);
    }
    PyTuple::new(py, &values)
}

struct PyScalarFunction {
    func: Py<PyAny>,
}

impl ScalarFunction for PyScalarFunction {
    fn call(&self, ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> CoreResult<Value> {
        Python::attach(|py| {
            call_python(ctx, SCALAR_RAISED, || {
                let args = args_to_py(py, args)?;
                py_to_db_value(self.func.bind(py).call1(args)?)
            })
        })
    }
}

/// One instance is built per group.
struct PyAggregateFunction {
    class: Arc<Py<PyAny>>,
    window: bool,
}

impl AggregateFunction for PyAggregateFunction {
    fn init(&self, _ctx: &mut FunctionContext<'_>) -> CoreResult<Box<dyn AggregateState>> {
        // Built on the first `step`, not here: CPython allocates its aggregate
        // context lazily, so a group with no rows never touches the class.
        Ok(Box::new(PyAggregateState {
            class: self.class.clone(),
            instance: None,
        }))
    }

    fn supports_window(&self) -> bool {
        self.window
    }
}

struct PyAggregateState {
    class: Arc<Py<PyAny>>,
    instance: Option<Py<PyAny>>,
}

impl PyAggregateState {
    fn instance<'py>(
        &mut self,
        ctx: &FunctionContext<'_>,
        py: Python<'py>,
    ) -> CoreResult<Bound<'py, PyAny>> {
        if self.instance.is_none() {
            let instance = call_python(ctx, INIT_RAISED, || self.class.bind(py).call0())?;
            self.instance = Some(instance.unbind());
        }
        let instance = self
            .instance
            .as_ref()
            .expect("instance was just created if it was missing");
        Ok(instance.bind(py).clone())
    }
}

impl AggregateState for PyAggregateState {
    fn step(&mut self, ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> CoreResult<()> {
        Python::attach(|py| {
            let instance = self.instance(ctx, py)?;
            call_python(ctx, STEP_RAISED, || {
                let args = args_to_py(py, args)?;
                instance.call_method1("step", args)?;
                Ok(())
            })
        })
    }

    fn finalize(self: Box<Self>, ctx: &mut FunctionContext<'_>) -> CoreResult<Value> {
        let Some(instance) = self.instance else {
            // No rows in this group, so the aggregate is NULL, like CPython.
            return Ok(Value::Null);
        };
        Python::attach(|py| {
            call_python(ctx, FINALIZE_RAISED, || {
                py_to_db_value(instance.bind(py).call_method0("finalize")?)
            })
        })
    }

    fn value(&self, ctx: &mut FunctionContext<'_>) -> CoreResult<Value> {
        let Some(instance) = self.instance.as_ref() else {
            // Empty frame: nothing stepped yet, so the value is NULL.
            return Ok(Value::Null);
        };
        Python::attach(|py| {
            call_python(ctx, VALUE_RAISED, || {
                py_to_db_value(instance.bind(py).call_method0("value")?)
            })
        })
    }

    fn inverse(&mut self, ctx: &mut FunctionContext<'_>, args: &[ValueRef<'_>]) -> CoreResult<()> {
        Python::attach(|py| {
            let instance = self.instance(ctx, py)?;
            call_python(ctx, INVERSE_RAISED, || {
                let args = args_to_py(py, args)?;
                instance.call_method1("inverse", args)?;
                Ok(())
            })
        })
    }
}

/// `narg` is -1 for a function taking any number of arguments.
pub(crate) fn create_scalar_function(
    connection: &Arc<rsapi::TursoConnection>,
    name: &str,
    narg: i32,
    func: Py<PyAny>,
    deterministic: bool,
) -> PyResult<()> {
    let flags = if deterministic {
        FunctionFlags::DETERMINISTIC
    } else {
        FunctionFlags::empty()
    };
    connection
        .create_scalar_function(name, narg, flags, PyScalarFunction { func })
        .map_err(turso_error_to_py_err)
}

/// With `window`, the class must also have `value` and `inverse` methods.
pub(crate) fn create_aggregate_function(
    connection: &Arc<rsapi::TursoConnection>,
    name: &str,
    narg: i32,
    class: Py<PyAny>,
    window: bool,
) -> PyResult<()> {
    connection
        .create_aggregate_function(
            name,
            narg,
            FunctionFlags::empty(),
            PyAggregateFunction {
                class: Arc::new(class),
                window,
            },
        )
        .map_err(turso_error_to_py_err)
}

/// Removing a function that was never registered is not an error.
pub(crate) fn remove_function(
    connection: &Arc<rsapi::TursoConnection>,
    name: &str,
    narg: i32,
) -> PyResult<()> {
    connection
        .remove_function(name, narg)
        .map_err(turso_error_to_py_err)
}
