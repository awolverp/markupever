use pyo3::prelude::*;

pub mod bridge;
pub mod core;

/// A Python module implemented in Rust.
#[pymodule(gil_used = false)]
fn _rustlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "awolverp")?;

    m.add_class::<bridge::PyHtml>()?;
    m.add_class::<bridge::PyXml>()?;
    m.add("QUIRKS_MODE_FULL", bridge::QUIRKS_MODE_FULL)?;
    m.add("QUIRKS_MODE_LIMITED", bridge::QUIRKS_MODE_LIMITED)?;
    m.add("QUIRKS_MODE_OFF", bridge::QUIRKS_MODE_OFF)?;

    m.add_class::<bridge::PyNode>()?;
    m.add_class::<bridge::PyCommentData>()?;
    m.add_class::<bridge::PyDoctypeData>()?;
    m.add_class::<bridge::PyDocumentData>()?;
    m.add_class::<bridge::PyFragmentData>()?;
    m.add_class::<bridge::PyProcessingInstructionData>()?;
    m.add_class::<bridge::PyQualName>()?;
    m.add_class::<bridge::PyTextData>()?;
    m.add_class::<bridge::PyElementAttributes>()?;
    m.add_class::<bridge::PyElementData>()?;
    m.add_class::<bridge::PyChildren>()?;

    Ok(())
}
