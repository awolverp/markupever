//! (Un)safe (I)d Tree - An ID-tree [`Vec`]-backed that uses [`core::ptr::NonNull`] to avoid lifetimes.
//!
//! Be very careful while using this crate. we thought that you're master in Rust, and know how to
//! use threads and tasks. we use [`core::ptr::NonNull`] rather than lifetimes; By this way you can do everything you want.

#![allow(clippy::len_without_is_empty)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

use core::ptr::NonNull;

/// Index of items in [`UNITree`]-internal vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(core::num::NonZeroUsize);

impl Index {
    /// Creates a new [`Index`]
    ///
    /// # Safety
    /// The value must not be [`usize::MAX`].
    unsafe fn new(n: usize) -> Self {
        Self(core::num::NonZeroUsize::new_unchecked(n + 1))
    }

    /// Consumes self and returns usize to use it as index
    pub fn into_usize(self) -> usize {
        self.0.get() - 1
    }
}

impl Default for Index {
    fn default() -> Self {
        unsafe { Self::new(0) }
    }
}

pub struct Item<T> {
    /// Parent item index
    parent: Option<Index>,
    /// Previous sibling item index
    prev_sibling: Option<Index>,
    /// Next sibling item index
    next_sibling: Option<Index>,
    /// Children (first, last) items' indexes
    children: Option<(Index, Index)>,
    /// The value that item holds
    value: T,
}

impl<T> Item<T> {
    /// Creates a new [`Item`]
    fn new(value: T) -> Self {
        Self {
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            children: None,
            value,
        }
    }

    /// Consumes the [`Item`], returning a [`NonNull`] that wrapped it.
    ///
    /// # Safety
    /// Don't forgot to deallocate the pointer using [`Box::from_raw`].
    unsafe fn into_nonnull(x: Self) -> NonNull<Self> {
        let ptr = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(x));
        NonNull::new_unchecked(ptr)
    }

    /// Calls the destructor of [`NonNull<Self>`] and free the allocated memory.
    ///
    /// # Safety
    /// This function is unsafe because improper use may lead to memory problems.
    /// For example, a double-free may occur if the function is called twice on the same raw pointer.
    unsafe fn drop_nonnull(x: NonNull<Self>) {
        core::mem::drop(alloc::boxed::Box::from_raw(x.as_ptr()));
    }

    /// Returns the parent index
    pub fn parent(&self) -> Option<Index> {
        self.parent
    }

    /// Returns the previous sibling index
    pub fn prev_sibling(&self) -> Option<Index> {
        self.prev_sibling
    }

    /// Returns the next sibling index
    pub fn next_sibling(&self) -> Option<Index> {
        self.next_sibling
    }

    /// Returns the children indexes (first, last)
    pub fn children(&self) -> Option<(Index, Index)> {
        self.children
    }

    /// Returns the first children index
    pub fn first_children(&self) -> Option<Index> {
        self.children.map(|(i, _)| i)
    }

    /// Returns the last children index
    pub fn last_children(&self) -> Option<Index> {
        self.children.map(|(_, i)| i)
    }

    /// Returns a immutable reference to the item value
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the item value
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: PartialEq> PartialEq for Item<T> {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent
            && self.prev_sibling == other.prev_sibling
            && self.next_sibling == other.next_sibling
            && self.children == other.children
            && self.value.eq(&other.value)
    }
}
impl<T: Eq> Eq for Item<T> {}

impl<T: core::hash::Hash> core::hash::Hash for Item<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(&self.value, state);
    }
}

impl<T: Clone> Clone for Item<T> {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent,
            prev_sibling: self.prev_sibling,
            next_sibling: self.next_sibling,
            children: self.children,
            value: self.value.clone(),
        }
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for Item<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Item")
            .field("parent", &self.parent)
            .field("prev_sibling", &self.prev_sibling)
            .field("next_sibling", &self.next_sibling)
            .field("children", &self.children)
            .field("value", &self.value)
            .finish()
    }
}

pub struct UNITree<T> {
    vec: alloc::vec::Vec<NonNull<Item<T>>>,
}

