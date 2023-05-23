use pyo3::prelude::*;

pub trait Match {}

pub trait Problem {}

impl Match for PyObject {}

impl Problem for PyObject {}

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
            m.map(|m| Box::new(m) as Box<dyn Match>),
            p.map(|p| Box::new(p) as Box<dyn Problem>),
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
