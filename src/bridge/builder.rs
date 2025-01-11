use super::utils::get_node_from_pyobject;
use crate::core::arcdom;
use crate::core::matching;
use pyo3::types::PyAnyMethods;

pub const QUIRKS_MODE_FULL: u8 = 0;
pub const QUIRKS_MODE_LIMITED: u8 = 1;
pub const QUIRKS_MODE_OFF: u8 = 2;

fn quirks_mode_from_u8(value: u8) -> markup5ever::interface::QuirksMode {
    match value {
        QUIRKS_MODE_FULL => markup5ever::interface::QuirksMode::Quirks,
        QUIRKS_MODE_LIMITED => markup5ever::interface::QuirksMode::LimitedQuirks,
        _ => markup5ever::interface::QuirksMode::NoQuirks,
    }
}

fn quirks_mode_to_u8(value: markup5ever::interface::QuirksMode) -> u8 {
    match value {
        markup5ever::interface::QuirksMode::Quirks => QUIRKS_MODE_FULL,
        markup5ever::interface::QuirksMode::LimitedQuirks => QUIRKS_MODE_LIMITED,
        markup5ever::interface::QuirksMode::NoQuirks => QUIRKS_MODE_OFF,
    }
}

/// These are options for HTML parsing
#[pyo3::pyclass(name = "RawHtmlOptions", module = "markupselect._rustlib", frozen)]
pub struct PyRawHtmlOptions {
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
    quirks_mode: markup5ever::interface::QuirksMode,
}

#[pyo3::pymethods]
impl PyRawHtmlOptions {
    #[new]
    #[pyo3(signature=(full_document=true, *, exact_errors=false, discard_bom=true, profile=false, iframe_srcdoc=false, drop_doctype=false, quirks_mode=QUIRKS_MODE_OFF))]
    fn new(
        full_document: bool,
        exact_errors: bool,
        discard_bom: bool,
        profile: bool,
        iframe_srcdoc: bool,
        drop_doctype: bool,
        quirks_mode: u8,
    ) -> Self {
        Self {
            exact_errors,
            discard_bom,
            profile,
            iframe_srcdoc,
            drop_doctype,
            full_document,
            quirks_mode: quirks_mode_from_u8(quirks_mode),
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

    #[getter]
    fn quirks_mode(&self) -> u8 {
        quirks_mode_to_u8(self.quirks_mode)
    }
}

#[pyo3::pyclass(name = "RawXmlOptions", module = "markupselect._rustlib", frozen)]
pub struct PyRawXmlOptions {
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
impl PyRawXmlOptions {
    #[new]
    #[pyo3(signature=(*, exact_errors=false, discard_bom=true, profile=false))]
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
}

/// HTML Tree / HTML Document Parser
///
/// Parses a HTML document into a tree link of `Node`s
#[pyo3::pyclass(name = "RawHtml", module = "markupselect._rustlib", frozen)]
pub struct PyRawHtml(arcdom::ArcDom);

#[pyo3::pymethods]
impl PyRawHtml {
    #[new]
    pub(super) fn new(
        py: pyo3::Python<'_>,
        content: pyo3::PyObject,
        options: pyo3::PyObject,
    ) -> pyo3::PyResult<Self> {
        use tendril::TendrilSink;

        let options = options
            .bind(py)
            .extract::<pyo3::PyRef<'_, PyRawHtmlOptions>>()
            .map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "expected RawHtmlOptions for options argument",
                )
            })?;

        let content = unsafe {
            if pyo3::ffi::PyBytes_Check(content.as_ptr()) == 1 {
                content.bind(py).extract::<Vec<u8>>().unwrap()
            } else if pyo3::ffi::PyUnicode_Check(content.as_ptr()) == 1 {
                let s = content.bind(py).extract::<String>().unwrap();
                s.into_bytes()
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "expected bytes or str for content argument",
                ));
            }
        };

        let parser = arcdom::ArcDom::parse_html(
            arcdom::Node::new(arcdom::DocumentData),
            options.full_document,
            html5ever::tokenizer::TokenizerOpts {
                exact_errors: options.exact_errors,
                discard_bom: options.discard_bom,
                profile: options.profile,
                ..Default::default()
            },
            html5ever::tree_builder::TreeBuilderOpts {
                exact_errors: options.exact_errors,
                iframe_srcdoc: options.iframe_srcdoc,
                drop_doctype: options.drop_doctype,
                quirks_mode: options.quirks_mode,
                ..Default::default()
            },
        )
        .from_utf8()
        .one(tendril::ByteTendril::from_slice(&content));

        Ok(Self(parser))
    }

    /// Returns a list of errors
    #[getter]
    pub(super) fn errors(&self) -> Vec<String> {
        self.0
            .errors
            .borrow()
            .iter()
            .map(|x| x.to_string())
            .collect()
    }

    /// Returns the quirks mode
    #[getter]
    pub(super) fn quirks_mode(&self) -> u8 {
        quirks_mode_to_u8(self.0.quirks_mode.get())
    }

    /// Returns the root node
    ///
    /// Most of the time is document node
    #[getter]
    pub(super) fn root(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = super::node::PyRawNode(self.0.root.clone());
        pyo3::Py::new(py, node).map(|x| x.into_any())
    }
}

