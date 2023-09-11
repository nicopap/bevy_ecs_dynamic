pub use builder::{AndFilter, AndFilters, DQuery, Fetch, OrFilters};
pub use dynamic_query::{DynamicItem, DynamicQuery};
pub use state::{DynamicState, Ticks};

/// Panic in debug mode, assume `true` in release mode.
macro_rules! assert_invariant {
    ($invariant:expr) => {{
        debug_assert!($invariant);
        if !$invariant {
            std::hint::unreachable_unchecked();
        }
    }};
}

mod builder;
mod debug_unchecked;
mod dynamic_query;
mod fetches;
mod filters;
mod iter;
mod jagged_array;
mod maybe_item;
mod state;

#[cfg(test)]
mod tests;
