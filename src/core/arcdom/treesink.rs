use super::node::CommentData;
use super::node::DoctypeData;
use super::node::DocumentData;
use super::node::ElementData;
use super::node::Node;
use super::node::ProcessingInstructionData;
use super::node::TextData;
use super::node::NamespacesHashMap;

use crate::core::send::make_atomic_tendril;

use std::cell::Cell;
use std::cell::RefCell;

/// We have to implement a clonable
#[derive(Debug, Clone)]
pub struct ClonedExpandedName {
    pub ns: markup5ever::Namespace,
    pub local: markup5ever::LocalName,
}

impl markup5ever::interface::ElemName for ClonedExpandedName {
    fn local_name(&self) -> &xml5ever::LocalName {
        &self.local
    }
    fn ns(&self) -> &xml5ever::Namespace {
        &self.ns
    }
}

impl From<markup5ever::ExpandedName<'_>> for ClonedExpandedName {
    fn from(value: markup5ever::ExpandedName<'_>) -> Self {
        Self {
            ns: value.ns.clone(),
            local: value.local.clone(),
        }
    }
}

/// ArcDom that implemented [`markup5ever::interface::TreeSink`]
#[derive(Debug)]
pub struct ArcDom {
    pub root: Node,
    pub errors: RefCell<Vec<std::borrow::Cow<'static, str>>>,
    pub quirks_mode: Cell<markup5ever::interface::QuirksMode>,
    pub namespaces: RefCell<NamespacesHashMap>,
}

impl ArcDom {
    pub fn new(root: Node) -> Self {
        Self {
            root,
            errors: RefCell::new(Vec::new()),
            quirks_mode: Cell::new(markup5ever::interface::QuirksMode::NoQuirks),
            namespaces: RefCell::new(NamespacesHashMap::new()),
        }
    }

    pub fn parse_html(
        root: Node,
        full_document: bool,
        tokenizer: html5ever::tokenizer::TokenizerOpts,
        tree_builder: html5ever::tree_builder::TreeBuilderOpts,
    ) -> html5ever::driver::Parser<Self> {
        let opts = html5ever::driver::ParseOpts {
            tokenizer,
            tree_builder,
        };

        if full_document {
            html5ever::driver::parse_document(Self::new(root), opts)
        } else {
            html5ever::driver::parse_fragment(
                Self::new(root),
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
        root: Node,
        tokenizer: xml5ever::tokenizer::XmlTokenizerOpts,
    ) -> xml5ever::driver::XmlParser<Self> {
        let opts = xml5ever::driver::XmlParseOpts {
            tokenizer,
            tree_builder: Default::default(),
        };

        xml5ever::driver::parse_document(Self::new(root), opts)
    }
}

impl Default for ArcDom {
    fn default() -> Self {
        Self::new(Node::new(DocumentData))
    }
}

impl markup5ever::interface::TreeSink for ArcDom {
    type Handle = Node;
    type Output = Self;
    type ElemName<'a> = ClonedExpandedName;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
        self.errors.borrow_mut().push(msg);
    }

    fn set_current_line(&self, _line_number: u64) {}

    fn get_document(&self) -> Self::Handle {
        self.root.clone()
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        if !target
            .as_element()
            .expect("target is not a element")
            .template
        {
            unreachable!("target is not a template");
        }

        target.clone()
    }

    fn set_quirks_mode(&self, mode: markup5ever::interface::QuirksMode) {
        self.quirks_mode.set(mode);
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.ptr_eq(y)
    }

    fn elem_name<'a>(&self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        let element = target.as_element().expect("target is not a element");
        element.name.expanded().into()
    }

    fn create_element(
        &self,
        name: markup5ever::QualName,
        attrs: Vec<markup5ever::Attribute>,
        flags: markup5ever::interface::ElementFlags,
    ) -> Self::Handle {
        if let Some(ref prefix) = name.prefix {
            self.namespaces
                .borrow_mut()
                .insert(prefix.clone(), name.ns.clone());
        }

        let mut elem = ElementData::from_non_atomic(
            name,
            attrs.into_iter().map(|x| (x.name, x.value)),
            flags.template,
            flags.mathml_annotation_xml_integration_point,
        );

        elem.attrs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        elem.attrs.dedup();

        Node::new(elem)
    }

