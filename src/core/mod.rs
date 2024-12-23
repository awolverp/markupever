// pub type AtomicTendril = tendril::Tendril<tendril::fmt::UTF8, tendril::Atomic>;

// #[inline(always)]
// pub(super) fn make_atomic_tendril(t: tendril::StrTendril) -> AtomicTendril {
//     t.into_send().into()
// }

pub mod arcdom;
pub mod send;
