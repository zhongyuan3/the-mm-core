//! Page-frame conversions and the address type constraint used by the page
//! allocators.
//!
//! The kernel keeps allocator arithmetic in *page frame number* (PFN) space
//! (`include/linux/pfn.h`), never in physical address space, because a
//! physical address can be wider than a pointer: on 32-bit x86 with PAE,
//! `phys_addr_t` is 64-bit (`CONFIG_PHYS_ADDR_T_64BIT`) while
//! `unsigned long` is 32-bit. Buddy math such as `pfn ^ (1 << order)` is
//! therefore performed on the pointer-width PFN.
//!
//! This module follows the same split:
//!
//! - absolute physical addresses stay in the [`PhysAddr`] type;
//! - [`PageFrame`] converts between addresses, PFNs and `usize` index space;
//! - `usize` is used only for array indices, free list links and bitmap
//!   positions.
//!
//! The conversions mirror the kernel's `PFN_UP`, `PFN_DOWN` and `PFN_PHYS`
//! macros: [`PageFrame::phys_to_pfn`] is `PFN_DOWN`,
//! [`PageFrame::pfn_ceil`] is `PFN_UP`, and [`PageFrame::pfn_to_phys`] is
//! `PFN_PHYS`.
//!
//! The only narrowing operation is the checked [`PageFrame::try_to_usize`],
//! which is applied exclusively to *bounded* quantities: page offsets
//! relative to the arena base and the page size. Because the arena base is
//! aligned to the largest block and the managed page count fits in `usize`,
//! an offset that passed the allocator's range check is mathematically
//! representable and the conversion cannot truncate.

use crate::addr::PhysAddr;

/// A physical address type that can be split into PFN-space indices.
///
/// Implemented for all unsigned primitives; custom address newtypes (such
/// as the ones created with `#[derive(PageFrame)]`) can implement it by
/// forwarding to the inner type.
///
/// # Contract
///
/// - [`try_to_usize`](PageFrame::try_to_usize) must return `None` if `self`
///   is not representable as a `usize`. It is only ever called with page
///   offsets and page sizes, never with absolute addresses.
/// - [`from_usize`](PageFrame::from_usize) must widen `index` exactly when
///   the type is at least as wide as `usize`. Narrower types may saturate;
///   the page allocators reject arenas whose page count does not survive a
///   `from_usize`/`try_to_usize` round trip.
/// - The PFN conversions must use the same truncating division and
///   two's-complement semantics as the primitive implementations, and must
///   panic when `page_size` is zero.
pub trait PageFrame: PhysAddr {
    /// Narrows a bounded value (a page offset or a page size) to `usize`,
    /// returning `None` when the value is not representable.
    fn try_to_usize(self) -> Option<usize>;

    /// Widens a page index into this address type.
    ///
    /// Types narrower than `usize` saturate to their maximum instead of
    /// wrapping; the round trip check in the page allocators guarantees
    /// that all indices used by an accepted arena are exact.
    fn from_usize(index: usize) -> Self;

    /// Widens `index` exactly, returning `None` when the address type cannot
    /// represent it.
    ///
    /// The default implementation uses
    /// [`from_usize`](PageFrame::from_usize) and checks the round trip, so
    /// implementors do not need to provide it. Allocator validation code can
    /// use it instead of an explicit
    /// `from_usize`/`try_to_usize` round trip.
    fn try_from_usize(index: usize) -> Option<Self> {
        let value = Self::from_usize(index);
        (value.try_to_usize() == Some(index)).then_some(value)
    }

    /// Returns the page frame number containing `addr`, i.e.
    /// `floor(addr / page_size)`.
    ///
    /// Mirrors the kernel's `PFN_DOWN`. This is the inverse of
    /// [`PageFrame::pfn_to_phys`] for page-aligned addresses.
    ///
    /// # Panics
    ///
    /// Panics if `page_size` is zero.
    fn phys_to_pfn(addr: Self, page_size: Self) -> Self;

