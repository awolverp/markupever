use super::data;

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;
#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

pub type NamespaceMap = HashMap<markup5ever::Prefix, markup5ever::Namespace>;

/// A DOM based on [`ego_tree::Tree`]
pub struct TreeDom {
    tree: ego_tree::Tree<data::NodeData>,
    namespaces: NamespaceMap,
}

macro_rules! declare_treedom_getters {
    (
        $(
            $name:ident $mutname:ident $ret:ty
        )+
    ) => {
        $(
            pub fn $name(&self) -> &$ret {
                &self.$name
            }

            pub fn $mutname(&mut self) -> &mut $ret {
                &mut self.$name
            }
        )+
    };
}

impl TreeDom {
    /// Creates a new [`TreeDom`].
    ///
    /// Use [`TreeDom::default`] if you don't want to specify this parameters.
    pub fn new(tree: ego_tree::Tree<data::NodeData>, namespaces: NamespaceMap) -> Self {
        Self { tree, namespaces }
    }

    declare_treedom_getters!(
        namespaces namespaces_mut NamespaceMap
    );

    /// Returns a reference to the root node.
    pub fn root(&self) -> ego_tree::NodeRef<'_, data::NodeData> {
        self.tree.root()
    }

    /// Returns a mutable reference to the root node.
    pub fn root_mut(&mut self) -> ego_tree::NodeMut<'_, data::NodeData> {
        self.tree.root_mut()
    }

    /// Returns a reference to the specified node.
    pub fn get(&self, id: ego_tree::NodeId) -> Option<ego_tree::NodeRef<'_, data::NodeData>> {
        self.tree.get(id)
    }

    /// Returns a mutator of the specified node.
    pub fn get_mut(
        &mut self,
        id: ego_tree::NodeId,
    ) -> Option<ego_tree::NodeMut<'_, data::NodeData>> {
        self.tree.get_mut(id)
    }
}

impl Default for TreeDom {
    fn default() -> Self {
        Self::new(
            ego_tree::Tree::new(data::Document.into()),
            NamespaceMap::new(),
        )
    }
}

impl std::fmt::Debug for TreeDom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.tree)?;
        } else {
            f.debug_struct("TreeDom")
                .field("tree", &self.tree)
                .field("namespaces", &self.namespaces)
                .finish()?;
        }

        Ok(())
    }
}

impl std::fmt::Display for TreeDom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree)
    }
}

pub struct Serializer<'a> {
    dom: &'a TreeDom,
    id: ego_tree::NodeId,
}

impl<'a> Serializer<'a> {
    pub fn new(dom: &'a TreeDom, id: ego_tree::NodeId) -> Self {
        Self { dom, id }
    }
}

impl<'a> markup5ever::serialize::Serialize for Serializer<'a> {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        traversal_scope: markup5ever::serialize::TraversalScope,
    ) -> std::io::Result<()>
    where
        S: markup5ever::serialize::Serializer,
    {
        let mut skipped = false;

        for edge in unsafe { self.dom.tree.get_unchecked(self.id).traverse() } {
            if let markup5ever::serialize::TraversalScope::ChildrenOnly(_) = traversal_scope {
                if !skipped {
                    skipped = true;
                    continue;
                }
            }

            match edge {
                ego_tree::iter::Edge::Close(x) => {
                    if let Some(element) = x.value().element() {
                        serializer.end_elem(element.name.clone())?;
                    }
                }
                ego_tree::iter::Edge::Open(x) => match x.value() {
                    data::NodeData::Comment(comment) => {
                        serializer.write_comment(&comment.contents)?
                    }
                    data::NodeData::Doctype(doctype) => {
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
                    data::NodeData::Element(element) => serializer.start_elem(
                        element.name.clone(),
                        element.attrs.iter().map(|at| (&at.0, &at.1[..])),
                    )?,
                    data::NodeData::ProcessingInstruction(pi) => {
                        serializer.write_processing_instruction(&pi.target, &pi.data)?
                    }
                    data::NodeData::Text(text) => serializer.write_text(&text.contents)?,
                    data::NodeData::Document(_) => (),
                },
            }
        }

        Ok(())
    }
}
