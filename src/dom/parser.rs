//! **Parser**:
//! - new
//! - errors
//! - quirks_mode
//! - namespaces
//! - lineno
//! - dom
//!
//! **HtmlOptions**
//!
//! **XmlOptions**
use std::sync::Arc;

/// These are options for HTML parsing
#[pyo3::pyclass(name = "HtmlOptions", module = "markupselect._rustlib", frozen)]
pub struct PyHtmlOptions {
    /// Report all parse errors described in the spec, at some
    /// performance penalty?  Default: false
    exact_errors: bool,

    /// Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning
    /// of the stream?  Default: true
    discard_bom: bool,

    /// Keep a record of how long we spent in each state?  Printed
    /// when `end()` is called.  Default: false
    profile: bool,

    /// Is this an `iframe srcdoc` document?
    iframe_srcdoc: bool,

    /// Should we drop the DOCTYPE (if any) from the tree?
    drop_doctype: bool,

    /// Is this a complete document? (means includes html, head, and body tag)
    /// Default: true
    full_document: bool,

    /// Initial TreeBuilder quirks mode. Default: NoQuirks
    quirks_mode: treedom::markup5ever::interface::QuirksMode,
}

#[pyo3::pymethods]
impl PyHtmlOptions {
    #[new]
    #[pyo3(signature=(full_document=true, exact_errors=false, discard_bom=true, profile=false, iframe_srcdoc=false, drop_doctype=false, quirks_mode=crate::tools::QUIRKS_MODE_OFF))]
    fn new(
        full_document: bool,
        exact_errors: bool,
        discard_bom: bool,
        profile: bool,
        iframe_srcdoc: bool,
        drop_doctype: bool,
        quirks_mode: u8,
    ) -> pyo3::PyResult<Self> {
        let quirks_mode =
            crate::tools::convert_u8_to_quirks_mode(quirks_mode).ok_or_else(|| {
                pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "quirks_mode must be between 0 and 2, got {}",
                    quirks_mode
                ))
            })?;

        Ok(Self {
            exact_errors,
            discard_bom,
            profile,
            iframe_srcdoc,
            drop_doctype,
            full_document,
            quirks_mode,
        })
    }

    #[getter]
    fn quirks_mode(&self) -> u8 {
        crate::tools::convert_quirks_mode_to_u8(self.quirks_mode)
    }

    #[getter]
    fn exact_errors(&self) -> bool {
        self.exact_errors
    }

    #[getter]
    fn discard_bom(&self) -> bool {
        self.discard_bom
    }

    #[getter]
    fn profile(&self) -> bool {
        self.profile
    }

    #[getter]
    fn iframe_srcdoc(&self) -> bool {
        self.iframe_srcdoc
    }

    #[getter]
    fn drop_doctype(&self) -> bool {
        self.drop_doctype
    }

    #[getter]
    fn full_document(&self) -> bool {
        self.full_document
    }

    fn __repr__(&self) -> String {
        format!(
            "xmarkup._rustlib.HtmlOptions(full_document={}, exact_errors={}, discard_bom={}, profile={}, iframe_srcdoc={}, drop_doctype={}, quirks_mode={})",
            self.full_document,
            self.exact_errors,
            self.discard_bom,
            self.profile,
            self.iframe_srcdoc,
            self.drop_doctype,
            crate::tools::convert_quirks_mode_to_u8(self.quirks_mode),
        )
    }
}

#[pyo3::pyclass(name = "XmlOptions", module = "markupselect._rustlib", frozen)]
pub struct PyXmlOptions {
    /// Report all parse errors described in the spec, at some
    /// performance penalty?  Default: false
    exact_errors: bool,

    /// Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning
    /// of the stream?  Default: true
    discard_bom: bool,

    /// Keep a record of how long we spent in each state?  Printed
    /// when `end()` is called.  Default: false
    profile: bool,
}

#[pyo3::pymethods]
impl PyXmlOptions {
    #[new]
    #[pyo3(signature=(exact_errors=false, discard_bom=true, profile=false))]
    pub(super) fn new(exact_errors: bool, discard_bom: bool, profile: bool) -> Self {
        Self {
            exact_errors,
            discard_bom,
            profile,
        }
    }

    #[getter]
    fn exact_errors(&self) -> bool {
        self.exact_errors
    }

    #[getter]
    fn discard_bom(&self) -> bool {
        self.discard_bom
    }

    #[getter]
    fn profile(&self) -> bool {
        self.profile
    }

    fn __repr__(&self) -> String {
        format!(
            "xmarkup._rustlib.XmlOptions(exact_errors={}, discard_bom={}, profile={})",
            self.exact_errors,
            self.discard_bom,
            self.profile,
        )
    }
}
