use pyo3::prelude::*;

extern crate treedom;
extern crate matching;

mod tools;
mod dom;

#[pymodule(gil_used = false)]
#[cold]
fn _rustlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<dom::PyQualName>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "awolverp")?;
    Ok(())
}
