/// A synchronization primitive which can nominally be written to only once.
///
/// Uses [`parking_lot::Once`] instead of [`std::sync::Once`]:
/// - Lower memory usage.
/// - Not required to be 'static.
/// - Relaxed memory barriers in the fast path, which can significantly improve performance on some architectures.
/// - Efficient handling of micro-contention using adaptive spinning.
pub struct OnceLock<T> {
    once: ::parking_lot::Once,
    value: ::std::cell::UnsafeCell<::std::mem::MaybeUninit<T>>,
    phantom: ::std::marker::PhantomData<T>,
}

impl<T> OnceLock<T> {
    pub const fn new() -> OnceLock<T> {
        OnceLock {
            once: ::parking_lot::Once::new(),
            value: ::std::cell::UnsafeCell::new(::std::mem::MaybeUninit::uninit()),
            phantom: ::std::marker::PhantomData,
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.once.state().done() {
            Some(unsafe { (*self.value.get()).assume_init_ref() })
        } else {
            None
        }
    }

    #[cold]
    fn initialize<F>(&self, f: F)
    where
        F: FnOnce() -> T,
    {
        let slot = &self.value;

        self.once.call_once_force(|_| {
            unsafe { (*slot.get()).write(f()) };
        });
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if let Some(value) = self.get() {
            return value;
        }

        self.initialize(f);

        debug_assert!(self.once.state().done());

        // SAFETY: The inner value has been initialized
        unsafe { (*self.value.get()).assume_init_ref() }
    }

    pub fn take(&mut self) -> Option<T> {
        if self.once.state().done() {
            self.once = ::parking_lot::Once::new();
            // SAFETY: `self.value` is initialized and contains a valid `T`.
            // `self.once` is reset, so `is_initialized()` will be false again
            // which prevents the value from being read twice.
            unsafe { Some((*self.value.get()).assume_init_read()) }
        } else {
            None
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for OnceLock<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get() {
            Some(val) => write!(f, "OnceLock<{:?}>", val),
            None => write!(f, "OnceLock<uninialized>"),
        }
    }
}

impl<T> Clone for OnceLock<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<T> Default for OnceLock<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub type AtomicTendril = tendril::Tendril<tendril::fmt::UTF8, tendril::Atomic>;

/// Makes a [`AtomicTendril`] from a non-atomic tendril
#[inline(always)]
pub(crate) fn make_atomic_tendril(t: tendril::StrTendril) -> AtomicTendril {
    t.into_send().into()
}
