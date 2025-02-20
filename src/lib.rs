use pyo3::prelude::*;

extern crate matching;
extern crate treedom;

mod iter;
mod nodes;
mod parser;
mod qualname;
mod select;
mod tools;
mod tree;

#[pymodule(gil_used = false)]
#[cold]
fn _rustlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("QUIRKS_MODE_FULL", tools::QUIRKS_MODE_FULL)?;
    m.add("QUIRKS_MODE_LIMITED", tools::QUIRKS_MODE_LIMITED)?;
    m.add("QUIRKS_MODE_OFF", tools::QUIRKS_MODE_OFF)?;

    m.add_class::<qualname::PyQualName>()?;
    m.add_class::<parser::PyHtmlOptions>()?;
    m.add_class::<parser::PyXmlOptions>()?;
    m.add_class::<parser::PyParser>()?;
    m.add_class::<tree::PyTreeDom>()?;

    m.add_class::<nodes::PyAttrsList>()?;
    m.add_class::<nodes::PyAttrsListItems>()?;
    m.add_class::<nodes::PyComment>()?;
    m.add_class::<nodes::PyDoctype>()?;
    m.add_class::<nodes::PyDocument>()?;
    m.add_class::<nodes::PyElement>()?;
    m.add_class::<nodes::PyProcessingInstruction>()?;
    m.add_class::<nodes::PyText>()?;

    m.add_class::<select::PySelect>()?;

    m.add_function(wrap_pyfunction!(parser::serialize, m)?)?;

    iter::register_iter_module(m)?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "awolverp")?;
    Ok(())
}
