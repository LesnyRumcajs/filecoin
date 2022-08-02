//! This module defines the low-level syscall API.
//!
//! # Wasm Syscall ABI
//!
//! Here we specify how the syscalls specified in this module map to Wasm. For more information on
//! the FVM syscall ABI, read the [syscall ABI spec][abi].
//!
//! By return type, there are three "kinds" of syscalls:
//!
//! 1. Syscalls that do not return (return `!`).
//! 2. Syscalls that do not return a value (return `Result<()>`).
//! 3. Syscalls that return a value (return `Result<T>`).
//!
//! Syscalls may also "return" values by writing them to out-pointers passed into the syscall by the
//! caller, but this is documented on a syscall-by-syscall basis.
//!
//! [abi]: https://github.com/filecoin-project/fvm-specs/blob/main/08-syscalls.md
//!
//! ## Kind 1: Divergent
//!
//! Syscalls that return `!` (e.g. [`vm::abort`]) have the signature:
//!
//! ```wat
//! (func $name (param ...) ... (result i32))
//! ```
//!
//! `result` is unused because the syscall never returns.
//!
//! ## Kind 2: Empty Return
//!
//! Syscalls that return `Result<()>` (nothing) have the signature:
//!
//! ```wat
//! (func $name (param ...) ... (result i32))
//! ```
//!
//! Here, `result` is an [`ErrorNumber`] or `0` on success.
//!
//! ## Kind 3: Non-Empty Return
//!
//! Syscalls that return `Result<T>` where `T` is a non-empty sized value have the signature:
//!
//! ```wat
//! (func $name (param $ret_ptr) (param ...) ... (result i32))
//! ```
//!
//! Here:
//!
//! - `result` is an [`ErrorNumber`] or `0` on success.
//! - `ret_ptr` is the offset (specified by the _caller_) where the FVM will write the return value
//!   if, and only if the result is `0` (success).
#[doc(inline)]
pub use fvm_shared::error::ErrorNumber;
#[doc(inline)]
pub use fvm_shared::sys::TokenAmount;


/// Generate a set of FVM syscall shims.
///
/// ```ignore
/// fvm_sdk::sys::fvm_syscalls! {
///     module = "my_wasm_module";
///
///     /// This method will translate to a syscall with the signature:
///     ///
///     ///     fn(arg: u64) -> u32;
///     ///
///     /// Where the returned u32 is the status code.
///     pub fn returns_nothing(arg: u64) -> Result<()>;
///
///     /// This method will translate to a syscall with the signature:
///     ///
///     ///     fn(out: u32, arg: u32) -> u32;
///     ///
///     /// Where `out` is a pointer to where the return value will be written and the returned u32
///     /// is the status code.
///     pub fn returns_value(arg: u64) -> Result<u64>;
///
///     /// This method will translate to a syscall with the signature:
///     ///
///     ///     fn(arg: u32) -> u32;
///     ///
///     /// But it will panic if this function returns.
///     pub fn aborts(arg: u32) -> !;
/// }
/// ```
macro_rules! fvm_syscalls {
    // Returns no values.
    (module = $module:literal; $(#[$attrs:meta])* $v:vis fn $name:ident($($args:ident : $args_ty:ty),*$(,)?) -> Result<()>; $($rest:tt)*) => {
        $(#[$attrs])*
        #[no_mangle]
        $v extern "C" fn $name($($args:$args_ty),*) -> u32 {
            unimplemented!("non-wasm dummy import");
        }
        $crate::macros::fvm_syscalls! {
            module = $module; $($rest)*
        }
    };
    // Returns a value.
    (module = $module:literal; $(#[$attrs:meta])* $v:vis fn $name:ident($($args:ident : $args_ty:ty),*$(,)?) -> Result<$ret:ty>; $($rest:tt)*) => {
        $(#[$attrs])*
        #[no_mangle]
        $v extern "C" fn $name($($args:$args_ty),*) -> u32 {
            unimplemented!("non-wasm dummy import");
        }
        $crate::macros::fvm_syscalls! {
            module = $module;
            $($rest)*
        }
    };
    // Does not return.
    (module = $module:literal; $(#[$attrs:meta])* $v:vis fn $name:ident($($args:ident : $args_ty:ty),*$(,)?) -> !; $($rest:tt)*) => {
        $(#[$attrs])*
        #[no_mangle]
        $v extern "C" fn $name($($args:$args_ty),*) -> ! {
            unimplemented!("non-wasm dummy import");
        }
        $crate::macros::fvm_syscalls! {
            module = $module;
            $($rest)*
        }
    };
    // Base case.
    (module = $module:literal;) => {};
}

pub(crate) use fvm_syscalls;