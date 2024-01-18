#![no_main]

use libfuzzer_sys::fuzz_target;
use csi::params::{ChanSpec, Band, Bandwidth};

fuzz_target!(|data: (u8, Bandwidth)| {
    let (channel, bandwidth) = data;
    let a = nexmon_test::chanspec_aton(&format!("{}/{}", channel, bandwidth.mhz()));
    match ChanSpec::new(channel, Band::Band5G, bandwidth) {
        Some(b) => assert_eq!(a, b.as_u16()),
        None => assert_eq!(a, 0),
    }
});