/// HTML Tree / HTML Document Parser
///
/// Parses a HTML document into a tree link of `Node`s
#[pyo3::pyclass(name = "RawXml", module = "markupselect._rustlib", frozen)]
pub struct PyRawXml(arcdom::ArcDom);

#[pyo3::pymethods]
impl PyRawXml {
    #[new]
    pub(super) fn new(
        py: pyo3::Python<'_>,
        content: pyo3::PyObject,
        options: pyo3::PyObject,
    ) -> pyo3::PyResult<Self> {
        use tendril::TendrilSink;

        let options = options
            .bind(py)
            .extract::<pyo3::PyRef<'_, PyRawXmlOptions>>()
            .map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "expected RawXmlOptions for options argument",
                )
            })?;

        let content = unsafe {
            if pyo3::ffi::PyBytes_Check(content.as_ptr()) == 1 {
                content.bind(py).extract::<Vec<u8>>().unwrap()
            } else if pyo3::ffi::PyUnicode_Check(content.as_ptr()) == 1 {
                let s = content.bind(py).extract::<String>().unwrap();
                s.into_bytes()
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "expected bytes or str for content argument",
                ));
            }
        };

        let parser = arcdom::ArcDom::parse_xml(
            arcdom::Node::new(arcdom::DocumentData),
            xml5ever::tokenizer::XmlTokenizerOpts {
                exact_errors: options.exact_errors,
                discard_bom: options.discard_bom,
                profile: options.profile,
                ..Default::default()
            },
        )
        .from_utf8()
        .one(tendril::ByteTendril::from_slice(&content));

        Ok(Self(parser))
    }

    /// Returns a list of errors
    #[getter]
    pub(super) fn errors(&self) -> Vec<String> {
        self.0
            .errors
            .borrow()
            .iter()
            .map(|x| x.to_string())
            .collect()
    }

    /// Returns the root node
    ///
    /// Most of the time is document node
    #[getter]
    pub(super) fn root(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = super::node::PyRawNode(self.0.root.clone());
        pyo3::Py::new(py, node).map(|x| x.into_any())
    }
}

#[pyo3::pyclass(name = "RawMatching", module = "markupselect._rustlib")]
pub struct PyRawMatching(pub matching::Select);

#[pyo3::pymethods]
impl PyRawMatching {
    #[new]
    #[pyo3(signature=(node, expr, parser=None))]
    pub(super) fn new(
        py: pyo3::Python<'_>,
        node: pyo3::PyObject,
        expr: String,
        parser: Option<pyo3::PyObject>,
    ) -> pyo3::PyResult<Self> {
        let node = get_node_from_pyobject(node.bind(py))?;

        // find namespaces
        let mut namespaces = arcdom::NamespacesHashMap::new();

        if let Some(parser) = parser {
            if let Ok(html) = parser.extract::<pyo3::PyRef<'_, PyRawHtml>>(py) {
                namespaces = html.0.namespaces.borrow().clone();
            } else if let Ok(html) = parser.extract::<pyo3::PyRef<'_, PyRawXml>>(py) {
                namespaces = html.0.namespaces.borrow().clone();
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "expected RawHtml or RawXml as parser",
                ));
            }
        } else {
            if let Some(elem) = node.as_element() {
                if let Some(ref prefix) = elem.name.prefix {
                    namespaces.insert(prefix.clone(), elem.name.ns.clone());
                }
            }

            for n in node.tree() {
                if let Some(elem) = n.as_element() {
                    if let Some(ref prefix) = elem.name.prefix {
                        namespaces.insert(prefix.clone(), elem.name.ns.clone());
                    }
                }
            }
        }

        let expr =
            matching::Select::new(node.into_tree(), &expr, Some(namespaces)).map_err(|err| {
                pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(err.to_string())
            })?;
        
        Ok(Self(expr))
    }

    pub fn __iter__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf
    }

    pub fn __next__(
        mut slf: pyo3::PyRefMut<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        match slf.0.next() {
            None => Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(())),
            Some(node) => {
                let node = super::node::PyRawNode(node);
                Ok(pyo3::Py::new(py, node)?.into_any())
            }
        }
    }
}
