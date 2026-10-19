# Changelog

All notable changes to this project are documented in this file.

## [0.1.0]

### Added

- Initial release: shared `PhysAddr` and `PageFrame` traits, plus the
  `saturating_add` and `cap_size` helpers used by the allocator crates.
- `PhysAddr` provides the address limits and `align_up`/`align_down`, and
  exposes `saturating_add`/`cap_size` as provided methods (the free-function
  wrappers remain available).
- `PageFrame` extends `PhysAddr` with checked `usize` index conversions
  (`try_to_usize`, `from_usize`, `try_from_usize`) and the physical-address
  ↔ PFN conversions `phys_to_pfn`, `pfn_to_phys` and `pfn_ceil`.
- Implementations for all unsigned primitives (`u8`..`u128`, `usize`) and a
  small `prelude`.

[`PhysAddr`]: https://docs.rs/the-mm-core/latest/the_mm_core/trait.PhysAddr.html
[`PageFrame`]: https://docs.rs/the-mm-core/latest/the_mm_core/trait.PageFrame.html
