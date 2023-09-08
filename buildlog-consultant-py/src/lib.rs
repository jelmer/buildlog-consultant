use pyo3::exceptions::PyNotImplementedError;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use std::collections::HashMap;

#[pyclass]
struct Match(Box<dyn buildlog_consultant::Match>);

#[pymethods]
impl Match {
    #[getter]
    fn line(&self) -> String {
        self.0.line().to_string()
    }

    #[getter]
    fn offset(&self) -> usize {
        self.0.offset()
    }

    #[getter]
    fn origin(&self) -> String {
        self.0.origin().to_string()
    }

    #[getter]
    fn lineno(&self) -> usize {
        self.0.lineno()
    }

    fn __repr__(&self) -> String {
        format!(
            "Match({:?}, {}, {})",
            self.0.line(),
            self.0.lineno(),
            self.0.lineno()
        )
    }
}

#[pyclass]
struct Problem(Box<dyn buildlog_consultant::Problem>);

fn json_to_py(py: Python, json: serde_json::Value) -> PyResult<PyObject> {
    match json {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.into_py(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(u) = n.as_u64() {
                Ok(u.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Invalid number",
                ))
            }
        }
        serde_json::Value::String(s) => Ok(s.into_py(py)),
        serde_json::Value::Array(a) => {
            let mut ret = Vec::with_capacity(a.len());
            for v in a {
                ret.push(json_to_py(py, v)?);
            }
            Ok(ret.into_py(py))
        }
        serde_json::Value::Object(o) => {
            let mut ret = pyo3::types::PyDict::new(py);
            for (k, v) in o {
                ret.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(ret.into())
        }
    }
}

#[pymethods]
impl Problem {
    #[getter]
    fn kind(&self) -> String {
        self.0.kind().to_string()
    }

    fn __repr__(&self) -> String {
        format!("Problem({:?}, {})", self.0.kind(), self.0.json())
    }

    fn json(&self, py: Python) -> PyResult<PyObject> {
        json_to_py(py, self.0.json())
    }

    fn __richcmp__(&self, other: PyRef<Problem>, op: CompareOp) -> PyResult<bool> {
        let s = self.0.json();
        let o = other.0.json();
        match op {
            CompareOp::Eq => Ok(self.0.kind() == other.0.kind() && s == o),
            CompareOp::Ne => Ok(self.0.kind() != other.0.kind() || s != o),
            _ => Err(PyNotImplementedError::new_err(
                "Only == and != are implemented",
            )),
        }
    }
}

#[pyfunction]
fn match_lines(lines: Vec<&str>, offset: usize) -> PyResult<(Option<Match>, Option<Problem>)> {
    let ret = buildlog_consultant::common::match_lines(lines.as_slice(), offset)
        .map_err(|e| pyo3::exceptions::PyException::new_err(format!("Error: {}", e)))?;

    if let Some((m, p)) = ret {
        Ok((Some(Match(m)), p.map(|p| Some(Problem(p))).unwrap_or(None)))
    } else {
        Ok((None, None))
    }
}

#[pymodule]
fn _buildlog_consultant_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Match>()?;
    m.add_class::<Problem>()?;
    m.add_function(wrap_pyfunction!(match_lines, m)?)?;
    Ok(())
}
