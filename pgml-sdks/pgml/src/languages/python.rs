use futures::StreamExt;
use pyo3::conversion::IntoPy;
use pyo3::types::{PyDict, PyFloat, PyInt, PyList, PyString};
use pyo3::{prelude::*, types::PyBool};
use std::sync::Arc;

use rust_bridge::python::CustomInto;

use crate::{pipeline::PipelineSyncData, transformer_pipeline::TransformerStream, types::Json};

////////////////////////////////////////////////////////////////////////////////
// Rust to PY //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

impl IntoPy<PyObject> for Json {
    fn into_py(self, py: Python) -> PyObject {
        match &self.0 {
            serde_json::Value::Bool(x) => x.into_py(py),
            serde_json::Value::Number(x) => {
                if x.is_f64() {
                    x.as_f64()
                        .expect("Error converting to f64 in impl ToPyObject for Json")
                        .into_py(py)
                } else {
                    x.as_i64()
                        .expect("Error converting to i64 in impl ToPyObject for Json")
                        .into_py(py)
                }
            }
            serde_json::Value::String(x) => x.into_py(py),
            serde_json::Value::Array(x) => {
                let list = PyList::empty(py);
                for v in x.iter() {
                    list.append(Json(v.clone()).into_py(py)).unwrap();
                }
                list.into_py(py)
            }
            serde_json::Value::Object(x) => {
                let dict = PyDict::new(py);
                for (k, v) in x.iter() {
                    dict.set_item(k, Json(v.clone()).into_py(py)).unwrap();
                }
                dict.into_py(py)
            }
            serde_json::Value::Null => py.None(),
        }
    }
}

impl IntoPy<PyObject> for PipelineSyncData {
    fn into_py(self, py: Python) -> PyObject {
        Json::from(self).into_py(py)
    }
}

#[pyclass]
#[derive(Clone)]
struct TransformerStreamPython {
    wrapped: Arc<tokio::sync::Mutex<TransformerStream>>,
}

#[pymethods]
impl TransformerStreamPython {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'p>(slf: PyRefMut<'_, Self>, py: Python<'p>) -> PyResult<Option<PyObject>> {
        let ts = slf.wrapped.clone();
        let fut = pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut ts = ts.lock().await;
            if let Some(o) = ts.next().await {
                Ok(Some(Python::with_gil(|py| {
                    o.expect("Error calling next on TransformerStream")
                        .to_object(py)
                })))
            } else {
                Err(pyo3::exceptions::PyStopAsyncIteration::new_err(
                    "stream exhausted",
                ))
            }
        })?;
        Ok(Some(fut.into()))
    }
}

impl IntoPy<PyObject> for TransformerStream {
    fn into_py(self, py: Python) -> PyObject {
        let f: Py<TransformerStreamPython> = Py::new(
            py,
            TransformerStreamPython {
                wrapped: Arc::new(tokio::sync::Mutex::new(self)),
            },
        )
        .expect("Error converting TransformerStream to TransformerStreamPython");
        f.to_object(py)
    }
}

////////////////////////////////////////////////////////////////////////////////
// PY to Rust //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

impl FromPyObject<'_> for Json {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        if ob.is_instance_of::<PyDict>()? {
            let dict: &PyDict = ob.downcast()?;
            let mut json = serde_json::Map::new();
            for (key, value) in dict.iter() {
                let value = Json::extract(value)?;
                json.insert(String::extract(key)?, value.0);
            }
            Ok(Self(serde_json::Value::Object(json)))
        } else if ob.is_instance_of::<PyBool>()? {
            let value = bool::extract(ob)?;
            Ok(Self(serde_json::Value::Bool(value)))
        } else if ob.is_instance_of::<PyInt>()? {
            let value = i64::extract(ob)?;
            Ok(Self(serde_json::Value::Number(value.into())))
        } else if ob.is_instance_of::<PyFloat>()? {
            let value = f64::extract(ob)?;
            let value = serde_json::value::Number::from_f64(value)
                .expect("Could not convert f64 to serde_json::Number");
            Ok(Self(serde_json::Value::Number(value)))
        } else if ob.is_instance_of::<PyString>()? {
            let value = String::extract(ob)?;
            Ok(Self(serde_json::Value::String(value)))
        } else if ob.is_instance_of::<PyList>()? {
            let value = ob.downcast::<PyList>()?;
            let mut json_values = Vec::new();
            for v in value {
                let v = v.extract::<Json>()?;
                json_values.push(v.0);
            }
            Ok(Self(serde_json::Value::Array(json_values)))
        } else {
            if ob.is_none() {
                return Ok(Self(serde_json::Value::Null));
            }
            panic!("Unsupported type for JSON conversion");
        }
    }
}

impl FromPyObject<'_> for PipelineSyncData {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let json = Json::extract(ob)?;
        Ok(json.into())
    }
}

impl FromPyObject<'_> for TransformerStream {
    fn extract(_ob: &PyAny) -> PyResult<Self> {
        panic!("We must implement this, but this is impossible to be reached")
    }
}

////////////////////////////////////////////////////////////////////////////////
// Rust to Rust //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

impl CustomInto<Json> for PipelineSyncData {
    fn custom_into(self) -> Json {
        Json::from(self)
    }
}
