//! (Un)safe (I)d Tree - An ID-tree [`Vec`]-backed that uses [`std::ptr::NonNull`] to avoid lifetimes.
//!
//! Be very careful while using this crate. we thought that you're master in Rust, and know how to
//! use threads and tasks. we use [`std::ptr::NonNull`] rather than lifetimes; By this way you can do everything you want.

use std::ptr::NonNull;

/// Index of items in [`UNITree`]-internal vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(std::num::NonZeroUsize);

impl Index {
    /// Creates a new [`Index`]
    ///
    /// # Safety
    /// The value must not be [`usize::MAX`].
    unsafe fn new(n: usize) -> Self {
        Self(std::num::NonZeroUsize::new_unchecked(n + 1))
    }

    pub fn into_nonzero(self) -> std::num::NonZeroUsize {
        self.0
    }

    pub fn into_usize(self) -> usize {
        self.0.get()
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
        let ptr = Box::into_raw(Box::new(x));
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

impl<T: std::hash::Hash> std::hash::Hash for Item<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.value, state);
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

impl<T: std::fmt::Debug> std::fmt::Debug for Item<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    vec: Vec<NonNull<Item<T>>>,
}

impl<T> UNITree<T> {
    #[inline]
    pub fn new(root: T) -> Self {
        unsafe {
            Self {
                vec: vec![Item::into_nonnull(Item::new(root))],
            }
        }
    }

    pub fn with_capacity(root: T, capacity: usize) -> Self {
        let mut vec = Vec::with_capacity(capacity);
        unsafe {
            vec.push(Item::into_nonnull(Item::new(root)));
        }
        UNITree { vec }
    }

    /// Returns a pointer to an item, without doing bounds checking.
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
}

impl<T> Drop for UNITree<T> {
    fn drop(&mut self) {
        // drop the pointers ...
        for i in self.vec.drain(..) {
            let _ = unsafe { Box::from_raw(i.as_ptr()) };
        }
    }
}
