#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: u16| {
    if let Ok(chan_spec) = csi::params::ChanSpec::try_from(data) {
        let _ = chan_spec.bandwidth();
    }
});
