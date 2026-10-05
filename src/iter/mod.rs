use pyo3::types::{PyAnyMethods, PyModuleMethods};

mod iterator;
mod traverse;

pub use traverse::PyTraverse;

#[cold]
pub fn register_iter_module(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    let iter_module = pyo3::types::PyModule::new(m.py(), "iter")?;

    iter_module.add_class::<iterator::PyIterator>()?;
    iter_module.add_class::<iterator::PyAncestors>()?;
    iter_module.add_class::<iterator::PyPrevSiblings>()?;
    iter_module.add_class::<iterator::PyNextSiblings>()?;
    iter_module.add_class::<iterator::PyFirstChildren>()?;
    iter_module.add_class::<iterator::PyLastChildren>()?;
    iter_module.add_class::<iterator::PyChildren>()?;

    iter_module.add_class::<traverse::PyTraverse>()?;
    iter_module.add_class::<traverse::PyDescendants>()?;

    m.add_submodule(&iter_module)?;

    // Register it like a submodule of a package, so that `import markupever._rustlib.iter`
    // works.
    let name = format!("{}.iter", m.name()?);
    iter_module.setattr("__name__", &name)?;
    m.py()
        .import("sys")?
        .getattr("modules")?
        .set_item(name, iter_module)
}
