use super::node::Node;
use super::node::SizedSmallVec;
use super::node::WeakNode;

pub struct ChildrenMutexGuard<'a> {
    root: &'a Node,
    item: parking_lot::MappedMutexGuard<'a, SizedSmallVec<Node>>,
}

impl<'a> std::ops::Deref for ChildrenMutexGuard<'a> {
    type Target = SizedSmallVec<Node>;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> ChildrenMutexGuard<'a> {
    /// Creates a new `ChildrenMutexGuard`
    pub fn new(root: &'a Node) -> Self {
        #[cfg(debug_assertions)]
        let ref_ = root.inner.children.try_lock().expect("children is locked");

        #[cfg(not(debug_assertions))]
        let ref_ = root.inner.children.lock();

        Self {
            root,
            item: parking_lot::MutexGuard::map(ref_, |x| x),
        }
    }

    /// Append a new child into node and sets its new parent
    ///
    /// Returns error if the child has parent for itself.
    /// Also returns error if child cycle be detected.
    pub fn push(&mut self, val: Node) -> Result<(), super::errors::ErrorKind> {
        if self.root.ptr_eq(&val) {
            return Err(super::errors::ErrorKind::ChildCycleDetected);
        }

        let mut parent = val.parent();
        if parent.is_some() {
            return Err(super::errors::ErrorKind::NodeHasParent);
        }

        parent.replace(self.root.downgrade());
        std::mem::drop(parent);

        self.item.push(val);
        Ok(())
    }

    /// Pop a child from node and removes its parent
    pub fn pop(&mut self) -> Option<Node> {
        let node = self.item.pop()?;
        node.parent().take().unwrap();
        Some(node)
    }

    /// Clears children and removes their parent
    pub fn clear(&mut self) {
        for child in self.item.drain(..) {
            child.parent().take().unwrap();
        }
    }

    /// Remove and return the child at position `index`. Also removes its parent.
    ///
    /// Panics if index is out of bounds.
    pub fn remove(&mut self, index: usize) -> Node {
        let node = self.item.remove(index);
        node.parent().take().unwrap();
        node
    }

    /// Inserts a child at position `index`.
    ///
    /// Returns error if the child has parent for itself.
    /// Also returns error if child cycle be detected.
    ///
    /// Panics if `index > len`.
    pub fn insert(&mut self, index: usize, val: Node) -> Result<(), super::errors::ErrorKind> {
        assert!(index <= self.item.len());

        if self.root.ptr_eq(&val) {
            return Err(super::errors::ErrorKind::ChildCycleDetected);
        }

        let mut parent = val.parent();
        if parent.is_some() {
            return Err(super::errors::ErrorKind::NodeHasParent);
        }

        parent.replace(self.root.downgrade());
        std::mem::drop(parent);

        self.item.insert(index, val);
        Ok(())
    }

    /// Creates a draining iterator that removes the specified range in the vector and yields the removed children.
    ///
    /// # Safety
    /// be careful because while draining we do not remove their parent and this can cause errors.
    /// **You have to remove their parent yourself.**
    pub unsafe fn drain<R: std::ops::RangeBounds<usize>>(
        &mut self,
        range: R,
    ) -> smallvec::Drain<'_, [Node; 4]> {
        self.item.drain(range)
    }
}

pub struct TreeIterator {
    order: Vec<Node>,
}

impl TreeIterator {
    /// Creates a new `TreeIterator` that includes root node.
    pub fn new_with_node(root: Node) -> Self {
        Self { order: vec![root] }
    }

    /// Creates a new `TreeIterator` from a node children.
    pub fn new(children: ChildrenMutexGuard) -> Self {
        let mut order = Vec::with_capacity(children.len());

        order.extend(children.iter().cloned().rev());
        Self { order }
    }
}

impl Iterator for TreeIterator {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.order.pop()?;

        self.order.extend(node.children().iter().cloned().rev());

        Some(node)
    }
}

pub struct ParentsIterator {
    last: Option<Node>,
}

impl ParentsIterator {
    /// Creates a new `ParentsIterator` that includes root node.
    pub fn new_with_node(root: Node) -> Self {
        Self { last: Some(root) }
    }

    /// Creates a new `ParentsIterator` from a node parent.
    pub fn new(parent: Option<WeakNode>) -> Self {
        Self {
            last: parent.map(|x| x.upgrade().expect("dangling weak reference")),
        }
    }
}

impl Iterator for ParentsIterator {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.last.take()?;

        self.last = node
            .parent()
            .clone()
            .map(|x| x.upgrade())
            .unwrap_or_default();

        Some(node)
    }
}
