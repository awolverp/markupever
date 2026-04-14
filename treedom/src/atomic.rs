pub type AtomicTendril = tendril::Tendril<tendril::fmt::UTF8, tendril::Atomic>;

/// Makes a [`AtomicTendril`] from a non-atomic tendril
#[inline(always)]
pub(crate) fn make_atomic_tendril(t: tendril::StrTendril) -> AtomicTendril {
    t.into_send().into()
}
