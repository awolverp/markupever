use pyo3::types::PyAnyMethods;

/// These are options for HTML parsing.
///
/// # Note
/// this type is immutable.
#[pyo3::pyclass(name = "HtmlOptions", module = "markupselect._rustlib", frozen)]
pub struct PyHtmlOptions {
    exact_errors: bool,
    discard_bom: bool,
    profile: bool,
    iframe_srcdoc: bool,
    drop_doctype: bool,
    full_document: bool,
    quirks_mode: treedom::markup5ever::interface::QuirksMode,
}

#[pyo3::pymethods]
impl PyHtmlOptions {
    /// Creates a new [`PyHtmlOptions`]
    ///
    /// - `full_document`: Is this a complete document? (means includes html, head, and body tag). Default: true.
    /// - `exact_errors`: Report all parse errors described in the spec, at some performance penalty? Default: false.
    /// - `discard_bom`: Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? Default: true.
    /// - `profile`: Keep a record of how long we spent in each state? Printed when `finish()` is called. Default: false.
    /// - `iframe_srcdoc`: Is this an `iframe srcdoc` document? Default: false.
    /// - `drop_doctype`: Should we drop the DOCTYPE (if any) from the tree? Default: false.
    /// - `quirks_mode`: Initial TreeBuilder quirks mode. Default: QUIRKS_MODE_OFF.
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
            "markupever._rustlib.HtmlOptions(full_document={}, exact_errors={}, discard_bom={}, profile={}, iframe_srcdoc={}, drop_doctype={}, quirks_mode={})",
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
    exact_errors: bool,
    discard_bom: bool,
    profile: bool,
}

#[pyo3::pymethods]
impl PyXmlOptions {
    /// Creates a new [`PyXmlOptions`]
    ///
    /// - `exact_errors`: Report all parse errors described in the spec, at some performance penalty? Default: false.
    /// - `discard_bom`: Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? Default: true.
    /// - `profile`: Keep a record of how long we spent in each state? Printed when `finish()` is called. Default: false.
    #[new]
    #[pyo3(signature=(exact_errors=false, discard_bom=true, profile=false))]
    fn new(exact_errors: bool, discard_bom: bool, profile: bool) -> Self {
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
            "markupever._rustlib.XmlOptions(exact_errors={}, discard_bom={}, profile={})",
            self.exact_errors, self.discard_bom, self.profile,
        )
    }
}

enum ParserState {
    /// Means [`PyParser`] is parsing HTML
    OnHtml(
        Box<
            treedom::tendril::stream::Utf8LossyDecoder<
                treedom::html5ever::driver::Parser<treedom::ParserSink>,
            >,
        >,
    ),

    /// Means [`PyParser`] is parsing XML
    OnXml(
        Box<
            treedom::tendril::stream::Utf8LossyDecoder<
                treedom::xml5ever::driver::XmlParser<treedom::ParserSink>,
            >,
        >,
    ),

    /// Means [`PyParser`] has completed the parsing process
    Finished(treedom::ParserSink),

    /// Means [`PyParser`] has converted into [`PyTreeDom`](struct@crate::dom::PyTreeDom)
    /// and it is un-usable now
    Dropped,
}

impl ParserState {
    fn as_html(val: treedom::html5ever::driver::Parser<treedom::ParserSink>) -> Self {
        Self::OnHtml(Box::new(treedom::tendril::stream::Utf8LossyDecoder::new(
            val,
        )))
    }

    fn as_xml(val: treedom::xml5ever::driver::XmlParser<treedom::ParserSink>) -> Self {
        Self::OnXml(Box::new(treedom::tendril::stream::Utf8LossyDecoder::new(
            val,
        )))
    }

