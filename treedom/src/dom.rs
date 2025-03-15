use super::interface;
use hashbrown::HashMap;
use std::{fmt::Write, ops::Deref};

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

struct IndentToken {
    indent: usize,
    nested: usize,
    ignore_end: usize,
    newline: bool,
}

impl IndentToken {
    fn new(indent: usize) -> Self {
        Self {
            indent,
            nested: 0,
            ignore_end: 0,
            newline: false,
        }
    }

    fn write_to<S>(&mut self, serializer: &mut S) -> std::io::Result<()>
    where
        S: markup5ever::serialize::Serializer,
    {
        if self.ignore_end > 0 {
            self.ignore_end -= 1;
            return Ok(());
        }

        let count = self.indent * self.nested;

        if count == 0 {
            if self.newline {
                serializer.write_text("\n")?;
                self.newline = false;
            }

            return Ok(());
        }

        let mut f = String::from("\n");

        let mut m = 0;
        while m < count {
            f.write_char(' ').unwrap();
            m += 1;
        }

        serializer.write_text(&f)
    }

    fn start<S: markup5ever::serialize::Serializer>(
        &mut self,
        serializer: &mut S,
    ) -> std::io::Result<()> {
        if self.ignore_end > 0 {
            self.ignore_end -= 1;
        }

        self.write_to(serializer)?;

        self.nested += 1;
        self.newline = self.indent > 0;
        Ok(())
    }

    fn end<S: markup5ever::serialize::Serializer>(
        &mut self,
        serializer: &mut S,
    ) -> std::io::Result<()> {
        self.nested -= 1;
        self.write_to(serializer)
    }

    fn ignore_end(&mut self) {
        self.ignore_end += 1;
    }
}

/// A serializer for [`IDTreeDOM`]
pub struct Serializer<'a> {
    dom: &'a IDTreeDOM,
    id: ego_tree::NodeId,
    indent: usize,
}

impl<'a> Serializer<'a> {
    pub fn new(dom: &'a IDTreeDOM, id: ego_tree::NodeId, indent: usize) -> Self {
        Self { dom, id, indent }
    }

    fn serialize_iter<S>(
        &self,
        iter: impl Iterator<Item = ego_tree::iter::Edge<'a, interface::Interface>>,
        serializer: &mut S,
    ) -> std::io::Result<()>
    where
        S: markup5ever::serialize::Serializer,
    {
        let mut indentation = IndentToken::new(self.indent);
        let mut last_element_name = markup5ever::LocalName::from("");

        for edge in iter {
            match edge {
                ego_tree::iter::Edge::Close(x) => {
                    if let interface::Interface::Element(element) = x.value() {
                        if last_element_name == element.name.local && indentation.ignore_end == 0 {
                            indentation.ignore_end();
                        }

                        indentation.end(serializer)?;
                        serializer.end_elem(element.name.clone())?;
                    }
                }
                ego_tree::iter::Edge::Open(x) => match x.value() {
                    interface::Interface::Comment(comment) => {
                        serializer.write_comment(&comment.contents)?;
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

                        serializer.write_doctype(&docname)?;

                        indentation.newline = indentation.indent > 0;
                        indentation.write_to(serializer)?;
                        indentation.newline = false;
                    }
                    interface::Interface::Element(element) => {
                        last_element_name = element.name.local.clone();

                        indentation.start(serializer)?;
                        serializer.start_elem(
                            element.name.clone(),
                            element.attrs.iter().map(|at| (at.0.deref(), &at.1[..])),
                        )?;
                    }
                    interface::Interface::ProcessingInstruction(pi) => {
                        serializer.write_processing_instruction(&pi.target, &pi.data)?
                    }
                    interface::Interface::Text(text) => {
                        if !text.contents.trim_ascii().is_empty() {
                            serializer.write_text(text.contents.trim_ascii_end())?;
                        }
                    }
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
