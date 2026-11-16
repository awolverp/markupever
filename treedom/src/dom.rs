use super::interface;
use std::{fmt::Write, ops::Deref};

/// Prefix bindings that are still used, with a count.
///
/// One prefix can be bound to more than one namespace. [`NamespaceMap::iter`]
/// yields the first binding for each prefix, which is what `TreeDom.namespaces()`
/// exposes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NamespaceMap {
    bindings: Vec<(markup5ever::Prefix, markup5ever::Namespace, usize)>,
}

impl NamespaceMap {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn contains_prefix(&self, prefix: &markup5ever::Prefix) -> bool {
        self.bindings.iter().any(|binding| &binding.0 == prefix)
    }

    /// The first namespace still bound to `prefix`.
    pub fn get(&self, prefix: &markup5ever::Prefix) -> Option<&markup5ever::Namespace> {
        self.bindings
            .iter()
            .find(|binding| &binding.0 == prefix)
            .map(|binding| &binding.1)
    }

    pub fn uris(&self, prefix: &markup5ever::Prefix) -> Vec<markup5ever::Namespace> {
        self.bindings
            .iter()
            .filter(|binding| &binding.0 == prefix)
            .map(|binding| binding.1.clone())
            .collect()
    }

    /// Record one more element using this binding.
    pub fn insert(&mut self, prefix: markup5ever::Prefix, namespace: markup5ever::Namespace) {
        if let Some(binding) = self
            .bindings
            .iter_mut()
            .find(|binding| binding.0 == prefix && binding.1 == namespace)
        {
            binding.2 += 1;
            return;
        }
        self.bindings.push((prefix, namespace, 1));
    }

    /// Forget one element using this binding.
    pub fn remove_binding(
        &mut self,
        prefix: &markup5ever::Prefix,
        namespace: &markup5ever::Namespace,
    ) {
        let Some(index) = self
            .bindings
            .iter()
            .position(|binding| &binding.0 == prefix && &binding.1 == namespace)
        else {
            return;
        };
        self.bindings[index].2 -= 1;
        if self.bindings[index].2 == 0 {
            self.bindings.remove(index);
        }
    }

    pub fn iter(&self) -> Vec<(&markup5ever::Prefix, &markup5ever::Namespace)> {
        let mut unique = Vec::new();
        for binding in &self.bindings {
            if unique.iter().any(|(prefix, _)| *prefix == &binding.0) {
                continue;
            }
            unique.push((&binding.0, &binding.1));
        }
        unique
    }
}

/// A DOM based on [`ego_tree::Tree`]
#[derive(PartialEq, Eq)]
pub struct IDTreeDOM {
    pub(super) tree: ego_tree::Tree<interface::Interface>,
    pub(super) namespaces: NamespaceMap,
    pub(super) fragment_context: Option<markup5ever::QualName>,
    pub(super) quirks_mode: markup5ever::interface::QuirksMode,
    /// True when this tree was parsed as HTML. Hand-built trees stay false
    /// until a later step marks them.
    pub(super) html_document: bool,
}

impl IDTreeDOM {
    /// Creates a new [`IDTreeDOM`].
    ///
    /// Use [`IDTreeDOM::default`] if you don't want to specify this parameters.
    pub fn new<T: Into<interface::Interface>>(root: T, namespaces: NamespaceMap) -> Self {
        Self {
            tree: ego_tree::Tree::new(root.into()),
            namespaces,
            fragment_context: None,
            quirks_mode: markup5ever::interface::QuirksMode::NoQuirks,
            html_document: false,
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
            fragment_context: None,
            quirks_mode: markup5ever::interface::QuirksMode::NoQuirks,
            html_document: false,
        }
    }

    pub fn namespaces(&self) -> &NamespaceMap {
        &self.namespaces
    }

    pub fn namespaces_mut(&mut self) -> &mut NamespaceMap {
        &mut self.namespaces
    }

    /// Returns the context element that this DOM was parsed in, if it was parsed as a fragment.
    /// Its root's children are the fragment's nodes.
    pub fn fragment_context(&self) -> Option<&markup5ever::QualName> {
        self.fragment_context.as_ref()
    }

    pub fn quirks_mode(&self) -> markup5ever::interface::QuirksMode {
        self.quirks_mode
    }

    pub fn html_document(&self) -> bool {
        self.html_document
    }

    pub fn set_html_document(&mut self, html_document: bool) {
        self.html_document = html_document;
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
                .field("fragment_context", &self.fragment_context)
                .field("quirks_mode", &self.quirks_mode)
                .field("html_document", &self.html_document)
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
        let mut last_opened: Option<ego_tree::NodeId> = None;

        for edge in iter {
            match edge {
                ego_tree::iter::Edge::Close(x) => {
                    if let interface::Interface::Element(element) = x.value() {
                        // Suppress the break only for the element that just
                        // opened, so an empty tag stays on one line. A
                        // same-named descendant must not eat the outer close.
                        if last_opened == Some(x.id()) && indentation.ignore_end == 0 {
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
                        serializer.write_doctype(&doctype_declaration(
                            &doctype.name,
                            &doctype.public_id,
                            &doctype.system_id,
                        ))?;

                        indentation.newline = indentation.indent > 0;
                        indentation.write_to(serializer)?;
                        indentation.newline = false;
                    }
                    interface::Interface::Element(element) => {
                        last_opened = Some(x.id());

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
                        // Without indentation, whitespace is significant: write it unchanged.
                        if self.indent == 0 {
                            serializer.write_text(&text.contents)?;
                        } else if !text.contents.trim_ascii().is_empty() {
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

/// HTML doctype serialization. A public id and a system id are
/// `PUBLIC "pub" "sys"`, not `PUBLIC "pub" SYSTEM "sys"`.
fn doctype_declaration(name: &str, public_id: &str, system_id: &str) -> String {
    let mut declaration = escape_doctype(name);
    if !public_id.is_empty() {
        declaration.push_str(" PUBLIC \"");
        declaration.push_str(&escape_doctype(public_id));
        declaration.push('"');
        if !system_id.is_empty() {
            declaration.push_str(" \"");
            declaration.push_str(&escape_doctype(system_id));
            declaration.push('"');
        }
    } else if !system_id.is_empty() {
        declaration.push_str(" SYSTEM \"");
        declaration.push_str(&escape_doctype(system_id));
        declaration.push('"');
    }
    declaration
}

fn escape_doctype(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn skip_last<T>(mut iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    let last = iter.next();
    iter.scan(last, |state, item| state.replace(item))
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
        let mut traverse = self.dom.tree.get(self.id).unwrap().traverse();

        if let markup5ever::serialize::TraversalScope::ChildrenOnly(_) = traversal_scope {
            traverse.next();
            self.serialize_iter(skip_last(traverse), serializer)
        } else {
            self.serialize_iter(traverse, serializer)
        }
    }
}