    fn process(&mut self, content: Vec<u8>) -> pyo3::PyResult<()> {
        use treedom::tendril::TendrilSink;

        match self {
            Self::OnHtml(x) => x.process(treedom::tendril::ByteTendril::from_slice(&content)),
            Self::OnXml(x) => x.process(treedom::tendril::ByteTendril::from_slice(&content)),
            _ => {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "The parser is completed parsing",
                ))
            }
        }

        Ok(())
    }

    fn finish(self) -> treedom::ParserSink {
        use treedom::tendril::TendrilSink;

        match self {
            Self::OnHtml(x) => x.finish(),
            Self::OnXml(x) => x.finish(),
            _ => panic!("The parser is completed parsing"),
        }
    }
}

impl std::fmt::Debug for ParserState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnHtml(..) => write!(f, "parsing HTML"),
            Self::OnXml(..) => write!(f, "parsing XML"),
            Self::Finished(..) => write!(f, "finished"),
            Self::Dropped => write!(f, "converted"),
        }
    }
}

/// An HTML/XML parser, ready to receive unicode input.
///
/// This is very easy to use and allows you to stream input using `.process()` method; By this way
/// you are don't worry about memory usages of huge inputs.
#[pyo3::pyclass(name = "Parser", module = "markupever._rustlib", frozen)]
pub struct PyParser {
    state: parking_lot::Mutex<ParserState>,
}

#[pyo3::pymethods]
impl PyParser {
    /// Creates a new [`PyParser`]
    ///
    /// - `options`: If your input is a HTML document, pass a PyHtmlOptions;
    ///              If your input is a XML document, pass PyXmlOptions.
    #[new]
    fn new(options: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let state = {
            if let Ok(options) = options.extract::<pyo3::PyRef<'_, PyHtmlOptions>>() {
                ParserState::as_html(treedom::ParserSink::parse_html(
                    options.full_document,
                    treedom::html5ever::tokenizer::TokenizerOpts {
                        exact_errors: options.exact_errors,
                        discard_bom: options.discard_bom,
                        profile: options.profile,
                        ..Default::default()
                    },
                    treedom::html5ever::tree_builder::TreeBuilderOpts {
                        exact_errors: options.exact_errors,
                        iframe_srcdoc: options.iframe_srcdoc,
                        drop_doctype: options.drop_doctype,
                        quirks_mode: options.quirks_mode,
                        ..Default::default()
                    },
                ))
            } else if let Ok(options) = options.extract::<pyo3::PyRef<'_, PyXmlOptions>>() {
                ParserState::as_xml(treedom::ParserSink::parse_xml(
                    treedom::xml5ever::tokenizer::XmlTokenizerOpts {
                        exact_errors: options.exact_errors,
                        discard_bom: options.discard_bom,
                        profile: options.profile,
                        ..Default::default()
                    },
                ))
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    format!(
                        "expected HtmlOptions or XmlOptions for options, got {}",
                        unsafe { crate::tools::get_type_name(options.py(), options.as_ptr()) }
                    ),
                ));
            }
        };

        Ok(Self {
            state: parking_lot::Mutex::new(state),
        })
    }

    /// Processes an input.
    ///
    /// `content` must be `str` or `bytes`.
    ///
    /// Raises `RuntimeError` if `.finish()` method is called.
    fn process(&self, content: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let content = unsafe {
            if pyo3::ffi::PyBytes_Check(content.as_ptr()) == 1 {
                content.extract::<Vec<u8>>().unwrap()
            } else if pyo3::ffi::PyUnicode_Check(content.as_ptr()) == 1 {
                let s = content.extract::<String>().unwrap();
                s.into_bytes()
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    format!(
                        "expected bytes or str for content, got {}",
                        crate::tools::get_type_name(content.py(), content.as_ptr())
                    ),
                ));
            }
        };

        let mut state = self.state.lock();
        state.process(content)
    }

    /// Finishes the parser and marks it as finished.
    fn finish(&self) -> pyo3::PyResult<()> {
        let mut state = self.state.lock();

        if matches!(&*state, ParserState::Finished(..) | ParserState::Dropped) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "The parser is already finished",
            ));
        }

        let dom = std::mem::replace(&mut *state, ParserState::Dropped);
        let _ = std::mem::replace(&mut *state, ParserState::Finished(dom.finish()));

        Ok(())
    }

    /// Converts the self into `PyTreeDom`.
    /// after calling this method, the self is unusable and you cannot use it.
    #[allow(clippy::wrong_self_convention)]
    fn into_dom(&self) -> pyo3::PyResult<super::tree::PyTreeDom> {
        let mut state = self.state.lock();

        let markup = std::mem::replace(&mut *state, ParserState::Dropped);

        match markup {
            ParserState::Finished(p) => Ok(super::tree::PyTreeDom::from_treedom(p.into_dom())),
            ParserState::Dropped => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the parser is already converted into dom and dropped",
            )),
            _ => {
                let _ = std::mem::replace(&mut *state, markup);

                Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "the parser is not finished yet",
                ))
            }
        }
    }

    /// Returns the errors which are detected while parsing
    fn errors(&self) -> pyo3::PyResult<Vec<String>> {
        let state = self.state.lock();

        match &*state {
            ParserState::Finished(p) => {
                Ok(p.errors().iter().map(|x| x.clone().into_owned()).collect())
            }
            ParserState::Dropped => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the parser has converted into dom and dropped",
            )),
            _ => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the parser is not finished yet",
            )),
        }
    }

    /// Returns the Quirks Mode.
    fn quirks_mode(&self) -> pyo3::PyResult<u8> {
        let state = self.state.lock();

        match &*state {
            ParserState::Finished(p) => {
                Ok(crate::tools::convert_quirks_mode_to_u8(p.quirks_mode()))
            }
            ParserState::Dropped => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "The parser has converted into dom and dropped",
            )),
            _ => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the parser is not finished yet",
            )),
        }
    }

    /// Returns the line count of the parsed content (always is `1` for XML).
    fn lineno(&self) -> pyo3::PyResult<u64> {
        let state = self.state.lock();

        match &*state {
            ParserState::Finished(p) => Ok(p.lineno()),
            ParserState::Dropped => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "The parser has converted into dom and dropped",
            )),
            _ => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the parser is not finished yet",
            )),
        }
    }

    fn __repr__(&self) -> String {
        let state = self.state.lock();

        format!("<Parser - {:?}>", &*state)
    }
}

