use super::data;
use std::sync::Arc;

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;
#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

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

/// A node that keeps a `Arc<parking_lot::Mutex<unitree::UNITree<data::NodeData>>>`, and an item's index
/// for having access to that item.
///
/// By this way, we don't have any issues about memory allocations and [`Send`]/[`Sync`] topics.
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

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        if !Arc::ptr_eq(&self.tree, &other.tree) {
            return false;
        }

        let tree = self.tree.lock();
        tree.get(self.index).eq(&tree.get(other.index))
    }
}
impl Eq for Node {}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({:?})", *self.as_item())
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({:?})", *self.value())
    }
}

/// A DOM based on [`unitree::UNITree`]
///
/// This is thread-safe and you are free to use everywhere:
/// - Don't worry about unsafe codes.
/// - Don't worry about memory leaks.
/// - And don't worry about [`Send`] and [`Sync`] issues.
#[derive(Debug)]
pub struct TreeDom {
    tree: Arc<parking_lot::Mutex<unitree::UNITree<data::NodeData>>>,
    errors: Vec<std::borrow::Cow<'static, str>>,
    quirks_mode: markup5ever::interface::QuirksMode,
    namespaces: parking_lot::Mutex<HashMap<markup5ever::Prefix, markup5ever::Namespace>>,
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
    /// Creates a new [`TreeDom`].
    ///
    /// Use [`TreeDom::default`] if you don't want to specify this parameters.
    pub fn new(
        tree: unitree::UNITree<data::NodeData>,
        errors: Vec<std::borrow::Cow<'static, str>>,
        quirks_mode: markup5ever::interface::QuirksMode,
        namespaces: HashMap<markup5ever::Prefix, markup5ever::Namespace>,
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
    ) -> parking_lot::MappedMutexGuard<'_, HashMap<markup5ever::Prefix, markup5ever::Namespace>>
    {
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
    pub fn get_by_index(&self, index: unitree::Index) -> Option<Node> {
        Node::new(&self.tree, index)
    }

    /// Creates an orphan item.
    ///
    /// In simple terms, Returns an item that has no parent and children;
    /// In other words, an item which is only inserted to the UNITree-internal vector.
    pub fn orphan<D: Into<data::NodeData>>(&self, value: D) -> Node {
        let mut tree = self.tree.lock();
        let index = tree.orphan(value.into());

        std::mem::drop(tree);
        unsafe { Node::new_unchecked(self.tree.clone(), index) }
    }

    /// Appends a child to parent (push_back). It's not important child is an orphan item or not.
    pub fn append(&self, parent: &Node, child: &Node) {
        assert!(
            Arc::ptr_eq(&self.tree, &parent.tree),
            "parent tree is different, cannot append"
        );
        assert!(
            Arc::ptr_eq(&self.tree, &child.tree),
            "child tree is different, cannot append"
        );

        {
            let mut tree = self.tree.lock();
            tree.append(parent.index, child.index);
        }

        let val = child.value();
        if let Some(element) = val.element() {
            if let Some(ref prefix) = element.name.prefix {
                let mut ns = self.namespaces.lock();
                ns.insert(prefix.clone(), element.name.ns.clone());
            }
        }
    }

    /// Prepends a child to parent (push_front). It's not important child is an orphan item or not.
    pub fn prepend(&self, parent: &Node, child: &Node) {
        assert!(
            Arc::ptr_eq(&self.tree, &parent.tree),
            "parent tree is different, cannot prepend"
        );
        assert!(
            Arc::ptr_eq(&self.tree, &child.tree),
            "child tree is different, cannot prepend"
        );

        {
            let mut tree = self.tree.lock();
            tree.prepend(parent.index, child.index);
        }

        let val = child.value();
        if let Some(element) = val.element() {
            if let Some(ref prefix) = element.name.prefix {
                let mut ns = self.namespaces.lock();
                ns.insert(prefix.clone(), element.name.ns.clone());
            }
        }
    }

    /// Sets the item `node` as next sibling of the `sibling`
    pub fn insert_after(&self, sibling: &Node, node: &Node) {
        assert!(
            Arc::ptr_eq(&self.tree, &sibling.tree),
            "sibling tree is different, cannot insert_after"
        );
        assert!(
            Arc::ptr_eq(&self.tree, &node.tree),
            "child tree is different, cannot insert_after"
        );

        {
            let mut tree = self.tree.lock();
            tree.insert_after(sibling.index, node.index);
        }

        let val = node.value();
        if let Some(element) = val.element() {
            if let Some(ref prefix) = element.name.prefix {
                let mut ns = self.namespaces.lock();
                ns.insert(prefix.clone(), element.name.ns.clone());
            }
        }
    }

    /// Sets the item `node` as previous sibling of the `sibling`
    pub fn insert_before(&self, sibling: &Node, node: &Node) {
        assert!(
            Arc::ptr_eq(&self.tree, &sibling.tree),
            "sibling tree is different, cannot insert_before"
        );
        assert!(
            Arc::ptr_eq(&self.tree, &node.tree),
            "child tree is different, cannot insert_before"
        );

        {
            let mut tree = self.tree.lock();
            tree.insert_before(sibling.index, node.index);
        }

        let val = node.value();
        if let Some(element) = val.element() {
            if let Some(ref prefix) = element.name.prefix {
                let mut ns = self.namespaces.lock();
                ns.insert(prefix.clone(), element.name.ns.clone());
            }
        }
    }

    /// Detaches the `node`. In other words, makes it orphan item.
    pub fn detach(&self, node: &Node) {
        assert!(
            Arc::ptr_eq(&self.tree, &node.tree),
            "node tree is different, cannot detach"
        );

        {
            let mut tree = self.tree.lock();
            tree.detach(node.index);
        }

        let val = node.value();
        if let Some(element) = val.element() {
            if let Some(ref prefix) = element.name.prefix {
                let mut ns = self.namespaces.lock();
                ns.remove(prefix);
            }
        }
    }

    /// Remove all the children from `node` and append them to `new_parent`.
    pub fn reparent_append(&self, new_parent: &Node, node: &Node) {
        assert!(
            Arc::ptr_eq(&self.tree, &new_parent.tree),
            "new_parent tree is different, cannot reparent_append"
        );
        assert!(
            Arc::ptr_eq(&self.tree, &node.tree),
            "node tree is different, cannot reparent_append"
        );

        let mut tree = self.tree.lock();
        tree.reparent_append(new_parent.index, node.index);
    }

    /// Remove all the children from `node` and prepend them to `new_parent`.
    pub fn reparent_prepend(&self, new_parent: &Node, node: &Node) {
        assert!(
            Arc::ptr_eq(&self.tree, &new_parent.tree),
            "new_parent tree is different, cannot reparent_prepend"
        );
        assert!(
            Arc::ptr_eq(&self.tree, &node.tree),
            "node tree is different, cannot reparent_prepend"
        );

        let mut tree = self.tree.lock();
        tree.reparent_prepend(new_parent.index, node.index);
    }
}

impl Default for TreeDom {
    fn default() -> Self {
        Self::new(
            unitree::UNITree::new(data::NodeData::new(data::Document)),
            vec![],
            markup5ever::interface::QuirksMode::NoQuirks,
            HashMap::new(),
            0,
        )
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

    #[test]
    fn node() {
        let tree = TreeDom::default();
        let root = tree.root();

        assert_eq!(root.index(), unitree::Index::default());
        assert_eq!(root.first_children(), None);
        assert_eq!(root.last_children(), None);
        assert_eq!(root.parent(), None);
        assert_eq!(root.next_sibling(), None);
        assert_eq!(root.prev_sibling(), None);
        assert_ne!(root.value().document(), None);

        assert_eq!(root, tree.root());

        let tree2 = TreeDom::default();
        assert_ne!(root, tree2.root());
    }

    #[test]
    fn get_by_index() {
        let tree = TreeDom::default();
        let text = tree.orphan(data::Text::new("html5".into()));

        assert_eq!(
            tree.root(),
            tree.get_by_index(unitree::Index::default()).unwrap()
        );
        assert_eq!(text, tree.get_by_index(text.index()).unwrap());
    }

    #[test]
    fn namespaces() {
        let tree = TreeDom::default();
        let element = data::Element::new(
            markup5ever::QualName::new(Some("ns1".into()), "namespace1".into(), "p".into()),
            [].into_iter(),
            false,
            false,
        );

        assert_eq!(tree.namespaces.lock().len(), 0);

        let node1 = tree.orphan(element);
        tree.append(&tree.root(), &node1);
        assert_eq!(tree.namespaces.lock().len(), 1);

        let element = data::Element::new(
            markup5ever::QualName::new(None, "".into(), "p".into()),
            [].into_iter(),
            false,
            false,
        );

        let node2 = tree.orphan(element);
        tree.append(&tree.root(), &node2);
        assert_eq!(tree.namespaces.lock().len(), 1);

        tree.detach(&node1);
        assert_eq!(tree.namespaces.lock().len(), 0);
    }
}
