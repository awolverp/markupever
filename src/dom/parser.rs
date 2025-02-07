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
            self.exact_errors, self.discard_bom, self.profile,
        )
    }
}

enum StreamWrapper {
    Html(
        treedom::tendril::stream::Utf8LossyDecoder<
            treedom::html5ever::driver::Parser<treedom::MarkupParser>,
        >,
    ),
    Xml(
        treedom::tendril::stream::Utf8LossyDecoder<
            treedom::xml5ever::driver::XmlParser<treedom::MarkupParser>,
        >,
    ),
}

impl StreamWrapper {
    fn as_html(val: treedom::html5ever::driver::Parser<treedom::MarkupParser>) -> Self {
        Self::Html(treedom::tendril::stream::Utf8LossyDecoder::new(val))
    }

    fn as_xml(val: treedom::xml5ever::driver::XmlParser<treedom::MarkupParser>) -> Self {
        Self::Xml(treedom::tendril::stream::Utf8LossyDecoder::new(val))
    }

    fn process(&mut self, content: Vec<u8>) {
        use treedom::tendril::TendrilSink;

        match self {
            Self::Html(x) => x.process(treedom::tendril::ByteTendril::from_slice(&content)),
            Self::Xml(x) => x.process(treedom::tendril::ByteTendril::from_slice(&content)),
        }
    }

    fn finish(self) -> treedom::MarkupParser {
        use treedom::tendril::TendrilSink;

        match self {
            Self::Html(x) => x.finish(),
            Self::Xml(x) => x.finish(),
        }
    }
}

#[derive(Debug)]
enum ParserState {
    /// Means [`PyParser`] has completed the parsing process
    Finished(Box<treedom::MarkupParser>),

    /// Means [`PyParser`] has converted into [`PyTreeDom`](struct@crate::dom::PyTreeDom)
    /// and it is un-usable now
    Dropped,
}

#[pyo3::pyclass(name = "Parser", module = "xmarkup._rustlib", frozen)]
pub struct PyParser {
    state: parking_lot::Mutex<ParserState>,
}

#[pyo3::pymethods]
impl PyParser {
    #[new]
    fn new(
        content: pyo3::Bound<'_, pyo3::types::PyAny>,
        options: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        use pyo3::types::PyAnyMethods;

        if unsafe { pyo3::ffi::PyGen_Check(content.as_ptr()) == 0 } {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                format!("expected generator for content, got {}", unsafe {
                    crate::tools::get_type_name(content.py(), content.as_ptr())
                }),
            ));
        }

        let mut stream = {
            if let Ok(options) = options.extract::<pyo3::PyRef<'_, PyHtmlOptions>>() {
                StreamWrapper::as_html(treedom::MarkupParser::parse_html(
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
                StreamWrapper::as_xml(treedom::MarkupParser::parse_xml(
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

        for result in unsafe { content.try_iter().unwrap_unchecked() } {
            let result = result?;

            let result = unsafe {
                if pyo3::ffi::PyBytes_Check(result.as_ptr()) == 1 {
                    result.extract::<Vec<u8>>().unwrap()
                } else if pyo3::ffi::PyUnicode_Check(result.as_ptr()) == 1 {
                    let s = result.extract::<String>().unwrap();
                    s.into_bytes()
                } else {
                    return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        format!(
                            "expected bytes or str for the content generator result, got {}",
                            crate::tools::get_type_name(result.py(), result.as_ptr())
                        ),
                    ));
                }
            };

            stream.process(result);
        }

        let state = ParserState::Finished(Box::new(stream.finish()));

        Ok(Self {
            state: parking_lot::Mutex::new(state),
        })
    }

    #[allow(clippy::wrong_self_convention)]
    fn into_dom(&self) -> pyo3::PyResult<super::tree::PyTreeDom> {
        let mut state = self.state.lock();

        let markup = std::mem::replace(&mut *state, ParserState::Dropped);

        match markup {
            ParserState::Finished(p) => Ok(super::tree::PyTreeDom::from_treedom(p.into_dom())),
            ParserState::Dropped => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "The parser is already converted into dom and dropped",
            )),
        }
    }

    fn errors(&self) -> pyo3::PyResult<Vec<String>> {
        let state = self.state.lock();

        match &*state {
            ParserState::Finished(p) => {
                Ok(p.errors().iter().map(|x| x.clone().into_owned()).collect())
            }
            ParserState::Dropped => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "The parser has converted into dom and dropped",
            )),
        }
    }

    fn quirks_mode(&self) -> pyo3::PyResult<u8> {
        let state = self.state.lock();

        match &*state {
            ParserState::Finished(p) => {
                Ok(crate::tools::convert_quirks_mode_to_u8(p.quirks_mode()))
            }
            ParserState::Dropped => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "The parser has converted into dom and dropped",
            )),
        }
    }

    fn lineno(&self) -> pyo3::PyResult<u64> {
        let state = self.state.lock();

        match &*state {
            ParserState::Finished(p) => Ok(p.lineno()),
            ParserState::Dropped => Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "The parser has converted into dom and dropped",
            )),
        }
    }
}

unsafe impl Send for PyParser {}
unsafe impl Sync for PyParser {}
