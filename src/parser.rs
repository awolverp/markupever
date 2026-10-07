/// These are options for HTML parsing.
///
/// # Note
/// this type is immutable.
#[pyo3::pyclass(name = "HtmlOptions", module = "markupever._rustlib", frozen)]
pub struct PyHtmlOptions {
    exact_errors: bool,
    discard_bom: bool,
    profile: bool,
    iframe_srcdoc: bool,
    drop_doctype: bool,
    full_document: bool,
    fragment_context: Option<treedom::markup5ever::QualName>,
    quirks_mode: treedom::markup5ever::interface::QuirksMode,
}

#[pyo3::pymethods]
impl PyHtmlOptions {
    /// Creates a new [`PyHtmlOptions`]
    ///
    /// - `full_document`: Is this a complete document? (means includes html, head, and body tag). Default: true,
    ///   or false if `fragment_context` is given.
    /// - `exact_errors`: Report all parse errors described in the spec, at some performance penalty? Default: false.
    /// - `discard_bom`: Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? Default: true.
    /// - `profile`: Keep a record of how long we spent in each state? Printed when `finish()` is called. Default: false.
    /// - `iframe_srcdoc`: Is this an `iframe srcdoc` document? Default: false.
    /// - `drop_doctype`: Should we drop the DOCTYPE (if any) from the tree? Default: false.
    /// - `quirks_mode`: Initial TreeBuilder quirks mode. Default: QUIRKS_MODE_OFF.
    /// - `fragment_context`: Parse a fragment as if it were the contents of this context element, e.g. `"td"` or
    ///   `QualName("path", "svg")`. A name without a namespace is an HTML element. Default: None (fragments are
    ///   parsed in a `body` context).
    #[new]
    #[pyo3(signature=(full_document=None, exact_errors=false, discard_bom=true, profile=false, iframe_srcdoc=false, drop_doctype=false, quirks_mode=crate::tools::QUIRKS_MODE_OFF, fragment_context=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        full_document: Option<bool>,
        exact_errors: bool,
        discard_bom: bool,
        profile: bool,
        iframe_srcdoc: bool,
        drop_doctype: bool,
        quirks_mode: u8,
        fragment_context: Option<crate::tools::PyQualNameOrStr>,
    ) -> pyo3::PyResult<Self> {
        let quirks_mode =
            crate::tools::convert_u8_to_quirks_mode(quirks_mode).ok_or_else(|| {
                pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "quirks_mode must be between 0 and 2, got {}",
                    quirks_mode
                ))
            })?;

        let fragment_context = fragment_context.map(|x| {
            let mut name = x.into_qualname();
            if name.ns.is_empty() {
                name.ns = treedom::markup5ever::namespace_url!("http://www.w3.org/1999/xhtml");
            }
            name
        });

        let full_document = match (full_document, &fragment_context) {
            (Some(true), Some(_)) => {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "full_document=True cannot be combined with fragment_context",
                ))
            }
            (Some(full_document), _) => full_document,
            (None, fragment_context) => fragment_context.is_none(),
        };

        Ok(Self {
            exact_errors,
            discard_bom,
            profile,
            iframe_srcdoc,
            drop_doctype,
            full_document,
            fragment_context,
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

    #[getter]
    fn fragment_context(&self) -> Option<crate::qualname::PyQualName> {
        self.fragment_context
            .clone()
            .map(|name| crate::qualname::PyQualName { name })
    }

    fn __repr__(&self) -> String {
        format!(
            "markupever._rustlib.HtmlOptions(full_document={}, exact_errors={}, discard_bom={}, profile={}, iframe_srcdoc={}, drop_doctype={}, quirks_mode={}, fragment_context={})",
            self.full_document,
            self.exact_errors,
            self.discard_bom,
            self.profile,
            self.iframe_srcdoc,
            self.drop_doctype,
            crate::tools::convert_quirks_mode_to_u8(self.quirks_mode),
            self.fragment_context
                .as_ref()
                .map_or_else(|| "None".to_owned(), crate::qualname::repr_qualname),
        )
    }
}

#[pyo3::pyclass(name = "XmlOptions", module = "markupever._rustlib", frozen)]
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

    fn process(&mut self, content: &[u8]) -> pyo3::PyResult<()> {
        use treedom::tendril::TendrilSink;

        match self {
            Self::OnHtml(x) => x.process(treedom::tendril::ByteTendril::from_slice(content)),
            Self::OnXml(x) => x.process(treedom::tendril::ByteTendril::from_slice(content)),
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

#[derive(pyo3::FromPyObject)]
enum Input {
    Bytes(pyo3::pybacked::PyBackedBytes),
    Str(pyo3::pybacked::PyBackedStr),
}

/// An HTML/XML parser, ready to receive unicode input.
///
/// This is very easy to use and allows you to stream input using `.process()` method; By this way
/// you are don't worry about memory usages of huge inputs.
#[pyo3::pyclass(name = "Parser", module = "markupever._rustlib", frozen, unsendable)]
pub struct PyParser {
    state: parking_lot::Mutex<ParserState>,
}

#[derive(pyo3::FromPyObject)]
enum PyParserOptions<'p> {
    Html(pyo3::PyRef<'p, PyHtmlOptions>),
    Xml(pyo3::PyRef<'p, PyXmlOptions>),
}

#[pyo3::pymethods]
impl PyParser {
    /// Creates a new [`PyParser`]
    ///
    /// - `options`: If your input is a HTML document, pass a PyHtmlOptions;
    ///              If your input is a XML document, pass PyXmlOptions.
    #[new]
    fn new(options: PyParserOptions) -> pyo3::PyResult<Self> {
        let state = {
            match options {
                PyParserOptions::Html(options) => {
                    let tokenizer = treedom::html5ever::tokenizer::TokenizerOpts {
                        exact_errors: options.exact_errors,
                        discard_bom: options.discard_bom,
                        profile: options.profile,
                        ..Default::default()
                    };
                    let tree_builder = treedom::html5ever::tree_builder::TreeBuilderOpts {
                        exact_errors: options.exact_errors,
                        iframe_srcdoc: options.iframe_srcdoc,
                        drop_doctype: options.drop_doctype,
                        quirks_mode: options.quirks_mode,
                        ..Default::default()
                    };

                    ParserState::as_html(match &options.fragment_context {
                        Some(context) => treedom::ParserSink::parse_html_fragment(
                            context.clone(),
                            tokenizer,
                            tree_builder,
                        ),
                        None => treedom::ParserSink::parse_html(
                            options.full_document,
                            tokenizer,
                            tree_builder,
                        ),
                    })
                }
                PyParserOptions::Xml(options) => {
                    ParserState::as_xml(treedom::ParserSink::parse_xml(
                        treedom::xml5ever::tokenizer::XmlTokenizerOpts {
                            exact_errors: options.exact_errors,
                            discard_bom: options.discard_bom,
                            profile: options.profile,
                            ..Default::default()
                        },
                    ))
                }
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
    fn process(&self, content: Input) -> pyo3::PyResult<()> {
        let content = match &content {
            Input::Bytes(b) => b,
            Input::Str(s) => s.as_bytes(),
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

        format!("<Parser - {:?}>", *state)
    }
}

#[pyo3::pyfunction]
#[pyo3(signature=(node, indent=0, include_self=true, is_html=None))]
pub fn serialize(
    node: crate::nodes::PyNodeRef,
    indent: usize,
    include_self: bool,
    is_html: Option<bool>,
) -> pyo3::PyResult<Vec<u8>> {
    let node = node.as_node_guard();

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

    let serializer = ::treedom::Serializer::new(&dom, node.id, indent);

    let value = dom.get(node.id).unwrap().value();

    let traversal_scope = if let Some(context) = value.document().and(dom.fragment_context()) {
        // A fragment's nodes are the children of the document node, but were parsed as
        // children of the context element, so serialize them as its children: for example,
        // text parsed in a <script> context is raw text.
        ::treedom::markup5ever::serialize::TraversalScope::ChildrenOnly(Some(context.clone()))
    } else if include_self {
        ::treedom::markup5ever::serialize::TraversalScope::IncludeNode
    } else {
        // The serializer needs the parent's name to know whether its text
        // children are raw text (as in <script> or <style>).
        let name = value.element().map(|element| element.name.clone());

        ::treedom::markup5ever::serialize::TraversalScope::ChildrenOnly(name)
    };

    if !is_html {
        ::treedom::xml5ever::serialize::serialize(
            &mut writer,
            &serializer,
            ::treedom::xml5ever::serialize::SerializeOpts { traversal_scope },
        )
        .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    } else {
        ::treedom::html5ever::serialize::serialize(
            &mut writer,
            &serializer,
            ::treedom::html5ever::serialize::SerializeOpts {
                traversal_scope,
                ..Default::default()
            },
        )
        .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    }

    Ok(writer)
}
