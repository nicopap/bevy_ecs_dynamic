pub(crate) trait DebugUnchecked<T> {
    /// # Safety
    /// This is undefined behavior if `Option::None`.
    unsafe fn prod_unchecked_unwrap(self) -> T;
}
impl<T> DebugUnchecked<T> for Option<T> {
    #[track_caller]
    unsafe fn prod_unchecked_unwrap(self) -> T {
        match self {
            Some(t) => t,
            #[cfg(debug_assertions)]
            None => unreachable!(),
            #[cfg(not(debug_assertions))]
            None => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