impl<T> UNITree<T> {
    /// Creates a new [`UNITree`] with `root` as root item.
    #[inline]
    pub fn new(root: T) -> Self {
        let mut vec = alloc::vec::Vec::new();

        unsafe {
            vec.push(Item::into_nonnull(Item::new(root)));
        }

        Self { vec }
    }

    /// Returns the number of items in the tree, also referred to as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns a pointer to an item as position `index`.
    ///
    /// Returns [`None`] if the `index` is out of bounds
    #[inline]
    pub fn get(&self, index: Index) -> Option<NonNull<Item<T>>> {
        self.vec.get(index.into_usize()).cloned()
    }

    /// Returns a pointer to an item, without doing bounds checking.
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: Index) -> NonNull<Item<T>> {
        *self.vec.get_unchecked(index.into_usize())
    }

    /// Returns a pointer to the root item.
    #[inline]
    pub fn root(&self) -> NonNull<Item<T>> {
        unsafe { *self.vec.get_unchecked(Index::default().into_usize()) }
    }

    /// Shorthand for [`Index::default`] which always returns the root index.
    #[inline(always)]
    pub fn root_index(&self) -> Index {
        Index::default()
    }

    /// Creates an orphan item.
    ///
    /// In simple terms, Returns an item that has no parent and children;
    /// In other words, an item which is only inserted to the [`UNITree`]-internal vector.
    pub fn orphan(&mut self, value: T) -> Index {
        let index = unsafe { Index::new(self.vec.len()) };

        unsafe {
            self.vec.push(Item::into_nonnull(Item::new(value)));
            index
        }
    }

    fn detach_item(&mut self, index: Index, item: &mut Item<T>) {
        let parent_index = match item.parent {
            Some(id) => id,
            None => return,
        };

        let (prev_index, next_index) = (item.prev_sibling, item.next_sibling);

        item.parent = None;
        item.prev_sibling = None;
        item.next_sibling = None;

        if let Some(id) = prev_index {
            unsafe {
                self.vec[id.into_usize()].as_mut().next_sibling = next_index;
            }
        }

        if let Some(id) = next_index {
            unsafe {
                self.vec[id.into_usize()].as_mut().prev_sibling = prev_index;
            }
        }

        let parent = unsafe { self.vec[parent_index.into_usize()].as_mut() };
        let (first_index, last_index) = parent.children.unwrap();

        if first_index == last_index {
            parent.children = None;
        } else if first_index == index {
            parent.children = Some((next_index.unwrap(), last_index));
        } else if last_index == index {
            parent.children = Some((first_index, prev_index.unwrap()));
        }
    }

    /// Detaches the item as position `index`. In other words, makes it orphan item.
    pub fn detach(&mut self, index: Index) {
        let mut item = self.get(index).unwrap();
        self.detach_item(index, unsafe { item.as_mut() });
    }

    /// Appends a child to parent (push_back). It's not important child is an orphan item or not.
    ///
    /// # Panics
    /// Panics if `parent_index` == `child_index`
    pub fn append(&mut self, parent_index: Index, child_index: Index) {
        assert_ne!(
            parent_index, child_index,
            "Cannot append node as a child to itself"
        );

        let mut parent = self.get(parent_index).unwrap();
        let last_child_index = unsafe { parent.as_ref().last_children() };

        if last_child_index != Some(child_index) {
            unsafe {
                let mut new_child = self.get(child_index).unwrap();
                self.detach_item(child_index, new_child.as_mut());
                new_child.as_mut().parent = Some(parent_index);
                new_child.as_mut().prev_sibling = last_child_index;
            }

            if let Some(index) = last_child_index {
                unsafe {
                    self.get(index).unwrap().as_mut().next_sibling = Some(child_index);
                }
            }

            unsafe {
                let children = match parent.as_ref().children {
                    Some((f, _)) => Some((f, child_index)),
                    None => Some((child_index, child_index)),
                };
                parent.as_mut().children = children;
            }
        }
    }

    /// Prepends a child to parent (push_front). It's not important child is an orphan item or not.
    ///
    /// # Panics
    /// Panics if `parent_index` == `child_index`
    pub fn prepend(&mut self, parent_index: Index, child_index: Index) {
        assert_ne!(
            parent_index, child_index,
            "Cannot prepend node as a child to itself"
        );

        let mut parent = self.get(parent_index).unwrap();
        let first_child_index = unsafe { parent.as_ref().first_children() };

        if first_child_index != Some(child_index) {
            unsafe {
                let mut new_child = self.get(child_index).unwrap();
                self.detach_item(child_index, new_child.as_mut());
                new_child.as_mut().parent = Some(parent_index);
                new_child.as_mut().next_sibling = first_child_index;
            }

            if let Some(index) = first_child_index {
                unsafe {
                    self.get(index).unwrap().as_mut().prev_sibling = Some(child_index);
                }
            }

            unsafe {
                let children = match parent.as_ref().children {
                    Some((_, l)) => Some((child_index, l)),
                    None => Some((child_index, child_index)),
                };
                parent.as_mut().children = children;
            }
        }
    }

    /// Sets the item at position `x` as previous sibling of the item at position `index`.
    ///
    /// # Panics
    /// - Panics if `x` is not valid.
    /// - Panics if `index` is an orphan.
    /// - Panics if `index` == `x`
    pub fn insert_before(&mut self, index: Index, x: Index) {
        assert_ne!(index, x, "Cannot insert an item as a sibling of itself");

        let mut node = self.get(index).unwrap();
        let parent_index = unsafe { node.as_ref().parent.unwrap() };
        let prev_index = unsafe { node.as_ref().prev_sibling };

        unsafe {
            let mut new_sibling = self.get(x).unwrap();
            self.detach_item(x, new_sibling.as_mut());
            new_sibling.as_mut().parent = Some(parent_index);
            new_sibling.as_mut().prev_sibling = prev_index;
            new_sibling.as_mut().next_sibling = Some(index);
        }

        if let Some(i) = prev_index {
            unsafe {
                self.get(i).unwrap().as_mut().next_sibling = Some(x);
            }
        }

        unsafe {
            node.as_mut().prev_sibling = Some(x);

            let mut parent = self.get(parent_index).unwrap();
            let (f_index, l_index) = parent.as_ref().children.unwrap();
            if f_index == index {
                parent.as_mut().children = Some((x, l_index));
            }
        }
    }

    /// Sets the item at position `x` as next sibling of the item at position `index`.
    ///
    /// # Panics
    /// - Panics if `x` is not valid.
    /// - Panics if `index` is an orphan.
    /// - Panics if `index` == `x`
    pub fn insert_after(&mut self, index: Index, x: Index) {
        assert_ne!(index, x, "Cannot insert an item as a sibling of itself");

        let mut node = self.get(index).unwrap();
        let parent_index = unsafe { node.as_ref().parent.unwrap() };
        let next_index = unsafe { node.as_ref().next_sibling };

        unsafe {
            let mut new_sibling = self.get(x).unwrap();
            self.detach_item(x, new_sibling.as_mut());
            new_sibling.as_mut().parent = Some(parent_index);
            new_sibling.as_mut().prev_sibling = Some(index);
            new_sibling.as_mut().next_sibling = next_index;
        }

        if let Some(i) = next_index {
            unsafe {
                self.get(i).unwrap().as_mut().prev_sibling = Some(x);
            }
        }

        unsafe {
            node.as_mut().next_sibling = Some(x);

            let mut parent = self.get(parent_index).unwrap();
            let (f_index, l_index) = parent.as_ref().children.unwrap();
            if l_index == index {
                parent.as_mut().children = Some((f_index, x));
            }
        }
    }

    /// Remove all the children from `index` and append them to `new_parent`.
    ///
    /// # Panics
    /// Panics if `new_parent` == `index`
    pub fn reparent_append(&mut self, new_parent: Index, index: Index) {
        assert_ne!(
            new_parent, index,
            "Cannot reparent node's children to itself"
        );

        let child_ids = {
            let item = self.get(index).unwrap();
            match unsafe { (*item.as_ptr()).children.take() } {
                Some(ids) => ids,
                None => return,
            }
        };

        unsafe {
            self.get(child_ids.0).unwrap().as_mut().parent = Some(new_parent);
            self.get(child_ids.1).unwrap().as_mut().parent = Some(new_parent);

            let mut parent = self.get(new_parent).unwrap();

            if parent.as_ref().children.is_none() {
                parent.as_mut().children = Some(child_ids);
                return;
            }

            let old_child_ids = parent.as_ref().children.unwrap();

            self.get(old_child_ids.1).unwrap().as_mut().next_sibling = Some(child_ids.0);
            self.get(child_ids.0).unwrap().as_mut().prev_sibling = Some(old_child_ids.1);

            parent.as_mut().children = Some((old_child_ids.0, child_ids.1));
        }
    }

    /// Remove all the children from `index` and prepend them to `new_parent`.
    ///
    /// # Panics
    /// Panics if `new_parent` == `index`
    pub fn reparent_prepend(&mut self, new_parent: Index, index: Index) {
        assert_ne!(
            new_parent, index,
            "Cannot reparent node's children to itself"
        );

        let child_ids = {
            let item = self.get(index).unwrap();
            match unsafe { (*item.as_ptr()).children.take() } {
                Some(ids) => ids,
                None => return,
            }
        };

        unsafe {
            self.get(child_ids.0).unwrap().as_mut().parent = Some(new_parent);
            self.get(child_ids.1).unwrap().as_mut().parent = Some(new_parent);

            let mut parent = self.get(new_parent).unwrap();

            if parent.as_ref().children.is_none() {
                parent.as_mut().children = Some(child_ids);
                return;
            }

            let old_child_ids = parent.as_ref().children.unwrap();

            self.get(old_child_ids.1).unwrap().as_mut().prev_sibling = Some(child_ids.1);
            self.get(child_ids.0).unwrap().as_mut().next_sibling = Some(old_child_ids.0);

            parent.as_mut().children = Some((child_ids.0, old_child_ids.1));
        }
    }
}

