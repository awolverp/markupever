use super::data;
use super::treedom::TreeDom;
use std::cell::{Cell, UnsafeCell};

pub struct Parser {
    tree: UnsafeCell<unitree::UNITree<data::NodeData>>,
    errors: UnsafeCell<Vec<std::borrow::Cow<'static, str>>>,
    quirks_mode: Cell<markup5ever::interface::QuirksMode>,
    namespaces: UnsafeCell<hashbrown::HashMap<markup5ever::Prefix, markup5ever::Namespace>>,
    lineno: UnsafeCell<u64>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    /// Creates a new [`Parser`]
    pub fn new() -> Self {
        Self {
            tree: UnsafeCell::new(unitree::UNITree::new(data::NodeData::new(data::Document))),
            errors: UnsafeCell::new(Vec::new()),
            quirks_mode: Cell::new(markup5ever::interface::QuirksMode::NoQuirks),
            namespaces: UnsafeCell::new(hashbrown::HashMap::new()),
            lineno: UnsafeCell::new(0),
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn tree_mut(&self) -> &mut unitree::UNITree<data::NodeData> {
        // SAFETY: Parser is not Send/Sync so cannot be used in multi threads.
        unsafe { &mut *self.tree.get() }
    }

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

impl markup5ever::interface::TreeSink for Parser {
    type Output = TreeDom;
    type Handle = unitree::Index;
    type ElemName<'a> = markup5ever::ExpandedName<'a>;

    // Consume this sink and return the overall result of parsing.
    fn finish(self) -> Self::Output {
        TreeDom::new(
            self.tree.into_inner(),
            self.errors.into_inner(),
            self.quirks_mode.into_inner(),
            self.namespaces.into_inner(),
            self.lineno.into_inner(),
        )
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
        unitree::Index::default()
    }

    // Get a handle to a template's template contents.
    // The tree builder promises this will never be called with something else than a template element.
    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        let item = self.tree_mut().get(*target).unwrap();

        if let Some(x) = unsafe { item.as_ref().value().element() } {
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
        let x = self.tree_mut().get(*x).unwrap();
        let y = self.tree_mut().get(*y).unwrap();

        std::ptr::addr_eq(x.as_ptr(), y.as_ptr())
    }

    // What is the name of this element?
    //
    // Should never be called on a non-element node; feel free to panic!.
    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        let item = self.tree_mut().get(*target).unwrap();

        if let Some(x) = unsafe { item.as_ref().value().element() } {
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
        if let Some(ref prefix) = name.prefix {
            unsafe {
                (*self.namespaces.get()).insert(prefix.clone(), name.ns.clone());
            }
        }

        let mut element = data::Element::from_non_atomic(
            name,
            attrs.into_iter().map(|x| (x.name, x.value)),
            flags.template,
            flags.mathml_annotation_xml_integration_point,
        );

        element.attrs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        element.attrs.dedup();

        let (index, _) = self.tree_mut().orphan(data::NodeData::new(element));
        index
    }

    // Create a comment node.
    fn create_comment(&self, text: tendril::StrTendril) -> Self::Handle {
        let (index, _) = self
            .tree_mut()
            .orphan(data::NodeData::new(data::Comment::from_non_atomic(text)));

        index
    }

    // Create a Processing Instruction node.
    fn create_pi(&self, target: tendril::StrTendril, data: tendril::StrTendril) -> Self::Handle {
        let (index, _) = self.tree_mut().orphan(data::NodeData::new(
            data::ProcessingInstruction::from_non_atomic(data, target),
        ));

        index
    }

    // Append a DOCTYPE element to the Document node.
    fn append_doctype_to_document(
        &self,
        name: tendril::StrTendril,
        public_id: tendril::StrTendril,
        system_id: tendril::StrTendril,
    ) {
        let doctype =
            data::NodeData::new(data::Doctype::from_non_atomic(name, public_id, system_id));
        let (index, _) = self.tree_mut().orphan(doctype);
        self.tree_mut().prepend(unitree::Index::default(), index);
    }

    // Append a node as the last child of the given node. If this would produce adjacent sibling text nodes, it should concatenate the text instead.
    //
    // The child node will not already have a parent.
    fn append(
        &self,
        parent: &Self::Handle,
        child: markup5ever::interface::NodeOrText<Self::Handle>,
    ) {
        match child {
            markup5ever::interface::NodeOrText::AppendNode(handle) => {
                self.tree_mut().append(*parent, handle);
            }
            markup5ever::interface::NodeOrText::AppendText(text) => {
                let parent_item = self.tree_mut().get(*parent).unwrap();

                if let Some(last_index) = unsafe { parent_item.as_ref().last_children() } {
                    let mut last_child = self.tree_mut().get(last_index).unwrap();

                    if let Some(textdata) = unsafe { last_child.as_mut().value_mut().text_mut() } {
                        textdata.push_non_atomic(text);
                        return;
                    }
                }

                let (text_index, _) = self
                    .tree_mut()
                    .orphan(data::NodeData::new(data::Text::from_non_atomic(text)));

                self.tree_mut().append(*parent, text_index);
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
        sibling: &Self::Handle,
        new_item: markup5ever::interface::NodeOrText<Self::Handle>,
    ) {
        let sibling_item = self.tree_mut().get(*sibling).unwrap();

        match (new_item, unsafe { sibling_item.as_ref().prev_sibling() }) {
            (markup5ever::interface::NodeOrText::AppendText(text), None) => {
                // There's no previous item, so we have to create a Text node data
                let (text_index, _) = self
                    .tree_mut()
                    .orphan(data::NodeData::new(data::Text::from_non_atomic(text)));

                self.tree_mut().insert_before(*sibling, text_index);
            }
            (markup5ever::interface::NodeOrText::AppendText(text), Some(prev_index)) => {
                // There's a previous item, so may it's a Text node data? we have to check
                let mut prev_item = self.tree_mut().get(prev_index).unwrap();

                if let Some(textdata) = unsafe { prev_item.as_mut().value_mut().text_mut() } {
                    textdata.push_non_atomic(text);
                } else {
                    let (text_index, _) = self
                        .tree_mut()
                        .orphan(data::NodeData::new(data::Text::from_non_atomic(text)));

                    self.tree_mut().insert_before(*sibling, text_index);
                }
            }
            (markup5ever::interface::NodeOrText::AppendNode(item_index), _) => {
                self.tree_mut().insert_before(*sibling, item_index);
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

        if unsafe { item.as_ref().parent().is_some() } {
            self.append_before_sibling(item_index, child);
        } else {
            self.append(prev_item_index, child);
        }
    }

    // Add each attribute to the given element, if no attribute with that name already exists.
    // The tree builder promises this will never be called with something else than an element.
    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<markup5ever::Attribute>) {
        let mut item = self.tree_mut().get(*target).unwrap();

        if let Some(element) = unsafe { item.as_mut().value_mut().element_mut() } {
            element.attrs.extend(
                attrs
                    .into_iter()
                    .map(|x| (x.name, crate::send::make_atomic_tendril(x.value))),
            );
            element.attrs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            element.attrs.dedup();
        } else {
            unreachable!("add_attrs_if_missing called on a non-element node")
        }
    }

    // Detach the given node from its parent.
    fn remove_from_parent(&self, target: &Self::Handle) {
        self.tree_mut().detach(*target);
    }

    // Remove all the children from node and append them to new_parent.
    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        self.tree_mut().reparent_append(*new_parent, *node);
    }

    // Returns true if the adjusted current node is an HTML integration point and the token is a start tag.
    fn is_mathml_annotation_xml_integration_point(&self, target: &Self::Handle) -> bool {
        let item = self.tree_mut().get(*target).unwrap();

        if let Some(x) = unsafe { item.as_ref().value().element() } {
            x.mathml_annotation_xml_integration_point
        } else {
            unreachable!("target is not a element");
        }
    }
}
