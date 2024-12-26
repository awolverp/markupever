use crate::core::arcdom::{
    parse_html_utf8, parse_xml_utf8, serialize_html, serialize_xml, TreeBuilder,
};

fn quirks_mode_from_u8(value: u8) -> markup5ever::interface::QuirksMode {
    match value {
        0 => markup5ever::interface::QuirksMode::Quirks,
        1 => markup5ever::interface::QuirksMode::LimitedQuirks,
        _ => markup5ever::interface::QuirksMode::NoQuirks,
    }
}

fn quirks_mode_to_u8(value: markup5ever::interface::QuirksMode) -> u8 {
    match value {
        markup5ever::interface::QuirksMode::Quirks => 0,
        markup5ever::interface::QuirksMode::LimitedQuirks => 1,
        markup5ever::interface::QuirksMode::NoQuirks => 2,
    }
}

/// HTML Tree / HTML Document Parser
///
/// Parses a HTML document into a tree link of `Node`s
#[pyo3::pyclass(name = "Html", module = "markupselect._rustlib", frozen)]
pub struct PyHtml(TreeBuilder);

#[pyo3::pymethods]
impl PyHtml {
    #[new]
    #[pyo3(signature=(content, quirks_mode, exact_errors=false, is_fragment=false, /))]
    pub fn new(content: Vec<u8>, quirks_mode: u8, exact_errors: bool, is_fragment: bool) -> Self {
        let dom = parse_html_utf8(
            content.as_slice(),
            quirks_mode_from_u8(quirks_mode),
            exact_errors,
            is_fragment,
        );

        Self(dom)
    }

    /// Returns a list of errors with its line
    #[getter]
    pub fn errors(&self) -> Vec<(String, u64)> {
        self.0
            .errors()
            .borrow()
            .iter()
            .map(|e| (e.0.to_string(), e.1))
            .collect()
    }

    /// Returns the line count of the parsed document
    #[getter]
    pub fn lineno(&self) -> u64 {
        unsafe { *self.0.lineno().get() }
    }

    /// Returns the quirks mode
    #[getter]
    pub fn quirks_mode(&self) -> u8 {
        quirks_mode_to_u8(self.0.quirks_mode().get())
    }

    // /// Returns the root node
    // #[getter]
    // pub fn root(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
    //     let node = PyNode(self.0.root.clone());
    //     pyo3::Py::new(py, node).map(|x| x.into_any())
    // }

    /// Serialize the content into bytes
    pub fn serialize(&self) -> pyo3::PyResult<Vec<u8>> {
        let mut writer = Vec::new();

        serialize_html(&mut writer, &self.0.root)
            .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(x.to_string()))?;

        Ok(writer)
    }

    // pub fn __repr__(&self) -> String {
    //     self.to_string()
    // }
}

/// HTML Tree / HTML Document Parser
///
/// Parses a HTML document into a tree link of `Node`s
#[pyo3::pyclass(name = "Xml", module = "markupselect._rustlib", frozen)]
pub struct PyXml(TreeBuilder);

#[pyo3::pymethods]
impl PyXml {
    #[new]
    #[pyo3(signature=(content, exact_errors=false, /))]
    pub fn new(content: Vec<u8>, exact_errors: bool) -> Self {
        let dom = parse_xml_utf8(content.as_slice(), exact_errors);

        Self(dom)
    }

    /// Returns a list of errors with its line
    #[getter]
    pub fn errors(&self) -> Vec<(String, u64)> {
        self.0
            .errors()
            .borrow()
            .iter()
            .map(|e| (e.0.to_string(), e.1))
            .collect()
    }

    /// Returns the line count of the parsed document
    #[getter]
    pub fn lineno(&self) -> u64 {
        unsafe { *self.0.lineno().get() }
    }

    /// Returns the quirks mode
    #[getter]
    pub fn quirks_mode(&self) -> u8 {
        quirks_mode_to_u8(self.0.quirks_mode().get())
    }

    // /// Returns the root node
    // #[getter]
    // pub fn root(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
    //     let node = PyNode(self.0.root.clone());
    //     pyo3::Py::new(py, node).map(|x| x.into_any())
    // }

    pub fn serialize(&self) -> pyo3::PyResult<Vec<u8>> {
        let mut writer = Vec::new();

        serialize_xml(&mut writer, &self.0.root)
            .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(x.to_string()))?;

        Ok(writer)
    }

    // pub fn __repr__(&self) -> String {
    //     self.to_string()
    // }
}
