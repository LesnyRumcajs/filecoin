#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod actor;
mod crypto;
mod debug;
mod gas;
mod ipld;
#[macro_use]
mod macros;
mod network;
mod rand;
mod send;
mod sself;
mod vm;

pub(crate) use macros::fvm_syscalls;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
