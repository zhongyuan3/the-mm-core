//! Shared physical-address primitives for the `the-*` memory allocators.
//!
//! This crate is the common foundation of [`the-memblock`], [`the-buddy-system`]
//! and [`the-slab`]. It owns the two traits that describe a physical address:
//!
//! - [`PhysAddr`] — overflow-safe address and alignment arithmetic
//!   (`ALIGN`/`ALIGN_DOWN`).
//! - [`PageFrame`] — physical-address ↔ page-frame-number (PFN) conversion
//!   (`PFN_UP`/`PFN_DOWN`/`PFN_PHYS`) plus the checked conversions a page
//!   allocator needs to move between an address type and `usize` index space.
//!
//! Keeping the traits here means the three allocators have no dependency on
//! each other: memblock only needs [`PhysAddr`] (and [`PageFrame`] for its
//! PFN iterator), buddy and slab use [`PageFrame`], and a custom address
//! newtype implements the same traits for all of them.
//!
//! The crate is `no_std` and has no dependencies.
//!
//! [`the-memblock`]: https://crates.io/crates/the-memblock
//! [`the-buddy-system`]: https://crates.io/crates/the-buddy-system
//! [`the-slab`]: https://crates.io/crates/the-slab

#![no_std]

pub use crate::addr::PhysAddr;
pub use crate::pfn::PageFrame;

pub mod addr;
pub mod pfn;

/// The address and page-frame traits in one import.
///
/// ```
/// use the_mm_core::prelude::*;
///
/// let page = 0x1000usize;
/// assert_eq!(usize::align_up(1, page), page);
/// assert_eq!(usize::phys_to_pfn(0x1fff, page), 1);
/// assert_eq!(usize::pfn_ceil(0x1fff, page), 2);
/// assert_eq!(usize::from_usize(3).try_to_usize(), Some(3));
/// ```
pub mod prelude {
    pub use crate::addr::PhysAddr;
    pub use crate::pfn::PageFrame;
}
