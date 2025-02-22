use super::dom::IDTreeDOM;
use super::interface;
use hashbrown::HashMap;
use std::cell::{Cell, UnsafeCell};

/// Markup parser that implemented [`markup5ever::interface::TreeSink`]
#[derive(Debug)]
pub struct ParserSink {
    tree: UnsafeCell<ego_tree::Tree<interface::Interface>>,
    errors: UnsafeCell<Vec<std::borrow::Cow<'static, str>>>,
    quirks_mode: Cell<markup5ever::interface::QuirksMode>,
    namespaces: UnsafeCell<HashMap<markup5ever::Prefix, markup5ever::Namespace>>,
    lineno: UnsafeCell<u64>,
}

impl Default for ParserSink {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserSink {
    /// Creates a new [`ParserSink`]
    pub fn new() -> Self {
        Self {
            tree: UnsafeCell::new(ego_tree::Tree::new(interface::Interface::new(
                interface::DocumentInterface,
            ))),
            errors: UnsafeCell::new(Vec::new()),
            quirks_mode: Cell::new(markup5ever::interface::QuirksMode::NoQuirks),
            namespaces: UnsafeCell::new(HashMap::new()),
            lineno: UnsafeCell::new(1),
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn tree_mut(&self) -> &mut ego_tree::Tree<interface::Interface> {
        // SAFETY: Parser is not Send/Sync so cannot be used in multi threads.
        unsafe { &mut *self.tree.get() }
    }

    /// Returns the errors which are detected while parsing
    pub fn errors(&self) -> &Vec<std::borrow::Cow<'static, str>> {
        unsafe { &*self.errors.get() }
    }

    /// Returns the Quirks Mode
    pub fn quirks_mode(&self) -> markup5ever::interface::QuirksMode {
        self.quirks_mode.get()
    }

    /// Returns the line count of the parsed content (does nothing for XML)
    pub fn lineno(&self) -> u64 {
        unsafe { *self.lineno.get() }
    }

    /// Consumes the self and returns [`IDTreeDOM`]
    pub fn into_dom(self) -> IDTreeDOM {
        IDTreeDOM {
            tree: self.tree.into_inner(),
            namespaces: self.namespaces.into_inner(),
        }
    }

    /// Returns a [`html5ever::driver::Parser<Self>`] that ready for parsing
    #[cfg(feature = "html5ever")]
    pub fn parse_html(
        full_document: bool,
        tokenizer: html5ever::tokenizer::TokenizerOpts,
        tree_builder: html5ever::tree_builder::TreeBuilderOpts,
    ) -> html5ever::driver::Parser<Self> {
        let opts = html5ever::driver::ParseOpts {
            tokenizer,
            tree_builder,
        };

        if full_document {
            html5ever::driver::parse_document(Self::new(), opts)
        } else {
            html5ever::driver::parse_fragment(
                Self::new(),
                opts,
                html5ever::QualName::new(
                    None,
                    markup5ever::namespace_url!("http://www.w3.org/1999/xhtml"),
                    markup5ever::local_name!("body"),
                ),
                Vec::new(),
            )
        }
    }

    /// Returns a [`xml5ever::driver::XmlParser<Self>`] that ready for parsing
    #[cfg(feature = "xml5ever")]
    pub fn parse_xml(
        tokenizer: xml5ever::tokenizer::XmlTokenizerOpts,
    ) -> xml5ever::driver::XmlParser<Self> {
        let opts = xml5ever::driver::XmlParseOpts {
            tokenizer,
            tree_builder: Default::default(),
        };

        xml5ever::driver::parse_document(Self::new(), opts)
    }
}

impl markup5ever::interface::TreeSink for ParserSink {
    type Output = Self;
    type Handle = ego_tree::NodeId;
    type ElemName<'a> = markup5ever::ExpandedName<'a>;

    // Consume this sink and return the overall result of parsing.
    fn finish(self) -> Self::Output {
        self
    }

    // Signal a parse error.
    fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
        unsafe { &mut *self.errors.get() }.push(msg);
    }

    // Called whenever the line number changes.
    fn set_current_line(&self, n: u64) {
        unsafe {
            *self.lineno.get() = n;
        }
    }

    // Set the document's quirks mode.
    fn set_quirks_mode(&self, mode: markup5ever::interface::QuirksMode) {
        self.quirks_mode.set(mode);
    }

    // Get a handle to the `Document` node.
    fn get_document(&self) -> Self::Handle {
        self.tree_mut().root().id()
    }

    // Get a handle to a template's template contents.
    // The tree builder promises this will never be called with something else than a template element.
    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        let item = self.tree_mut().get(*target).unwrap();

        if let Some(x) = item.value().element() {
            if x.template {
                return *target;
            }

            unreachable!("target is not a template");
        } else {
            unreachable!("target is not a element");
        }
    }

