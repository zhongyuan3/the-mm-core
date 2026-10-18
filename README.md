# the-mm-core

Shared physical-address primitives for the `the-*` memory allocators
([`the-memblock`], [`the-buddy-system`], [`the-slab`]).

This crate is `no_std` and dependency-free. It exists so the allocators do
not have to depend on each other:

- [`PhysAddr`] — overflow-safe address and alignment arithmetic (`ALIGN`,
  `ALIGN_DOWN`).
- [`PageFrame`] — physical address ↔ page-frame-number (PFN) conversion
  (`PFN_UP`, `PFN_DOWN`, `PFN_PHYS`) and checked conversions between an
  address type, page offsets and `usize` index space.

```rust
use the_mm_core::prelude::*;

let page = 0x1000usize;
assert_eq!(usize::align_up(1, page), page);
assert_eq!(usize::phys_to_pfn(0x1fff, page), 1);
assert_eq!(usize::pfn_ceil(0x1fff, page), 2);
assert_eq!(usize::from_usize(3).try_to_usize(), Some(3));
```

The traits are implemented for every unsigned primitive. The allocator crates
re-export them and provide derive macros for single-field newtypes:
`#[derive(the_memblock::PhysAddr)]` and
`#[derive(the_buddy_system::PageFrame)]`.

[`PhysAddr`]: https://docs.rs/the-mm-core/latest/the_mm_core/trait.PhysAddr.html
[`PageFrame`]: https://docs.rs/the-mm-core/latest/the_mm_core/trait.PageFrame.html
[`the-memblock`]: https://crates.io/crates/the-memblock
[`the-buddy-system`]: https://crates.io/crates/the-buddy-system
[`the-slab`]: https://crates.io/crates/the-slab

## License

MIT.
