//! (Un)safe (I)d Tree - An ID-tree [`Vec`]-backed that uses [`core::ptr::NonNull`] to avoid lifetimes.
//!
//! Be very careful while using this crate. we thought that you're master in Rust, and know how to
//! use threads and tasks. we use [`core::ptr::NonNull`] rather than lifetimes; By this way you can do everything you want.

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

    pub fn parent(&self) -> Option<Index> {
        self.parent
    }

    pub fn prev_sibling(&self) -> Option<Index> {
        self.prev_sibling
    }

    pub fn next_sibling(&self) -> Option<Index> {
        self.next_sibling
    }

    pub fn children(&self) -> Option<(Index, Index)> {
        self.children
    }

    pub fn first_children(&self) -> Option<Index> {
        self.children.map(|(i, _)| i)
    }

    pub fn last_children(&self) -> Option<Index> {
        self.children.map(|(_, i)| i)
    }

    pub fn value(&self) -> &T {
        &self.value
    }

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
    #[inline]
    pub fn new(root: T) -> Self {
        let mut vec = alloc::vec::Vec::new();

        unsafe {
            vec.push(Item::into_nonnull(Item::new(root)));
        }

        Self { vec }
    }

    /// Returns a pointer to an item as position `index`.
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
    pub fn orphan(&mut self, value: T) -> (Index, NonNull<Item<T>>) {
        let index = unsafe { Index::new(self.vec.len()) };

        unsafe {
            let ptr = Item::into_nonnull(Item::new(value));
            self.vec.push(ptr);

            (index, ptr)
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
}

impl<T> Drop for UNITree<T> {
    fn drop(&mut self) {
        // drop the pointers ...
        for i in self.vec.drain(..) {
            let _ = unsafe { alloc::boxed::Box::from_raw(i.as_ptr()) };
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
