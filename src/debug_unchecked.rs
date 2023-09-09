pub(crate) trait DebugUnchecked<T> {
    unsafe fn release_unchecked_unwrap(self) -> T;
}
impl<T> DebugUnchecked<T> for Option<T> {
    #[track_caller]
    unsafe fn release_unchecked_unwrap(self) -> T {
        match self {
            Some(t) => t,
            #[cfg(debug_assertions)]
            None => unreachable!(),
            #[cfg(not(debug_assertions))]
            None => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
