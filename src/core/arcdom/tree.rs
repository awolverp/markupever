use crate::core::send::make_atomic_tendril;

use super::node::DocumentData;
use super::node::ElementData;
use super::node::Node;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::UnsafeCell;

#[derive(Debug, Clone)]
pub struct ErrorWithLine(pub std::borrow::Cow<'static, str>, pub u64);

impl std::fmt::Display for ErrorWithLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] {}", self.1, self.0)
    }
}

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

struct SyncCells {
    /// The errors list
    errors: RefCell<Vec<ErrorWithLine>>,

    /// The quirks mode
    quirks_mode: Cell<markup5ever::interface::QuirksMode>,

    /// Line counter
    lineno: UnsafeCell<u64>,
}

unsafe impl Send for SyncCells {}
unsafe impl Sync for SyncCells {}

/// DOM tree builder & parser
pub struct TreeBuilder {
    /// The root node
    pub root: Node,

    /// The errors list
    other: SyncCells,
}

impl TreeBuilder {
    pub fn new(root: Node) -> Self {
        Self {
            root,
            other: SyncCells {
                errors: RefCell::new(Vec::new()),
                quirks_mode: Cell::new(markup5ever::interface::QuirksMode::NoQuirks),
                lineno: UnsafeCell::new(0),
            },
        }
    }

    pub fn errors(&self) -> &RefCell<Vec<ErrorWithLine>> {
        &self.other.errors
    }

    pub fn quirks_mode(&self) -> &Cell<markup5ever::interface::QuirksMode> {
        &self.other.quirks_mode
    }

    pub fn lineno(&self) -> &UnsafeCell<u64> {
        &self.other.lineno
    }
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new(Node::new(DocumentData))
    }
}

impl markup5ever::interface::TreeSink for TreeBuilder {
    type Handle = Node;
    type Output = Self;
    type ElemName<'a> = ClonedExpandedName;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
        self.other
            .errors
            .borrow_mut()
            .push(ErrorWithLine(msg, unsafe { *self.other.lineno.get() }));
    }

    fn set_current_line(&self, _line_number: u64) {
        unsafe { *self.other.lineno.get() = _line_number }
    }

    fn get_document(&self) -> Self::Handle {
        self.root.clone()
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        let element = target.as_element().expect("target is not a element");

        if !element.template {
            unreachable!("target is not a template");
        }

        _ = element;
        target.clone()
    }

    fn set_quirks_mode(&self, mode: markup5ever::interface::QuirksMode) {
        self.other.quirks_mode.set(mode);
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
        Node::new(ElementData::new(
            name,
            attrs
                .into_iter()
                .map(|x| (x.name, make_atomic_tendril(x.value)))
                .collect(),
            flags.template,
            flags.mathml_annotation_xml_integration_point,
        ))

        // if flags.template {
        //     node.children().append(Node::new(NodeData::Fragment));
        // }
    }

    fn create_comment(&self, text: tendril::StrTendril) -> Self::Handle {
        Node::new(super::node::CommentData::new(text))
    }

    fn create_pi(&self, target: tendril::StrTendril, data: tendril::StrTendril) -> Self::Handle {
        Node::new(super::node::ProcessingInstructionData::new(data, target))
    }

    fn append_doctype_to_document(
        &self,
        name: tendril::StrTendril,
        public_id: tendril::StrTendril,
        system_id: tendril::StrTendril,
    ) {
        let d = Node::new(super::node::DoctypeData::new(name, public_id, system_id));
        self.root.children().append(d);
    }

    fn append(
        &self,
        parent: &Self::Handle,
        child: markup5ever::interface::NodeOrText<Self::Handle>,
    ) {
        match child {
            markup5ever::interface::NodeOrText::AppendNode(handle) => {
                parent.children().append(handle);
            }
            markup5ever::interface::NodeOrText::AppendText(text) => {
                let mut c = parent.children();
                if let Some(last) = c.last() {
                    if let Some(mut last_text) = last.as_text() {
                        last_text.contents.push_tendril(&make_atomic_tendril(text));
                        return;
                    }
                }

                c.append(Node::new(super::node::TextData::new(text)));
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
            .position(sibling)
            .expect("have parent but couldn't find in parent's children!");

        let new_node = match (new_node, index) {
            (markup5ever::interface::NodeOrText::AppendText(text), 0) => {
                Node::new(super::node::TextData::new(text))
            }

            (markup5ever::interface::NodeOrText::AppendText(text), index) => {
                let c = parent.children();

                if let Some(mut last_text) = c[index - 1].as_text() {
                    last_text.contents.push_tendril(&make_atomic_tendril(text));
                    return;
                }

                Node::new(super::node::TextData::new(text))
            }

            (markup5ever::interface::NodeOrText::AppendNode(node), _) => {
                // unlink node from its parent
                node.unlink();
                node
            }
        };

        p_children.insert(index, new_node);
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
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        target.unlink();
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let mut c = new_parent.children();
        for child in node.children().drain(..) {
            unsafe { child.unlink_parent() };
            c.append(child);
        }
    }

    fn is_mathml_annotation_xml_integration_point(&self, _handle: &Self::Handle) -> bool {
        _handle
            .as_element()
            .expect("is_mathml_annotation_xml_integration_point called on a non-element node")
            .mathml_annotation_xml_integration_point
    }
}
