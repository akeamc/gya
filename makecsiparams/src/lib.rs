use std::ffi::{c_int, CString};

use base64::{engine::general_purpose::STANDARD, Engine};

pub fn makecsiparams(args: &[&str]) -> String {
    let args = args
        .iter()
        .map(|arg| CString::new(arg.as_bytes()).unwrap())
        .collect::<Vec<_>>();
    let mut c_args = args
        .iter()
        .map(|arg| arg.as_ptr() as *mut i8)
        .collect::<Vec<_>>();

    let mut out = [0u8; 34];

    let status = unsafe {
        makecsiparams_sys::cli(c_args.len() as c_int, c_args.as_mut_ptr(), out.as_mut_ptr())
    };

    assert_eq!(status, 0, "makecsiparams failed");

    STANDARD.encode(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(
            super::makecsiparams(&["makecsiparams", "-c", "36/40", "-C", "0x5", "-N", "0x7"]),
            "JtgBdQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
        );
    }
}
