# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Changed

- **PFN conversions moved to `PageFrame`.** `phys_to_pfn`, `pfn_to_phys` and
  the exact round-up helper `pfn_ceil` now live on `PageFrame`, together
  with the existing `usize` index conversions. `PhysAddr` is now purely
  address/alignment arithmetic.
- `pfn_down` was removed in favour of `phys_to_pfn`; `pfn_up` was renamed to
  `pfn_ceil` and its documentation now states that it rounds up to the next
  page boundary rather than returning the page containing the address.
- `PhysAddr` now provides `saturating_add` and `cap_size` as trait methods;
  the free functions remain as forwarding wrappers.
- `PageFrame` gained a provided `try_from_usize` method, which widens an
  index only when the address type can represent it exactly. The page
  allocators use it instead of manual `from_usize`/`try_to_usize` round
  trips.

## [0.1.0]

### Added

- Initial release: shared `PhysAddr` and `PageFrame` traits, plus the
  `saturating_add` and `cap_size` helpers used by the allocator crates.

[`PhysAddr`]: https://docs.rs/the-mm-core/latest/the_mm_core/trait.PhysAddr.html
[`PageFrame`]: https://docs.rs/the-mm-core/latest/the_mm_core/trait.PageFrame.html
