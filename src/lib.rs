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

#[pyfunction]
fn _is_node_impl(object: &pyo3::Bound<'_, pyo3::PyAny>) -> bool {
    use pyo3::type_object::PyTypeInfo;

    nodes::PyDocument::is_exact_type_of(object)
        || nodes::PyDoctype::is_exact_type_of(object)
        || nodes::PyComment::is_exact_type_of(object)
        || nodes::PyText::is_exact_type_of(object)
        || nodes::PyElement::is_exact_type_of(object)
        || nodes::PyProcessingInstruction::is_exact_type_of(object)
}

#[pymodule(gil_used = false)]
mod _rustlib {
    use pyo3::types::PyModuleMethods;
    use pyo3::PyResult;

    #[pymodule_export]
    use crate::qualname::PyQualName;

    #[pymodule_export]
    use crate::parser::{PyHtmlOptions, PyParser, PyXmlOptions};

    #[pymodule_export]
    use crate::tree::PyTreeDom;

    #[pymodule_export]
    use crate::nodes::{
        PyAttrsList, PyAttrsListItems, PyComment, PyDoctype, PyDocument, PyElement,
        PyProcessingInstruction, PyText,
    };

    #[pymodule_export]
    use crate::select::PySelect;

    #[pymodule_export]
    use crate::parser::serialize;

    #[pymodule_export]
    use crate::_is_node_impl;

    #[pymodule_export]
    const QUIRKS_MODE_FULL: u8 = crate::tools::QUIRKS_MODE_FULL;

    #[pymodule_export]
    const QUIRKS_MODE_LIMITED: u8 = crate::tools::QUIRKS_MODE_LIMITED;

    #[pymodule_export]
    const QUIRKS_MODE_OFF: u8 = crate::tools::QUIRKS_MODE_OFF;

    #[pymodule_init]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
        m.add("__version__", env!("CARGO_PKG_VERSION"))?;
        m.add("__author__", "awolverp")?;

        crate::iter::register_iter_module(m)?;

        Ok(())
    }
}
