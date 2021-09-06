# `AtomicDouble<T>`
A Rust library which provides 128-bit atomic operations for generic types on supported architectures (**currently only x86_64 with cmpxchg16b is supported**). In cases where atomic operations can't be supported fallback implementation using spin-locks has been provided.

You can use the `AtomicDouble::<T>::is_lock_free()` function to check whether native atomic operations are supported for a given type.
Note that the library is tailor made for 128-bit operations, types violating the size constraints will use the fallback implementation.
Fallback implementation is enabled by default and can be disabled by adding `default-features = false` to the dependency declaration.

This crate requires nightly.

[Documentation](https://docs.rs/atomicdouble)

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
atomicdouble = "0.1.3"
```

## Example
```rust
use std::ptr::NonNull;

use atomicdouble::AtomicDouble;
use atomicdouble::Ordering::SeqCst;

#[derive(Copy, Clone, Eq, PartialEq, Debug,Default)]
struct Node {
    head_ptr : Option< NonNull<i32> >,
    head_count : usize //assuming 64-bit machine
}

fn main() {
    let x = Box::new(5);
    let temp_node_x = Node {
        head_ptr:NonNull::new(Box::into_raw(x)),
        head_count:3
    };
    let a:AtomicDouble::<Node> = AtomicDouble::new(temp_node_x);
    println!("{}",AtomicDouble::<Node>::is_lock_free());
    let load_test = a.load(SeqCst);
    unsafe {
        let load_test_x = Box::from_raw(load_test.head_ptr.unwrap().as_ptr());
        println!("{}",*load_test_x);
        println!("{}",load_test.head_count);
    };
}
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
