use pyo3::prelude::*;

extern crate matching;
extern crate treedom;

mod dom;
mod tools;

#[pymodule(gil_used = false)]
#[cold]
fn _rustlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("QUIRKS_MODE_FULL", tools::QUIRKS_MODE_FULL)?;
    m.add("QUIRKS_MODE_LIMITED", tools::QUIRKS_MODE_LIMITED)?;
    m.add("QUIRKS_MODE_OFF", tools::QUIRKS_MODE_OFF)?;

    m.add_class::<dom::PyQualName>()?;
    m.add_class::<dom::PyHtmlOptions>()?;
    m.add_class::<dom::PyXmlOptions>()?;
    m.add_class::<dom::PyParser>()?;
    m.add_class::<dom::PyTreeDom>()?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "awolverp")?;
    Ok(())
}
