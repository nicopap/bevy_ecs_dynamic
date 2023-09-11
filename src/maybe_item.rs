use std::mem::{transmute, MaybeUninit};

use crate::DynamicItem;

#[derive(Debug)]
pub(crate) struct MaybeDynamicItem(MaybeUninit<DynamicItem<'static>>);
impl Clone for MaybeDynamicItem {
    fn clone(&self) -> Self {
        Self::uninit()
    }
}
impl MaybeDynamicItem {
    pub(crate) fn uninit() -> Self {
        MaybeDynamicItem(MaybeUninit::uninit())
    }
    /// Note that you must never call `assume_init[_mut]` with a lifetime outliving `'w`.
    pub(crate) fn set<'w>(&mut self, value: DynamicItem<'w>) {
        // SAFETY: This is safe as long as we don't dereference the stored value
        // with an eroneous lifetime. Which is guarenteed by the other methods
        // in this `impl` block.
        let static_value = unsafe { transmute::<DynamicItem<'w>, DynamicItem<'static>>(value) };

        self.0.write(static_value);
    }
}

/// SAFETY:
/// - All items must outlive `'w`.
/// - All items must be initialized.
pub(crate) unsafe fn assume_init_mut<'w>(items: &mut [MaybeDynamicItem]) -> &mut [DynamicItem<'w>] {
    // SAFETY: I really don't know
    unsafe { transmute(items) }
}