    fn create_comment(&self, text: tendril::StrTendril) -> Self::Handle {
        Node::new(CommentData::from_non_atomic(text))
    }

    fn create_pi(&self, target: tendril::StrTendril, data: tendril::StrTendril) -> Self::Handle {
        Node::new(ProcessingInstructionData::from_non_atomic(data, target))
    }

    fn append_doctype_to_document(
        &self,
        name: tendril::StrTendril,
        public_id: tendril::StrTendril,
        system_id: tendril::StrTendril,
    ) {
        let d = Node::new(DoctypeData::from_non_atomic(name, public_id, system_id));
        unsafe { self.root.children().push(d).unwrap_unchecked() };
    }

    fn append(
        &self,
        parent: &Self::Handle,
        child: markup5ever::interface::NodeOrText<Self::Handle>,
    ) {
        match child {
            markup5ever::interface::NodeOrText::AppendNode(handle) => {
                parent.children().push(handle).unwrap();
            }
            markup5ever::interface::NodeOrText::AppendText(text) => {
                let mut c = parent.children();

                if let Some(last) = c.last() {
                    if let Some(mut last_text) = last.as_text() {
                        last_text.push_non_atomic(text);
                        return;
                    }
                }

                c.push(Node::new(TextData::from_non_atomic(text))).unwrap();
            }
        }
    }

    fn append_before_sibling(
        &self,
        sibling: &Self::Handle,
        new_node: markup5ever::interface::NodeOrText<Self::Handle>,
    ) {
        let parent = sibling.parent();

        if parent.is_none() {
            unreachable!("sibling has no parent");
        }

        let parent = parking_lot::MappedMutexGuard::map(parent, |x| unsafe {
            x.as_mut().unwrap_unchecked()
        });

        // drop guard, clone and upgrade parent
        let parent = parent.clone().upgrade().expect("dangling weak pointer");

        let mut p_children = parent.children();

        let index = p_children
            .iter()
            .position(|x| x.ptr_eq(sibling))
            .expect("have parent but couldn't find in parent's children!");

        let new_node = match (new_node, index) {
            (markup5ever::interface::NodeOrText::AppendText(text), 0) => {
                Node::new(TextData::from_non_atomic(text))
            }

            (markup5ever::interface::NodeOrText::AppendText(text), index) => {
                let c = parent.children();

                if let Some(mut last_text) = c[index - 1].as_text() {
                    last_text.push_non_atomic(text);
                    return;
                }

                Node::new(TextData::from_non_atomic(text))
            }

            (markup5ever::interface::NodeOrText::AppendNode(node), _) => {
                // unlink node from its parent
                if let Some(oldparent) = node.parent().take() {
                    let oldparent = oldparent.upgrade().expect("dangling weak pointer");
                    let mut oldchildren = oldparent.children();
                    oldchildren.remove(oldchildren.iter().position(|x| x.ptr_eq(&node)).unwrap());
                }

                node
            }
        };

        p_children.insert(index, new_node).unwrap();
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: xml5ever::interface::NodeOrText<Self::Handle>,
    ) {
        if element.parent().is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<xml5ever::Attribute>) {
        let mut elem = target
            .as_element()
            .expect("add_attrs_if_missing called on a non-element node");

        elem.attrs.extend(
            attrs
                .into_iter()
                .map(|x| (x.name, make_atomic_tendril(x.value))),
        );
        elem.attrs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        elem.attrs.dedup();
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        if let Some(oldparent) = target.parent().take() {
            let oldparent = oldparent.upgrade().expect("dangling weak pointer");
            let mut oldchildren = oldparent.children();
            oldchildren.remove(oldchildren.iter().position(|x| x.ptr_eq(target)).unwrap());
        }
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let mut c = new_parent.children();

        for child in unsafe { node.children().drain(..) } {
            child.parent().take();
            c.push(child).unwrap();
        }
    }

    fn is_mathml_annotation_xml_integration_point(&self, _handle: &Self::Handle) -> bool {
        _handle
            .as_element()
            .expect("is_mathml_annotation_xml_integration_point called on a non-element node")
            .mathml_annotation_xml_integration_point
    }
}

unsafe impl Send for ArcDom {}
unsafe impl Sync for ArcDom {}