    /// Returns the start address of the page numbered `pfn`.
    ///
    /// Mirrors the kernel's `PFN_PHYS`. The result saturates to
    /// [`PhysAddr::MAX`] if `pfn * page_size` would overflow.
    ///
    /// # Panics
    ///
    /// Panics if `page_size` is zero.
    fn pfn_to_phys(pfn: Self, page_size: Self) -> Self;

    /// Returns the first page frame number at or after `addr`, i.e.
    /// `ceil(addr / page_size)`.
    ///
    /// Mirrors the kernel's `PFN_UP`. This is a one-way rounding operation:
    /// for an unaligned `addr` it returns the PFN of the next page boundary,
    /// not the PFN containing `addr` (use
    /// [`PageFrame::phys_to_pfn`] for that).
    ///
    /// Unlike `phys_to_pfn(align_up(addr, page_size))`, this stays exact
    /// when `addr` lies in the final partial page of the address space:
    /// `align_up` saturates to [`PhysAddr::MAX`] there, which would round
    /// down instead of up.
    ///
    /// # Panics
    ///
    /// Panics if `page_size` is zero.
    fn pfn_ceil(addr: Self, page_size: Self) -> Self;
}

macro_rules! impl_page_frame {
    ($($t:ty),+ $(,)?) => {
        $(
            impl PageFrame for $t {
                fn try_to_usize(self) -> Option<usize> {
                    usize::try_from(self).ok()
                }

                fn from_usize(index: usize) -> Self {
                    <$t>::try_from(index).unwrap_or(<$t>::MAX)
                }

                fn phys_to_pfn(addr: Self, page_size: Self) -> Self {
                    assert!(page_size != 0, "page_size must be non-zero");
                    addr / page_size
                }

                fn pfn_to_phys(pfn: Self, page_size: Self) -> Self {
                    assert!(page_size != 0, "page_size must be non-zero");
                    if pfn > Self::MAX / page_size {
                        Self::MAX
                    } else {
                        pfn * page_size
                    }
                }

                fn pfn_ceil(addr: Self, page_size: Self) -> Self {
                    assert!(page_size != 0, "page_size must be non-zero");
                    let q = addr / page_size;
                    if addr % page_size != 0 {
                        q + 1
                    } else {
                        q
                    }
                }
            }
        )+
    };
}

