use pyo3::prelude::*;

pub mod core;
pub mod builder;

/// A Python module implemented in Rust.
#[pymodule]
fn _rustlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "awolverp")?;

    m.add_class::<builder::PyHtml>()?;
    m.add_class::<builder::PyXml>()?;

    Ok(())
}
