use super::interface;
use hashbrown::HashMap;
use std::ops::Deref;

pub type NamespaceMap = HashMap<markup5ever::Prefix, markup5ever::Namespace>;

/// A DOM based on [`ego_tree::Tree`]
#[derive(PartialEq, Eq)]
pub struct IDTreeDOM {
    pub(super) tree: ego_tree::Tree<interface::Interface>,
    pub(super) namespaces: NamespaceMap,
}

impl IDTreeDOM {
    /// Creates a new [`IDTreeDOM`].
    ///
    /// Use [`IDTreeDOM::default`] if you don't want to specify this parameters.
    pub fn new<T: Into<interface::Interface>>(root: T, namespaces: NamespaceMap) -> Self {
        Self {
            tree: ego_tree::Tree::new(root.into()),
            namespaces,
        }
    }

    pub fn with_capacity<T: Into<interface::Interface>>(
        root: T,
        namespaces: NamespaceMap,
        capacity: usize,
    ) -> Self {
        Self {
            tree: ego_tree::Tree::with_capacity(root.into(), capacity),
            namespaces,
        }
    }

    pub fn namespaces(&self) -> &NamespaceMap {
        &self.namespaces
    }

    pub fn namespaces_mut(&mut self) -> &mut NamespaceMap {
        &mut self.namespaces
    }
}

impl std::ops::Deref for IDTreeDOM {
    type Target = ego_tree::Tree<interface::Interface>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl std::ops::DerefMut for IDTreeDOM {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

impl Default for IDTreeDOM {
    fn default() -> Self {
        Self::new(interface::DocumentInterface, NamespaceMap::new())
    }
}

impl std::fmt::Debug for IDTreeDOM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.tree)?;
        } else {
            f.debug_struct("IDTreeDOM")
                .field("tree", &self.tree)
                .field("namespaces", &self.namespaces)
                .finish()?;
        }

        Ok(())
    }
}

impl std::fmt::Display for IDTreeDOM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree)
    }
}

/// A serializer for [`IDTreeDOM`]
pub struct Serializer<'a> {
    dom: &'a IDTreeDOM,
    id: ego_tree::NodeId,
}

impl<'a> Serializer<'a> {
    pub fn new(dom: &'a IDTreeDOM, id: ego_tree::NodeId) -> Self {
        Self { dom, id }
    }

    fn serialize_iter<S>(
        &self,
        iter: impl Iterator<Item = ego_tree::iter::Edge<'a, interface::Interface>>,
        serializer: &mut S,
    ) -> std::io::Result<()>
    where
        S: markup5ever::serialize::Serializer,
    {
        for edge in iter {
            match edge {
                ego_tree::iter::Edge::Close(x) => {
                    if let Some(element) = x.value().element() {
                        serializer.end_elem(element.name.clone())?;
                    }
                }
                ego_tree::iter::Edge::Open(x) => match x.value() {
                    interface::Interface::Comment(comment) => {
                        serializer.write_comment(&comment.contents)?
                    }
                    interface::Interface::Doctype(doctype) => {
                        let mut docname = String::from(&doctype.name);
                        if !doctype.public_id.is_empty() {
                            docname.push_str(" PUBLIC \"");
                            docname.push_str(&doctype.public_id);
                            docname.push('"');
                        }
                        if !doctype.system_id.is_empty() {
                            docname.push_str(" SYSTEM \"");
                            docname.push_str(&doctype.system_id);
                            docname.push('"');
                        }

                        serializer.write_doctype(&docname)?
                    }
                    interface::Interface::Element(element) => serializer.start_elem(
                        element.name.clone(),
                        element.attrs.iter().map(|at| (at.0.deref(), &at.1[..])),
                    )?,
                    interface::Interface::ProcessingInstruction(pi) => {
                        serializer.write_processing_instruction(&pi.target, &pi.data)?
                    }
                    interface::Interface::Text(text) => serializer.write_text(&text.contents)?,
                    interface::Interface::Document(_) => (),
                },
            }
        }

        Ok(())
    }
}

fn skip_last<T>(mut iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    let last = iter.next();
    iter.scan(last, |state, item| std::mem::replace(state, Some(item)))
}

impl markup5ever::serialize::Serialize for Serializer<'_> {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        traversal_scope: markup5ever::serialize::TraversalScope,
    ) -> std::io::Result<()>
    where
        S: markup5ever::serialize::Serializer,
    {
        let mut traverse = unsafe { self.dom.tree.get_unchecked(self.id).traverse() };

        if let markup5ever::serialize::TraversalScope::ChildrenOnly(_) = traversal_scope {
            traverse.next();
            self.serialize_iter(skip_last(traverse), serializer)
        } else {
            self.serialize_iter(traverse, serializer)
        }
    }
}