    // Do two handles refer to the same node?
    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    // What is the name of this element?
    //
    // Should never be called on a non-element node; feel free to panic!.
    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        let item = self.tree_mut().get(*target).unwrap();

        if let Some(x) = item.value().element() {
            x.name.expanded()
        } else {
            unreachable!("target is not a element");
        }
    }

    // Create an element.
    //
    // When creating a template element (name.ns.expanded() == expanded_name!(html "template")),
    // an associated document fragment called the "template contents" should also be created.
    // Later calls to self.get_template_contents() with that given element return it.
    // See the template element in the whatwg spec.
    fn create_element(
        &self,
        name: markup5ever::QualName,
        attrs: Vec<markup5ever::Attribute>,
        flags: markup5ever::interface::ElementFlags,
    ) -> Self::Handle {
        // Keep all the namespaces in a hashmap, we need them for css selectors
        unsafe {
            if let Some(ref prefix) = name.prefix {
                (*self.namespaces.get()).insert(prefix.clone(), name.ns.clone());
            } else if (*self.namespaces.get()).is_empty() {
                (*self.namespaces.get()).insert(markup5ever::Prefix::from(""), name.ns.clone());
            }
        }

        let element = interface::ElementInterface::from_non_atomic(
            name,
            attrs.into_iter().map(|x| (x.name, x.value)),
            flags.template,
            flags.mathml_annotation_xml_integration_point,
        );

        let node = self.tree_mut().orphan(interface::Interface::new(element));
        node.id()
    }

    // Create a comment node.
    fn create_comment(&self, text: tendril::StrTendril) -> Self::Handle {
        let node = self.tree_mut().orphan(interface::Interface::new(
            interface::CommentInterface::from_non_atomic(text),
        ));

        node.id()
    }

    // Create a Processing Instruction node.
    fn create_pi(&self, target: tendril::StrTendril, data: tendril::StrTendril) -> Self::Handle {
        let node = self.tree_mut().orphan(interface::Interface::new(
            interface::ProcessingInstructionInterface::from_non_atomic(data, target),
        ));

        node.id()
    }

    // Append a DOCTYPE element to the Document node.
    fn append_doctype_to_document(
        &self,
        name: tendril::StrTendril,
        public_id: tendril::StrTendril,
        system_id: tendril::StrTendril,
    ) {
        self.tree_mut().root_mut().append(interface::Interface::new(
            interface::DoctypeInterface::from_non_atomic(name, public_id, system_id),
        ));
    }

    // Append a node as the last child of the given node. If this would produce adjacent sibling text nodes, it should concatenate the text instead.
    //
    // The child node will not already have a parent.
    fn append(
        &self,
        parent: &Self::Handle,
        child: markup5ever::interface::NodeOrText<Self::Handle>,
    ) {
        let mut parent = self.tree_mut().get_mut(*parent).unwrap();

        match child {
            markup5ever::interface::NodeOrText::AppendNode(handle) => {
                parent.append_id(handle);
            }
            markup5ever::interface::NodeOrText::AppendText(text) => {
                if let Some(mut last_index) = parent.last_child() {
                    if let Some(textval) = last_index.value().text_mut() {
                        textval.push_non_atomic(text);
                        return;
                    }
                }

                parent.append(interface::Interface::new(
                    interface::TextInterface::from_non_atomic(text),
                ));
            }
        }
    }

    // Append a node as the sibling immediately before the given node.
    //
    // The tree builder promises that sibling is not a text node. However its old previous sibling, which would become the new node's previous sibling, could be a text node. If the new node is also a text node, the two should be merged, as in the behavior of append.
    //
    // NB: new_item may have an old parent, from which it should be removed.
    fn append_before_sibling(
        &self,
        sibling_id: &Self::Handle,
        new_node_id: markup5ever::interface::NodeOrText<Self::Handle>,
    ) {
        let mut sibling = self.tree_mut().get_mut(*sibling_id).unwrap();

        match (new_node_id, sibling.prev_sibling()) {
            (markup5ever::interface::NodeOrText::AppendText(text), None) => {
                // There's no previous item, so we have to create a Text node data
                sibling.insert_before(interface::Interface::new(
                    interface::TextInterface::from_non_atomic(text),
                ));
            }
            (markup5ever::interface::NodeOrText::AppendText(text), Some(mut prev)) => {
                // // There's a previous item, so may it's a Text node data? we have to check
                if let Some(textval) = prev.value().text_mut() {
                    textval.push_non_atomic(text);
                } else {
                    sibling.insert_before(interface::Interface::new(
                        interface::TextInterface::from_non_atomic(text),
                    ));
                }
            }
            (markup5ever::interface::NodeOrText::AppendNode(node_id), _) => {
                sibling.insert_id_before(node_id);
            }
        }
    }

    // When the insertion point is decided by the existence of a parent node of the element,
    // we consider both possibilities and send the element which will be used if a parent
    // node exists, along with the element to be used if there isn't one.
    fn append_based_on_parent_node(
        &self,
        item_index: &Self::Handle,
        prev_item_index: &Self::Handle,
        child: markup5ever::interface::NodeOrText<Self::Handle>,
    ) {
        let item = self.tree_mut().get(*item_index).unwrap();

        if item.parent().is_some() {
            self.append_before_sibling(item_index, child);
        } else {
            self.append(prev_item_index, child);
        }
    }

    // Add each attribute to the given element, if no attribute with that name already exists.
    // The tree builder promises this will never be called with something else than an element.
    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<markup5ever::Attribute>) {
        let mut node = self.tree_mut().get_mut(*target).unwrap();

        if let Some(element) = node.value().element_mut() {
            element.attrs.extend(
                attrs
                    .into_iter()
                    .map(|x| (x.name.into(), crate::atomic::make_atomic_tendril(x.value))),
            );
        } else {
            unreachable!("add_attrs_if_missing called on a non-element node")
        }
    }

    // Detach the given node from its parent.
    fn remove_from_parent(&self, target: &Self::Handle) {
        self.tree_mut().get_mut(*target).unwrap().detach();
    }

    // Remove all the children from node and append them to new_parent.
    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        self.tree_mut()
            .get_mut(*new_parent)
            .unwrap()
            .reparent_from_id_append(*node);
    }

    // Returns true if the adjusted current node is an HTML integration point and the token is a start tag.
    fn is_mathml_annotation_xml_integration_point(&self, target: &Self::Handle) -> bool {
        let item = self.tree_mut().get(*target).unwrap();

        if let Some(x) = item.value().element() {
            x.mathml_annotation_xml_integration_point
        } else {
            unreachable!("target is not a element");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendril::TendrilSink;

    const HTML: &'static str = r#"<!DOCTYPE html><html lang="en"><head><title>Document</title></head><body><template>TEST</template></body></html>"#;
    const XML: &'static str = r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE suite SYSTEM "http://testng.org/testng-1.0.dtd"><suite name="TestSuite"><test name="TestProject"><classes><class name="package.firstClassName" /><class name="package.secondClassName" /></classes></test></suite>"#;

    #[test]
    fn html_parsing() {
        let parser = ParserSink::parse_html(true, Default::default(), Default::default());
        let dom = parser.one(HTML).into_dom();

        let root = dom.root();
        assert_eq!(
            *root.value(),
            interface::Interface::new(interface::DocumentInterface)
        );

        let children: Vec<_> = root.children().collect();

        assert!(children[0].value().is_doctype());
        assert_eq!(
            children[1].value().element().unwrap().name.local.as_ref(),
            "html",
        );

        let html = children[1];

        let children: Vec<_> = html.children().collect();

        assert_eq!(
            children[0].value().element().unwrap().name.local.as_ref(),
            "head",
        );
        assert_eq!(
            children[1].value().element().unwrap().name.local.as_ref(),
            "body",
        );
    }

    #[test]
    fn xml_parsing() {
        let parser = ParserSink::parse_xml(Default::default());
        let dom = parser.one(XML).into_dom();

        let root = dom.root();
        assert_eq!(
            *root.value(),
            interface::Interface::new(interface::DocumentInterface)
        );

        let children: Vec<_> = root.children().collect();

        assert!(children[0].value().is_processing_instruction());
        assert!(children[1].value().is_doctype());
        assert_eq!(
            children[2].value().element().unwrap().name.local.as_ref(),
            "suite",
        );

        let suite = children[2];

        let children: Vec<_> = suite.children().collect();

        assert_eq!(
            children[0].value().element().unwrap().name.local.as_ref(),
            "test",
        );
    }

    #[cfg(feature = "html5ever")]
    #[test]
    fn html_serializer() {
        use crate::Serializer;

        let parser = ParserSink::parse_html(true, Default::default(), Default::default());
        let dom = parser.one(HTML).into_dom();

        let mut buf: Vec<u8> = Vec::new();
        html5ever::serialize::serialize(
            &mut buf,
            &Serializer::new(&dom, dom.root().id()),
            Default::default(),
        )
        .unwrap();

        assert_eq!(HTML, String::from_utf8_lossy(&buf));
    }

    #[cfg(feature = "xml5ever")]
    #[test]
    fn xml_serializer() {
        use crate::Serializer;

        let parser = ParserSink::parse_xml(Default::default());
        let dom = parser.one(XML).into_dom();

        let mut buf: Vec<u8> = Vec::new();
        xml5ever::serialize::serialize(
            &mut buf,
            &Serializer::new(&dom, dom.root().id()),
            Default::default(),
        )
        .unwrap();

        assert_eq!(
            r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE suite SYSTEM "http://testng.org/testng-1.0.dtd"><suite name="TestSuite"><test name="TestProject"><classes><class name="package.firstClassName"></class><class name="package.secondClassName"></class></classes></test></suite>"#,
            String::from_utf8_lossy(&buf)
        );
    }
}
