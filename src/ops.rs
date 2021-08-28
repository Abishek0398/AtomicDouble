use core::mem;
use core::sync::atomic::Ordering;
use core::arch::x86_64::cmpxchg16b;

#[cfg(feature = "fallback")]
use crate::fallback;

#[inline(never)]
#[target_feature(enable="cmpxchg16b")]
unsafe fn compare_exchange_intrinsic<T>(dst: *mut u128, 
    current: u128, 
    new: u128, 
    success: Ordering, 
    failure: Ordering
) -> Result<u128,u128>{
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("cmpxchg16b") &&
        mem::size_of::<T>() == 16
        {
            let res = cmpxchg16b(dst, current, new, success, failure);
            if res == current {
                return Ok(res);
            }
            else {
                return Err(res);
            }
        }
    }

    #[cfg(feature = "fallback")]
    return fallback::atomic_compare_exchange(dst, current, new);
    #[cfg(not(feature = "fallback"))]
    panic!("Atomic operations for type `{}` are not available as the `fallback` feature of the `atomicdouble` crate is disabled.", core::any::type_name::<T>());
}

#[inline]
pub fn atomic_is_lock_free<T>() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("cmpxchg16b")&&
        mem::size_of::<T>() == 16 {
            return true
        }
    }
    false
}
#[inline]
pub unsafe fn atomic_load<T>(dst: *mut T, order: Ordering) -> T {
    let res = compare_exchange_intrinsic::<T>(
        dst as *mut u128,
        0,
        0,
        order,
        order,
    );
    match  res{
        Ok(load_val) => mem::transmute_copy(&load_val),
        Err(load_val) => mem::transmute_copy(&load_val),
    }
}
#[inline]
pub unsafe fn atomic_store<T>(dst: *mut T, val: T, order: Ordering) {
    let mut res = Err(0);
    let mut current:u128 = mem::transmute_copy(&val);
    let new:u128 = mem::transmute_copy(&val);
    while res.is_err() {
        res = compare_exchange_intrinsic::<T>(
            dst as *mut u128,
            current,
            new,
            order,
            order,
        );
        match  res{
            Ok(_) => {},
            Err(load_val) => {current = load_val},
        };
    }
    
}

#[inline]
unsafe fn map_result<T, U>(r: Result<T, T>) -> Result<U, U> {
    match r {
        Ok(x) => Ok(mem::transmute_copy(&x)),
        Err(x) => Err(mem::transmute_copy(&x)),
    }
}
#[inline]
pub unsafe fn atomic_compare_exchange<T>(
    dst: *mut T,
    current: T,
    new: T,
    success: Ordering,
    failure: Ordering,
) -> Result<T, T> {
        map_result(compare_exchange_intrinsic::<T>(
            dst as *mut u128,
            mem::transmute_copy(&current),
            mem::transmute_copy(&new),
            success,
            failure,
        ))
}
#[inline]
pub unsafe fn atomic_add<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
{
    let mut res:Result<u128,u128> = Err(0);
    let mut current:u128 = mem::transmute_copy(&atomic_load(dst, order));
    let mut new:u128 = current.wrapping_add(mem::transmute_copy(&val));
    while res.is_err() {
        res = compare_exchange_intrinsic::<T>(
            dst as *mut u128,
            current,
            new,
            order,
            order,
        );
        match  res{
            Ok(load_val) => {
                return mem::transmute_copy(&load_val);
            },
            Err(load_val) => {
                current = load_val;
                new = load_val.wrapping_add(mem::transmute_copy(&val));
            }
        };
    }
    val
}
#[inline]
pub unsafe fn atomic_sub<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
{
    let mut res = Err(0);
    let mut current:u128 = mem::transmute_copy(&atomic_load(dst, order));
    let mut new:u128 = current.wrapping_sub(mem::transmute_copy(&val));
    while res.is_err() {
        res = compare_exchange_intrinsic::<T>(
            dst as *mut u128,
            current,
            new,
            order,
            order,
        );
        match  res{
            Ok(load_val) => {
                return mem::transmute_copy(&load_val);
            },
            Err(load_val) => {
                current = load_val;
                new = load_val.wrapping_sub(mem::transmute_copy(&val));
            }
        };
    }
    val
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;
    use std::boxed::Box;
    use crate::AtomicDouble;
    use crate::Ordering::SeqCst;

    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
    struct Bar(u64, u64);

    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
    struct SizeBar(u32, u32);

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    struct Node {
        head_ptr : Option< NonNull<i32> >,
        head_count : usize
    }

    #[test]
    fn atomic_bar() {
        let a:AtomicDouble::<Bar> = AtomicDouble::default();
        assert_eq!(AtomicDouble::<Bar>::is_lock_free(), true);
        a.load(SeqCst);
        assert_eq!(format!("{:?}", a), "AtomicDouble(Bar(0, 0))");
        assert_eq!(a.load(SeqCst), Bar(0, 0));
        a.store(Bar(1, 1), SeqCst);
        assert_eq!(
            a.compare_exchange(Bar(5, 5), Bar(45, 45), SeqCst, SeqCst),
            Err(Bar(1, 1))
        );
        assert_eq!(
            a.compare_exchange(Bar(1, 1), Bar(3, 3), SeqCst, SeqCst),
            Ok(Bar(1, 1))
        );
        assert_eq!(a.load(SeqCst), Bar(3, 3));
    }

    #[test]
    fn atomic_sizebar() {
        assert_eq!(AtomicDouble::<SizeBar>::is_lock_free(), false);
    }

    #[test]
    fn atomic_node() {
        let x = Box::new(5);
        let y = Box::new(10);

        let temp_node_x = Node {
            head_ptr:NonNull::new(Box::into_raw(x)),
            head_count:3
        };

        let temp_node_y = Node {
            head_ptr:NonNull::new(Box::into_raw(y)),
            head_count:2
        };

        let a:AtomicDouble::<Node> = AtomicDouble::new(temp_node_x);
        assert_eq!(AtomicDouble::<Node>::is_lock_free(), true);

        let load_test = a.load(SeqCst);
        unsafe {
            let load_test_x = Box::from_raw(load_test.head_ptr.unwrap().as_ptr());
            assert_eq!(*load_test_x, 5);
            assert_eq!(load_test.head_count, 3);
        };
        a.store(temp_node_y, SeqCst);
       assert_eq!(
            a.compare_exchange(temp_node_x, temp_node_y, SeqCst, SeqCst),
            Err(temp_node_y)
        );
        assert_eq!(
            a.compare_exchange(temp_node_y, temp_node_x, SeqCst, SeqCst),
            Ok(temp_node_y)
        );
        assert_eq!(a.load(SeqCst), temp_node_x);

        a.fetch_add(Node{
            head_ptr:None,
            head_count:usize::MAX,
        }, SeqCst);
        assert_eq!(a.load(SeqCst).head_count,2);

        a.fetch_sub(Node{
            head_ptr:None,
            head_count:3,
        }, SeqCst);
        assert_eq!(a.load(SeqCst).head_count,usize::MAX);
    } 
}