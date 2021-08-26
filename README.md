# `AtomicDouble<T>`
A Rust library which provides 128-bit atomic operations for generic types on supported architectures (**currently only x86_64 with cmpxchg16b is supported**).
In cases where atomic operations can't be supported fallback implementation using spin-locks has been provided. You can use the `AtomicDouble::<T>::is_lock_free()` function to check whether native atomic operations are supported for a given type.
Note that the library is tailor made for 128-bit operations, types violating the size constraints will use the fallback implementation.
Fallback implementation is enabled by default and can be disabled by adding `default-features = false` to the dependency declaration.

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
atomicdouble = "0.1"
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Credits
This crate is effectively a 128-bit only version of [`Atomic<T>`](https://github.com/Amanieu/atomic-rs) crate. `Atomic<T>` crate doesn't work for 128 bit atomics for now, as rust doesnt have support for AtomicU128/AtomicI128 yet. In the mean time AtomicDouble<T> can be used as a replacement.
