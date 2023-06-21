use pyo3::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

pub trait Match: Send + Sync + std::fmt::Debug {
    fn line(&self) -> String;

    fn origin(&self) -> Origin;

    fn offset(&self) -> usize;

    fn lineno(&self) -> usize {
        self.offset() + 1
    }
}

#[derive(Clone, Debug)]
pub struct Origin(String);

impl ToString for Origin {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub struct SingleLineMatch {
    pub origin: Origin,
    pub offset: usize,
    pub line: String,
}

impl Match for SingleLineMatch {
    fn line(&self) -> String {
        self.line.clone()
    }

    fn origin(&self) -> Origin {
        self.origin.clone()
    }

    fn offset(&self) -> usize {
        self.offset
    }
}

impl std::fmt::Debug for SingleLineMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.origin.0, self.lineno(), self.line)
    }
}

pub struct MultiLineMatch {
    pub origin: Origin,
    pub offsets: Vec<usize>,
    pub lines: Vec<String>,
}

impl MultiLineMatch {
    pub fn new(origin: Origin, offsets: Vec<usize>, lines: Vec<String>) -> Self {
        assert!(!offsets.is_empty());
        assert!(offsets.len() == lines.len());
        Self {
            origin,
            offsets,
            lines,
        }
    }
}

impl Match for MultiLineMatch {
    fn line(&self) -> String {
        self.lines[0].clone()
    }

    fn origin(&self) -> Origin {
        self.origin.clone()
    }

    fn offset(&self) -> usize {
        self.offsets[0]
    }

    fn lineno(&self) -> usize {
        self.offset() + 1
    }
}

impl std::fmt::Debug for MultiLineMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.origin.0, self.lineno(), self.line())
    }
}

pub trait Problem: std::fmt::Display + Send + Sync {
    fn kind(&self) -> Cow<str>;

    fn json(&self) -> serde_json::Value;
}

pub struct PyMatch(PyObject);

impl Match for PyMatch {
    fn line(&self) -> String {
        Python::with_gil(|py| {
            let line = self.0.getattr(py, "line").unwrap();
            line.extract::<String>(py).unwrap()
        })
    }

    fn origin(&self) -> Origin {
        Python::with_gil(|py| {
            let origin = self.0.getattr(py, "origin").unwrap();
            let origin = origin.extract::<String>(py).unwrap();
            Origin(origin)
        })
    }

    fn offset(&self) -> usize {
        Python::with_gil(|py| {
            let offset = self.0.getattr(py, "offset").unwrap();
            offset.extract::<usize>(py).unwrap()
        })
    }
}

impl std::fmt::Debug for PyMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Python::with_gil(|py| {
            let s = self
                .0
                .call_method0(py, "__repr__")
                .unwrap()
                .extract::<String>(py)
                .unwrap();
            write!(f, "{}", s)
        })
    }
}

impl PartialEq for PyMatch {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| {
            let eq = self
                .0
                .call_method1(py, "__eq__", (other.0.clone(),))
                .unwrap();
            eq.extract::<bool>(py).unwrap()
        })
    }
}

#[derive(Clone, Debug)]
pub struct PyProblem(PyObject);

fn py_to_json(py: Python, json: PyObject) -> PyResult<serde_json::Value> {
    if let Ok(s) = json.extract::<String>(py) {
        Ok(serde_json::Value::String(s))
    } else if let Ok(n) = json.extract::<i64>(py) {
        Ok(serde_json::Value::Number(n.into()))
    } else if let Ok(b) = json.extract::<bool>(py) {
        Ok(serde_json::Value::Bool(b))
    } else if let Ok(l) = json.extract::<Vec<PyObject>>(py) {
        let mut v = Vec::new();
        for x in l {
            v.push(py_to_json(py, x)?);
        }
        Ok(serde_json::Value::Array(v))
    } else if let Ok(d) = json.extract::<HashMap<String, PyObject>>(py) {
        let mut m = serde_json::Map::new();
        for (k, v) in d {
            let v = py_to_json(py, v)?;
            m.insert(k, v);
        }
        Ok(serde_json::Value::Object(m))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "unsupported type",
        ))
    }
}

impl Problem for PyProblem {
    fn kind(&self) -> Cow<str> {
        Python::with_gil(|py| {
            let kind = self.0.getattr(py, "kind").unwrap();
            kind.extract::<String>(py).unwrap().into()
        })
    }

    fn json(&self) -> serde_json::Value {
        Python::with_gil(|py| {
            let json = self.0.getattr(py, "json").unwrap();
            py_to_json(py, json).unwrap()
        })
    }
}

impl PartialEq for PyProblem {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| {
            let eq = self
                .0
                .call_method1(py, "__eq__", (other.0.clone(),))
                .unwrap();
            eq.extract::<bool>(py).unwrap()
        })
    }
}

impl std::fmt::Display for PyProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Python::with_gil(|py| {
            let s = self
                .0
                .call_method0(py, "__str__")
                .unwrap()
                .extract::<String>(py)
                .unwrap();
            write!(f, "{}", s)
        })
    }
}

pub fn find_build_failure_description(
    lines: &[&str],
) -> (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) {
    Python::with_gil(|py| {
        let module = py.import("buildlog_consultant.common").unwrap();
        let find_build_failure_description =
            module.getattr("find_build_failure_description").unwrap();
        let result = find_build_failure_description
            .call1((lines.to_vec(),))
            .unwrap();
        let (m, p) = result
            .extract::<(Option<PyObject>, Option<PyObject>)>()
            .unwrap();
        (
            m.map(|m| Box::new(PyMatch(m)) as Box<dyn Match>),
            p.map(|p| Box::new(PyProblem(p)) as Box<dyn Problem>),
        )
    })
}

#[cfg(test)]
mod test {
    #[test]
    fn test_simple() {
        let (m, p) = super::find_build_failure_description(&[
            "make[1]: *** No rule to make target 'nno.autopgen.bin', needed by 'dan-nno.autopgen.bin'.  Stop."]);
        assert!(m.is_some());
        assert!(p.is_some());
    }
}

pub mod common;

pub mod r#match;