impl<T> Drop for UNITree<T> {
    fn drop(&mut self) {
        // drop the pointers ...
        for i in self.vec.drain(..) {
            unsafe { Item::drop_nonnull(i) };
        }
    }
}

pub mod iter {
    use crate::{Index, Item, UNITree};
    use core::ptr::NonNull;

    #[derive(PartialEq, Eq)]
    pub enum Edge<T> {
        Open(NonNull<Item<T>>),
        Close(NonNull<Item<T>>),
    }

    impl<T> Copy for Edge<T> {}
    impl<T> Clone for Edge<T> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<T: core::fmt::Debug> core::fmt::Debug for Edge<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Edge::Open(x) => {
                    write!(f, "Edge::Open({:?})", unsafe { x.as_ref() })?;
                }
                Edge::Close(x) => {
                    write!(f, "Edge::Close({:?})", unsafe { x.as_ref() })?;
                }
            }

            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct Traverse<'a, T> {
        tree: &'a UNITree<T>,
        root: Option<NonNull<Item<T>>>,
        edge: Option<Edge<T>>,
    }

    impl<'a, T> Traverse<'a, T> {
        /// Creates a new [`Traverse`] for a [`UNITree`]
        pub fn new(tree: &'a UNITree<T>, index: Index) -> Self {
            Self {
                tree,
                root: Some(tree.get(index).unwrap()),
                edge: None,
            }
        }
    }

    impl<'a, T> Iterator for Traverse<'a, T> {
        type Item = Edge<T>;

        fn next(&mut self) -> Option<Self::Item> {
            match self.edge {
                None => {
                    if let Some(root) = self.root {
                        self.edge = Some(Edge::Open(root));
                    }
                }
                Some(Edge::Open(node)) => {
                    if let Some(first_child) = unsafe { node.as_ref().first_children() } {
                        self.edge = Some(Edge::Open(self.tree.get(first_child).unwrap()));
                    } else {
                        self.edge = Some(Edge::Close(node));
                    }
                }
                Some(Edge::Close(node)) => {
                    if node == self.root.unwrap() {
                        self.root = None;
                        self.edge = None;
                    } else if let Some(next_sibling) = unsafe { node.as_ref().next_sibling() } {
                        self.edge = Some(Edge::Open(self.tree.get(next_sibling).unwrap()));
                    } else {
                        self.edge = unsafe {
                            node.as_ref()
                                .parent()
                                .map(|x| Edge::Close(self.tree.get(x).unwrap()))
                        };
                    }
                }
            }

            self.edge
        }
    }
}