unsafe impl Send for PyParser {}
unsafe impl Sync for PyParser {}

#[pyo3::pyfunction]
#[pyo3(signature=(node, is_html=None))]
pub fn serialize(
    node: &pyo3::Bound<'_, pyo3::PyAny>,
    is_html: Option<bool>,
) -> pyo3::PyResult<Vec<u8>> {
    let node = super::nodes::NodeGuard::from_pyobject(node).map_err(|_| {
        pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
            "expected an node (such as Element, Text, Comment, ...) for node, got {}",
            unsafe { crate::tools::get_type_name(node.py(), node.as_ptr()) }
        ))
    })?;

    let is_html = match is_html {
        Some(x) => x,
        None => {
            let tree = node.tree.lock();
            let ns = tree.namespaces();

            if ns.is_empty() {
                false
            } else if let Some(x) = ns.get(&::treedom::markup5ever::Prefix::from("")) {
                x == &::treedom::markup5ever::namespace_url!("http://www.w3.org/1999/xhtml")
            } else {
                false
            }
        }
    };

    let mut writer = Vec::with_capacity(10);
    let dom = node.tree.lock();

    let serializer = ::treedom::Serializer::new(&dom, node.id);

    if !is_html {
        ::treedom::xml5ever::serialize::serialize(
            &mut writer,
            &serializer,
            ::treedom::xml5ever::serialize::SerializeOpts {
                traversal_scope: ::treedom::xml5ever::serialize::TraversalScope::IncludeNode,
            },
        )
        .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    } else {
        ::treedom::html5ever::serialize::serialize(
            &mut writer,
            &serializer,
            ::treedom::html5ever::serialize::SerializeOpts {
                traversal_scope: ::treedom::html5ever::serialize::TraversalScope::IncludeNode,
                ..Default::default()
            },
        )
        .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    }

    Ok(writer)
}
