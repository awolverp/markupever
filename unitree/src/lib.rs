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
    pub unsafe fn new(n: usize) -> Self {
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
        self.vec.get_unchecked(index.into_usize()).clone()
    }

    /// Returns a pointer to the root item.
    #[inline]
    pub fn root(&self) -> NonNull<Item<T>> {
        unsafe {
            self.vec
                .get_unchecked(Index::default().into_usize())
                .clone()
        }
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
}

impl<T> Drop for UNITree<T> {
    fn drop(&mut self) {
        // drop the pointers ...
        for i in self.vec.drain(..) {
            let _ = unsafe { Box::from_raw(i.as_ptr()) };
        }
    }
}
