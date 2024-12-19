#[allow(unused_imports)]
use std::ffi::c_char;

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("pin-rs/include/wrapper.h");

        pub unsafe fn PIN_Init(argc: i32, argv: *mut *mut c_char) -> bool;
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        use super::*;
        use crate::ffi::PIN_Init;
        println!("Running test");

        let argc = 1;
        let mut argv = ["pin\0".as_ptr() as *mut c_char];
        let argv = argv.as_mut_ptr();
        let _result = unsafe { PIN_Init(argc, argv) };
    }
}