impl_page_frame!(u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_to_usize_accepts_representable_values() {
        assert_eq!(0u8.try_to_usize(), Some(0));
        assert_eq!(0xffu8.try_to_usize(), Some(0xff));
        assert_eq!(0x1234u32.try_to_usize(), Some(0x1234));
        assert_eq!(0x1234_5678u64.try_to_usize(), Some(0x1234_5678));
        assert_eq!(0x1234_5678usize.try_to_usize(), Some(0x1234_5678));
    }

    #[test]
    fn try_to_usize_rejects_wider_than_usize() {
        let too_wide = u128::from(u64::MAX) + 1;
        assert_eq!(too_wide.try_to_usize(), None);

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(0x1_0000_0000u64.try_to_usize(), None);
        }
    }

    #[test]
    fn from_usize_widens_exactly() {
        assert_eq!(u64::from_usize(0x1234), 0x1234);
        assert_eq!(u128::from_usize(0x1234), 0x1234);
        assert_eq!(usize::from_usize(0x1234), 0x1234);
    }

    #[test]
    fn from_usize_saturates_for_narrow_types() {
        assert_eq!(u8::from_usize(0xff), 0xff);
        assert_eq!(u8::from_usize(0x100), u8::MAX);
        assert_eq!(u16::from_usize(0x1_0000), u16::MAX);
    }

    #[test]
    fn round_trip_is_identity_for_representable_indices() {
        for index in [0usize, 1, 0xff, 0x1000, 0x1234] {
            assert_eq!(u64::from_usize(index).try_to_usize(), Some(index));
            assert_eq!(usize::from_usize(index).try_to_usize(), Some(index));
        }
    }

    #[test]
    fn try_from_usize_is_checked() {
        assert_eq!(u64::try_from_usize(0x1234), Some(0x1234));
        assert_eq!(u128::try_from_usize(0x1234), Some(0x1234));
        assert_eq!(usize::try_from_usize(0x1234), Some(0x1234));
        assert_eq!(u8::try_from_usize(0xff), Some(0xff));
        assert_eq!(u8::try_from_usize(0x100), None);
        assert_eq!(u16::try_from_usize(0x1_0000), None);
    }

    #[test]
    fn pfn_conversions_basic() {
        assert_eq!(usize::phys_to_pfn(0x0, 0x1000), 0);
        assert_eq!(usize::phys_to_pfn(0xfff, 0x1000), 0);
        assert_eq!(usize::phys_to_pfn(0x1000, 0x1000), 1);
        assert_eq!(usize::phys_to_pfn(0x1001, 0x1000), 1);

        assert_eq!(usize::pfn_ceil(0x0, 0x1000), 0);
        assert_eq!(usize::pfn_ceil(0x1, 0x1000), 1);
        assert_eq!(usize::pfn_ceil(0x1000, 0x1000), 1);
        assert_eq!(usize::pfn_ceil(0x1001, 0x1000), 2);

        assert_eq!(usize::pfn_to_phys(0, 0x1000), 0);
        assert_eq!(usize::pfn_to_phys(1, 0x1000), 0x1000);
        assert_eq!(usize::pfn_to_phys(2, 0x1000), 0x2000);
    }

    #[test]
    fn pfn_helpers_at_address_space_end() {
        let max = usize::MAX;
        assert_eq!(usize::pfn_ceil(max, 0x1000), max / 0x1000 + 1);
        assert_eq!(usize::phys_to_pfn(max, 0x1000), max / 0x1000);
    }

    #[test]
    fn pfn_to_phys_saturates_at_max() {
        assert_eq!(
            usize::pfn_to_phys(usize::MAX / 0x1000 + 1, 0x1000),
            usize::MAX
        );
        assert_eq!(usize::pfn_to_phys(usize::MAX, 0x1000), usize::MAX);
    }

    #[test]
    fn pfn_roundtrip_is_identity_for_full_pages() {
        let base = 0x1234_0000usize;
        let pfn = usize::phys_to_pfn(base, 0x1000);
        assert_eq!(usize::pfn_to_phys(pfn, 0x1000), base);
    }

    #[test]
    fn pfn_conversions_work_for_all_widths() {
        assert_eq!(u8::pfn_ceil(0xff, 0x10), 16);
        assert_eq!(u8::phys_to_pfn(0xff, 0x10), 15);
        assert_eq!(u32::pfn_to_phys(0x1000, 0x1000), 0x100_0000);
        assert_eq!(
            u64::phys_to_pfn(0xffff_ffff_ffff_f000, 0x1000),
            0x000f_ffff_ffff_ffff
        );
    }

    #[test]
    fn pfn_ceil_is_exact_for_the_final_partial_page() {
        // `align_up(0xff, 0x10)` saturates to 0xff, so
        // `phys_to_pfn(align_up(...))` would be 15 instead of 16.
        assert_eq!(u8::phys_to_pfn(0xff, 0x10), 15);
        assert_eq!(u8::pfn_ceil(0xff, 0x10), 16);
        assert_eq!(u8::pfn_to_phys(16, 0x10), u8::MAX);
    }

    #[test]
    #[should_panic(expected = "page_size must be non-zero")]
    fn pfn_ceil_rejects_zero_page_size() {
        usize::pfn_ceil(0x1000, 0);
    }

    #[test]
    #[should_panic(expected = "page_size must be non-zero")]
    fn phys_to_pfn_rejects_zero_page_size() {
        usize::phys_to_pfn(0x1000, 0);
    }

    #[test]
    #[should_panic(expected = "page_size must be non-zero")]
    fn pfn_to_phys_rejects_zero_page_size() {
        usize::pfn_to_phys(1, 0);
    }
}