struct DisplayChar {
    siblings: bool,
    children: bool,
}

impl DisplayChar {
    fn char(&self) -> &str {
        match (self.siblings, self.children) {
            (true, true) => "│   ",
            (true, false) => "├── ",
            (false, true) => "    ",
            (false, false) => "└── ",
        }
    }
}

struct Indentation {
    tokens: alloc::vec::Vec<DisplayChar>,
    ignore_root: bool,
}

impl core::fmt::Display for Indentation {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let first: usize = if self.ignore_root { 1 } else { 0 };

        for token in &self.tokens[first..] {
            write!(f, "{}", token.char())?;
        }

        Ok(())
    }
}

impl Indentation {
    fn new(ignore_root: bool) -> Self {
        Indentation {
            tokens: alloc::vec::Vec::new(),
            ignore_root,
        }
    }

    fn indent(&mut self, siblings: bool) -> &mut Self {
        let len = self.tokens.len();
        if len > 0 {
            self.tokens[len - 1].children = true;
        }

        self.tokens.push(DisplayChar {
            siblings,
            children: false,
        });
        self
    }

    fn deindent(&mut self) -> &mut Self {
        self.tokens.pop();
        self
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for UNITree<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        if f.alternate() {
            write!(f, "UNITree [")?;
            for edge in iter::Traverse::new(self, Index::default()) {
                match edge {
                    iter::Edge::Open(item) if unsafe { item.as_ref().children.is_some() } => {
                        write!(f, " {:?} => [[", unsafe { &item.as_ref().value })?;
                    }
                    iter::Edge::Open(item) if unsafe { item.as_ref().next_sibling.is_some() } => {
                        write!(f, " {:?},", unsafe { &item.as_ref().value })?;
                    }
                    iter::Edge::Open(item) => {
                        write!(f, " {:?}", unsafe { &item.as_ref().value })?;
                    }
                    iter::Edge::Close(item) if unsafe { item.as_ref().children.is_some() } => {
                        if unsafe { item.as_ref().next_sibling.is_some() } {
                            write!(f, " ]],")?;
                        } else {
                            write!(f, " ]]")?;
                        }
                    }
                    _ => {}
                }
            }
            write!(f, " ]")
        } else {
            f.debug_struct("Tree").field("vec", &self.vec).finish()
        }
    }
}

