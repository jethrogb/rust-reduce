#![cfg_attr(feature = "cfg-target-has-atomic", feature(cfg_target_has_atomic))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![warn(clippy::all)]
pub mod future {
    mod future_obj {
        unsafe impl<'a, T, F> UnsafeFutureObj<'a, T> for &'a mut F
        where
            F: Future<Output = T> + Unpin + 'a,
        {
            fn into_raw(self) -> *mut (Future<Output = T> + 'a) {
                unimplemented!()
            }
        }
    }
}
