use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn _rustlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "awolverp")?;
    Ok(())
}