impl<T: core::fmt::Display> core::fmt::Display for UNITree<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        let mut indent: Indentation = Indentation::new(true);

        for edge in iter::Traverse::new(self, Index::default()) {
            match edge {
                iter::Edge::Open(item) if unsafe { item.as_ref().children.is_some() } => {
                    indent.indent(unsafe { item.as_ref().next_sibling.is_some() });
                    writeln!(f, "{indent}{}", unsafe { &item.as_ref().value })?;
                }
                iter::Edge::Open(item) => {
                    indent.indent(unsafe { item.as_ref().next_sibling.is_some() });
                    writeln!(f, "{indent}{}", unsafe { &item.as_ref().value })?;
                    indent.deindent();
                }
                iter::Edge::Close(item) if unsafe { item.as_ref().children.is_some() } => {
                    indent.deindent();
                }
                _ => {}
            }
        }
        Ok(())
    }
}

unsafe impl<T: Send> Send for UNITree<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index() {
        let i = unsafe { Index::new(0) };
        assert_eq!(i, Index::default());
        assert_eq!(i.into_usize(), 0);
    }

    // used for test_item
    static mut IS_DROPPED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

    #[test]
    fn test_item() {
        let item = Item::new("test");

        assert_eq!(*item.value(), "test");
        assert_ne!(item, Item::new("test2"));
        assert_eq!(item, Item::new("test"));

        struct CheckDrop;

        impl Drop for CheckDrop {
            fn drop(&mut self) {
                unsafe {
                    IS_DROPPED.store(true, core::sync::atomic::Ordering::Relaxed);
                }
            }
        }

        let item = unsafe { Item::into_nonnull(
            Item::new(CheckDrop)
        ) };

        unsafe {
            assert_eq!(IS_DROPPED.load(core::sync::atomic::Ordering::Relaxed), false);
            Item::drop_nonnull(item);
            assert_eq!(IS_DROPPED.load(core::sync::atomic::Ordering::Relaxed), true);
        }
    }

    #[test]
    fn new() {
        let tree = UNITree::new('a');
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.root_index(), Index::default());

        let root = tree.root();
        unsafe {
            assert_eq!(root.as_ref().parent(), None);
            assert_eq!(root.as_ref().prev_sibling(), None);
            assert_eq!(root.as_ref().next_sibling(), None);
            assert_eq!(root.as_ref().first_children(), None);
            assert_eq!(root.as_ref().last_children(), None);
            assert_eq!(root.as_ref().children(), None);
            assert_eq!(*root.as_ref().value(), 'a');
        }
    }

    #[test]
    fn orphan() {
        let mut tree = UNITree::new('a');
        let index = tree.orphan('b');

        let ptr = tree.get(index).unwrap();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'b');
            assert_eq!(ptr.as_ref().parent, None);
        }
    }

    #[test]
    fn append() {
        let mut tree = UNITree::new('a');
        let first_child_index = tree.orphan('b');
        let ptr = tree.get(first_child_index).unwrap();
        
        tree.append(tree.root_index(), first_child_index);

        unsafe {
            assert_eq!(ptr.as_ref().value, 'b');
            assert_eq!(ptr.as_ref().parent, Some(tree.root_index()));
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, None);
        }

        let ptr = tree.root();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'a');
            assert_eq!(ptr.as_ref().parent, None);
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, Some((first_child_index, first_child_index)));
        }

        let last_child_index = tree.orphan('c');
        let ptr = tree.get(last_child_index).unwrap();

        tree.append(tree.root_index(), last_child_index);

        unsafe {
            assert_eq!(ptr.as_ref().value, 'c');
            assert_eq!(ptr.as_ref().parent, Some(tree.root_index()));
            assert_eq!(ptr.as_ref().prev_sibling, Some(first_child_index));
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, None);
        }

        let ptr = tree.root();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'a');
            assert_eq!(ptr.as_ref().parent, None);
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, Some((first_child_index, last_child_index)));
        }

        let ptr = tree.get(first_child_index).unwrap();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'b');
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, Some(last_child_index));
            assert_eq!(ptr.as_ref().children, None);
        }
    }

    #[test]
    fn prepend() {
        let mut tree = UNITree::new('a');
        let child_1 = tree.orphan('b');
        let ptr = tree.get(child_1).unwrap();
        
        tree.prepend(tree.root_index(), child_1);

        unsafe {
            assert_eq!(ptr.as_ref().value, 'b');
            assert_eq!(ptr.as_ref().parent, Some(tree.root_index()));
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, None);
        }

        let ptr = tree.root();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'a');
            assert_eq!(ptr.as_ref().parent, None);
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, Some((child_1, child_1)));
        }

        let child_2 = tree.orphan('c');
        let ptr = tree.get(child_2).unwrap();

        tree.prepend(tree.root_index(), child_2);

        unsafe {
            assert_eq!(ptr.as_ref().value, 'c');
            assert_eq!(ptr.as_ref().parent, Some(tree.root_index()));
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, Some(child_1));
            assert_eq!(ptr.as_ref().children, None);
        }

        let ptr = tree.root();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'a');
            assert_eq!(ptr.as_ref().parent, None);
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, Some((child_2, child_1)));
        }

        let ptr = tree.get(child_1).unwrap();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'b');
            assert_eq!(ptr.as_ref().prev_sibling, Some(child_2));
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, None);
        }
    }

    #[test]
    fn insert_after() {
        let mut tree = UNITree::new('a');
        let first_child_index = tree.orphan('b');
        let ptr = tree.get(first_child_index).unwrap();
        
        tree.append(tree.root_index(), first_child_index);

        unsafe {
            assert_eq!(ptr.as_ref().value, 'b');
            assert_eq!(ptr.as_ref().parent, Some(tree.root_index()));
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, None);
        }

        let ptr = tree.root();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'a');
            assert_eq!(ptr.as_ref().parent, None);
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, Some((first_child_index, first_child_index)));
        }

        let last_child_index = tree.orphan('c');
        let ptr = tree.get(last_child_index).unwrap();

        tree.insert_after(first_child_index, last_child_index);

        unsafe {
            assert_eq!(ptr.as_ref().value, 'c');
            assert_eq!(ptr.as_ref().parent, Some(tree.root_index()));
            assert_eq!(ptr.as_ref().prev_sibling, Some(first_child_index));
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, None);
        }

        let ptr = tree.root();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'a');
            assert_eq!(ptr.as_ref().parent, None);
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, Some((first_child_index, last_child_index)));
        }

        let ptr = tree.get(first_child_index).unwrap();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'b');
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, Some(last_child_index));
            assert_eq!(ptr.as_ref().children, None);
        }
    }

    #[test]
    fn insert_before() {
        let mut tree = UNITree::new('a');
        let child_1 = tree.orphan('b');
        let ptr = tree.get(child_1).unwrap();
        
        tree.append(tree.root_index(), child_1);

        unsafe {
            assert_eq!(ptr.as_ref().value, 'b');
            assert_eq!(ptr.as_ref().parent, Some(tree.root_index()));
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, None);
        }

        let ptr = tree.root();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'a');
            assert_eq!(ptr.as_ref().parent, None);
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, Some((child_1, child_1)));
        }

        let child_2 = tree.orphan('c');
        let ptr = tree.get(child_2).unwrap();

        tree.insert_before(child_1, child_2);

        unsafe {
            assert_eq!(ptr.as_ref().value, 'c');
            assert_eq!(ptr.as_ref().parent, Some(tree.root_index()));
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, Some(child_1));
            assert_eq!(ptr.as_ref().children, None);
        }

        let ptr = tree.root();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'a');
            assert_eq!(ptr.as_ref().parent, None);
            assert_eq!(ptr.as_ref().prev_sibling, None);
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, Some((child_2, child_1)));
        }

        let ptr = tree.get(child_1).unwrap();
        unsafe {
            assert_eq!(ptr.as_ref().value, 'b');
            assert_eq!(ptr.as_ref().prev_sibling, Some(child_2));
            assert_eq!(ptr.as_ref().next_sibling, None);
            assert_eq!(ptr.as_ref().children, None);
        }
    }

    #[test]
    fn reparent_append() {
        let mut tree = UNITree::new('a');
        let item_1 = tree.orphan('b');
        let item_2 = tree.orphan('c');

        tree.append(tree.root_index(), item_1);
        tree.append(tree.root_index(), item_2);

        // append some child to item_1
        let item_1_1 = tree.orphan('d');
        let item_1_2 = tree.orphan('e');

        tree.append(item_1, item_1_1);
        tree.append(item_1, item_1_2);

        let ptr = tree.get(item_1).unwrap();
        unsafe {
            assert_eq!(ptr.as_ref().children, Some((item_1_1, item_1_2)));
        }

        // reparent to item_2
        tree.reparent_append(item_2, item_1);

        unsafe {
            assert_eq!(ptr.as_ref().children, None);
        }

        let ptr = tree.get(item_2).unwrap();
        unsafe {
            assert_eq!(ptr.as_ref().children, Some((item_1_1, item_1_2)));
        }
    }
}
