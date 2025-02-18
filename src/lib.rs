use pyo3::prelude::*;

extern crate matching;
extern crate treedom;

mod core;
mod tools;

#[pymodule(gil_used = false)]
#[cold]
fn _rustlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("QUIRKS_MODE_FULL", tools::QUIRKS_MODE_FULL)?;
    m.add("QUIRKS_MODE_LIMITED", tools::QUIRKS_MODE_LIMITED)?;
    m.add("QUIRKS_MODE_OFF", tools::QUIRKS_MODE_OFF)?;

    m.add_class::<core::PyQualName>()?;
    m.add_class::<core::PyHtmlOptions>()?;
    m.add_class::<core::PyXmlOptions>()?;
    m.add_class::<core::PyParser>()?;
    m.add_class::<core::PyTreeDom>()?;

    m.add_class::<core::PyAttrsList>()?;
    m.add_class::<core::PyAttrsListItems>()?;
    m.add_class::<core::PyComment>()?;
    m.add_class::<core::PyDoctype>()?;
    m.add_class::<core::PyDocument>()?;
    m.add_class::<core::PyElement>()?;
    m.add_class::<core::PyProcessingInstruction>()?;
    m.add_class::<core::PyText>()?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "awolverp")?;
    Ok(())
}
