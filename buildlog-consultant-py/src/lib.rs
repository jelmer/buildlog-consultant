use pyo3::exceptions::PyNotImplementedError;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;

use std::io::BufReader;

#[pyclass]
struct Match(Box<dyn buildlog_consultant::Match>);

#[pymethods]
impl Match {
    #[getter]
    fn line(&self) -> String {
        self.0.line()
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

    #[getter]
    fn linenos(&self) -> Vec<usize> {
        self.0.linenos().to_vec()
    }

    #[getter]
    fn lines(&self) -> Vec<String> {
        self.0.lines().to_vec()
    }

    #[getter]
    fn offsets(&self) -> Vec<usize> {
        self.0.offsets().to_vec()
    }

    fn __richcmp__(&self, other: PyRef<Match>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.0.offsets() == other.0.offsets() && self.line() == other.line()),
            CompareOp::Ne => Ok(self.0.offsets() != other.0.offsets() || self.line() != other.line()),
            _ => Err(PyNotImplementedError::new_err("Only == and != are implemented")),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Match({:?}, {}, {})",
            self.0.line(),
            self.0.offset(),
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
            let ret = pyo3::types::PyDict::new_bound(py);
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
fn find_autopkgtest_failure_description(
    lines: Vec<String>,
) -> (Option<Match>, Option<String>, Option<Problem>, Option<String>) {
    let (m, t, p, d)= buildlog_consultant::autopkgtest::find_autopkgtest_failure_description(lines.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    (m.map(Match), t, p.map(Problem), d)
}


#[pyfunction]
fn match_lines(lines: Vec<String>, offset: usize) -> PyResult<(Option<Match>, Option<Problem>)> {
    let lines = lines.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    if offset >= lines.len() {
        return Err(pyo3::exceptions::PyIndexError::new_err(
            "Offset out of range",
        ));
    }
    let ret = buildlog_consultant::common::match_lines(lines.as_slice(), offset)
        .map_err(|e| pyo3::exceptions::PyException::new_err(format!("Error: {}", e)))?;

    if let Some((m, p)) = ret {
        Ok((Some(Match(m)), p.map(|p| Some(Problem(p))).unwrap_or(None)))
    } else {
        Ok((None, None))
    }
}

#[pyclass]
struct SbuildLogSection(buildlog_consultant::sbuild::SbuildLogSection);

#[pymethods]
impl SbuildLogSection {
    #[getter]
    fn title(&self) -> Option<String> {
        self.0.title.clone()
    }

    #[getter]
    fn offsets(&self) -> (usize, usize) {
        self.0.offsets
    }

    #[getter]
    fn lines(&self) -> Vec<String> {
        self.0.lines.clone()
    }
}

#[pyclass]
struct SbuildLog(buildlog_consultant::sbuild::SbuildLog);

#[pymethods]
impl SbuildLog {
    fn get_failed_stage(&self) -> Option<String> {
        self.0.get_failed_stage()
    }

    #[pyo3(signature = (section=None))]
    fn get_section_lines(&self, section: Option<&str>) -> Option<Vec<String>> {
        self.0
            .get_section_lines(section)
            .map(|v| v.into_iter().map(|s| s.to_string()).collect())
    }

    #[getter]
    fn sections(&self) -> Vec<SbuildLogSection> {
        self.0
            .sections()
            .map(|s| SbuildLogSection(s.clone()))
            .collect()
    }

    fn section_titles(&self) -> Vec<String> {
        self.0
            .section_titles()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[pyo3(signature = (section=None))]
    fn get_section(&self, section: Option<&str>) -> Option<SbuildLogSection> {
        self.0
            .get_section(section)
            .map(|s| SbuildLogSection(s.clone()))
    }

    #[staticmethod]
    fn parse(f: PyObject) -> PyResult<SbuildLog> {
        let f = pyo3_filelike::PyBinaryFile::from(f);
        let bufread = BufReader::new(f);
        Ok(SbuildLog(buildlog_consultant::sbuild::SbuildLog(
            buildlog_consultant::sbuild::parse_sbuild_log(bufread).collect(),
        )))
    }
}

#[pyfunction]
fn parse_sbuild_log(lines: Vec<Vec<u8>>) -> PyResult<Vec<SbuildLogSection>> {
    let text = lines.concat();
    let cursor = std::io::Cursor::new(text);
    let mut ret = Vec::new();
    let sections = buildlog_consultant::sbuild::parse_sbuild_log(cursor);
    for section in sections {
        ret.push(SbuildLogSection(section));
    }
    Ok(ret)
}

#[pyfunction]
fn find_secondary_build_failure(lines: Vec<String>, offset: usize) -> Option<Match> {
    let lines = lines.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    buildlog_consultant::common::find_secondary_build_failure(lines.as_slice(), offset)
        .map(|m| Match(Box::new(m)))
}

#[pymodule]
fn _buildlog_consultant_rs(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    pyo3_log::init();
    m.add_class::<Match>()?;
    m.add_class::<Problem>()?;
    m.add_class::<SbuildLogSection>()?;
    m.add_class::<SbuildLog>()?;
    m.add_function(wrap_pyfunction!(match_lines, m)?)?;
    m.add_function(wrap_pyfunction!(parse_sbuild_log, m)?)?;
    m.add_function(wrap_pyfunction!(find_secondary_build_failure, m)?)?;
    m.add_function(wrap_pyfunction!(find_autopkgtest_failure_description, m)?)?;
    Ok(())
}
