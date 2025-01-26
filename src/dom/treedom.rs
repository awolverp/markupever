use super::data;
use std::sync::Arc;

macro_rules! declare_node_getters {
    (
        $(
            $index_name:ident $name:ident $into_name:ident $docname:expr
        )+
    ) => {
        $(
            #[doc = "Returns the position of this node's"]
            #[doc = $docname]
            pub fn $index_name(&self) -> Option<unitree::Index> {
                self.as_item().$name()
            }

            #[doc = "Returns the node of this node's"]
            #[doc = $docname]
            #[doc = "\n- If you only want the position, use the `*_index` methods."]
            #[doc = "\n- If you don't want this node, use `into_*` methods. they have a better performance."]
            #[must_use]
            pub fn $name(&self) -> Option<Node> {
                let index = self.as_item().$name()?;
                unsafe { Some(Self::new_unchecked(self.tree.clone(), index)) }
            }

            #[doc = "Consumes this node and returns it's"]
            #[doc = $docname]
            #[doc = "\n- If you only want the position, use the `*_index` methods."]
            #[must_use]
            pub fn $into_name(self) -> Option<Node> {
                let index = self.as_item().$name()?;
                unsafe { Some(Self::new_unchecked(self.tree, index)) }
            }
        )+
    };
}

#[derive(Clone)]
pub struct Node {
    tree: Arc<parking_lot::Mutex<unitree::UNITree<data::NodeData>>>,
    index: unitree::Index,
}

impl Node {
    #[inline]
    fn new(
        tree: &Arc<parking_lot::Mutex<unitree::UNITree<data::NodeData>>>,
        index: unitree::Index,
    ) -> Option<Self> {
        let mutex = tree.lock();

        if index.into_usize() >= mutex.len() {
            return None;
        }

        std::mem::drop(mutex);
        Some(Self {
            tree: tree.clone(),
            index,
        })
    }

    #[inline(always)]
    unsafe fn new_unchecked(
        tree: Arc<parking_lot::Mutex<unitree::UNITree<data::NodeData>>>,
        index: unitree::Index,
    ) -> Self {
        Self { tree, index }
    }

    #[inline(always)]
    pub fn index(&self) -> unitree::Index {
        self.index
    }

    #[inline]
    fn as_item(&self) -> parking_lot::MappedMutexGuard<'_, unitree::Item<data::NodeData>> {
        let tree = self.tree.lock();
        parking_lot::MutexGuard::map(tree, |x| unsafe { x.get_unchecked(self.index).as_mut() })
    }

    declare_node_getters!(
        parent_index parent into_parent "parent"
        prev_sibling_index prev_sibling into_prev_sibling "previous sibling"
        next_sibling_index next_sibling into_next_sibling "next sibling"
        first_children_index first_children into_first_children "first children"
        last_children_index last_children into_last_children "last children"
    );

    pub fn value(&self) -> parking_lot::MappedMutexGuard<'_, data::NodeData> {
        let tree = self.tree.lock();
        parking_lot::MutexGuard::map(tree, |x| unsafe {
            x.get_unchecked(self.index).as_mut().value_mut()
        })
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({})", *self.value())
    }
}

#[derive(Debug)]
pub struct TreeDom {
    tree: Arc<parking_lot::Mutex<unitree::UNITree<data::NodeData>>>,
    errors: Vec<std::borrow::Cow<'static, str>>,
    quirks_mode: markup5ever::interface::QuirksMode,
    namespaces: parking_lot::Mutex<hashbrown::HashMap<markup5ever::Prefix, markup5ever::Namespace>>,
    lineno: u64,
}

macro_rules! declare_treedom_getters {
    (
        $(
            $name:ident $ret:ty
        )+
    ) => {
        $(
            pub fn $name(&self) -> &$ret {
                &self.$name
            }
        )+
    };
}

impl TreeDom {
    pub(super) fn new(
        tree: unitree::UNITree<data::NodeData>,
        errors: Vec<std::borrow::Cow<'static, str>>,
        quirks_mode: markup5ever::interface::QuirksMode,
        namespaces: hashbrown::HashMap<markup5ever::Prefix, markup5ever::Namespace>,
        lineno: u64,
    ) -> Self {
        Self {
            tree: Arc::new(parking_lot::Mutex::new(tree)),
            errors,
            quirks_mode,
            namespaces: parking_lot::Mutex::new(namespaces),
            lineno,
        }
    }

    declare_treedom_getters!(
        errors Vec<std::borrow::Cow<'static, str>>
        quirks_mode markup5ever::interface::QuirksMode
        lineno u64
    );

    pub fn namespaces(
        &self,
    ) -> parking_lot::MappedMutexGuard<
        '_,
        hashbrown::HashMap<markup5ever::Prefix, markup5ever::Namespace>,
    > {
        parking_lot::MutexGuard::map(self.namespaces.lock(), |x| x)
    }

    /// Returns a node for the root item.
    pub fn root(&self) -> Node {
        // SAFETY: root is always exists and `unitree::Index::default()` is not out of bounds.
        unsafe { Node::new_unchecked(self.tree.clone(), unitree::Index::default()) }
    }

    /// Returns a node for the item at position `index`.
    ///
    /// Returns [`None`] if the `index` is out of bounds.
    pub fn get(&self, index: unitree::Index) -> Option<Node> {
        Node::new(&self.tree, index)
    }

    /// Creates an orphan item.
    ///
    /// In simple terms, Returns an item that has no parent and children;
    /// In other words, an item which is only inserted to the UNITree-internal vector.
    pub fn orphan<D: Into<data::NodeData>>(&self, value: D) -> Node {
        let mut tree = self.tree.lock();
        let (index, _) = tree.orphan(value.into());

        std::mem::drop(tree);
        unsafe { Node::new_unchecked(self.tree.clone(), index) }
    }

    /// Appends a child to parent (push_back). It's not important child is an orphan item or not.
    pub fn append(&self, parent: &Node, child: &Node) {
        let mut tree = self.tree.lock();
        tree.append(parent.index, child.index);
    }

    /// Prepends a child to parent (push_front). It's not important child is an orphan item or not.
    pub fn prepend(&self, parent: &Node, child: &Node) {
        let mut tree = self.tree.lock();
        tree.prepend(parent.index, child.index);
    }
}

impl std::fmt::Display for TreeDom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree.lock())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    trait IsSendAndSync: Send + Sync {}
    impl IsSendAndSync for TreeDom {}

    #[test]
    fn test() {
        let dom = TreeDom::new(
            unitree::UNITree::new(data::NodeData::new(data::Document)),
            Vec::new(),
            markup5ever::interface::QuirksMode::NoQuirks,
            Default::default(),
            0,
        );

        println!("{}", dom);

        let root = dbg!(dom.root());

        let child1 = dom.orphan(data::Text::new("Hello Man 1".into()));
        let child2 = dom.orphan(data::Text::new("Hello Man 2".into()));
        dom.append(&root, &child1);
        dom.append(&root, &child2);

        println!("{:?}", child1.into_parent());
        println!("{:?}", child2.into_prev_sibling());

        println!("{}", dom);
    }
}
