//! IEEE 802.11 definitions.
//!
//! References:
//! - [802.11ac: A Survival Guide](https://www.oreilly.com/library/view/80211ac-a-survival/9781449357702/ch02.html)
//! - [List of WLAN channels (Wikipedia)](https://en.wikipedia.org/wiki/List_of_WLAN_channels#5_GHz_(802.11a/h/n/ac/ax))

use std::marker::PhantomData;

use ndarray::Array1;
use uom::si::{f64::Frequency, frequency::hertz};

/// Speed of light in meters per second.
const C: f64 = 299_792_458.;

/// Band.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Band {
    /// 2.4 GHz.
    Band2G,
    /// 5 GHz.
    Band5G,
}

/// Bandwidth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Bandwidth {
    /// 20 MHz.
    Bw20,
    /// 40 MHz.
    Bw40,
    /// 80 MHz.
    Bw80,
    /// 160 MHz.
    Bw160,
}

impl Bandwidth {
    /// Returns the bandwidth in MHz.
    pub const fn mhz(&self) -> u8 {
        match self {
            Bandwidth::Bw20 => 20,
            Bandwidth::Bw40 => 40,
            Bandwidth::Bw80 => 80,
            Bandwidth::Bw160 => 160,
        }
    }

    /// Returns the number of subcarriers rounded up to the nearest power of 2.
    ///
    /// Note that this is not the same as the number of _usable_ subcarriers.
    ///
    /// | PHY standard             | Subcarrier range                                   | Pilot subcarriers                           | Subcarriers (total/data)          |
    /// |--------------------------|----------------------------------------------------|---------------------------------------------|-----------------------------------|
    /// | 802.11n/802.11ac, 20 MHz | –28 to –1, +1 to +28                               | ±7, ±21                                     | 56 total, 52 usable (7% pilots)   |
    /// | 802.11n/802.11ac, 40 MHz | –58 to –2, +2 to +58                               | ±11, ±25, ±53                               | 114 total, 108 usable (5% pilots) |
    /// | 802.11ac, 80 MHz         | –122 to –2, +2 to +122                             | ±11, ±39, ±75, ±103                         | 242 total, 234 usable (3% pilots) |
    /// | 802.11ac, 160 MHz        | –250 to –130, –126 to –6, +6 to +126, +130 to +250 | ±25, ±53, ±89, ±117, ±139, ±167, ±203, ±231 | 484 total, 468 usable (3% pilots) |
    pub const fn nsub_pow2(&self) -> usize {
        match self {
            Bandwidth::Bw20 => 64,   // 56 total
            Bandwidth::Bw40 => 128,  // 108 total
            Bandwidth::Bw80 => 256,  // 242 total
            Bandwidth::Bw160 => 512, // 484 total
        }
    }

    /// The frequency in Hz.
    pub const fn freq(&self) -> Frequency {
        Frequency {
            dimension: PhantomData,
            units: PhantomData,
            value: match self {
                Bandwidth::Bw20 => 20e6,
                Bandwidth::Bw40 => 40e6,
                Bandwidth::Bw80 => 80e6,
                Bandwidth::Bw160 => 160e6,
            },
        }
    }
}

/// OFDM subcarriers can be either pilot, data or zero/null.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubcarrierType {
    /// Pilot subcarrier.
    Pilot,
    /// Data subcarrier.
    Data,
    /// Zero/null (unused) subcarrier.
    Zero,
}

/// Subcarrier type for 802.11n/802.11ac, 20 MHz.
///
/// ```
/// # use csi::ieee80211::{subcarrier_type_20mhz, SubcarrierType};
/// let usable = (-128..127).filter(|&i| subcarrier_type_20mhz(i) == SubcarrierType::Data).count();
///
/// assert_eq!(usable, 52);
/// ```
pub const fn subcarrier_type_20mhz(i: i8) -> SubcarrierType {
    match i {
        -21 | -7 | 7 | 21 => SubcarrierType::Pilot,
        -28..=-1 | 1..=28 => SubcarrierType::Data,
        _ => SubcarrierType::Zero,
    }
}

/// Subcarrier type for 802.11n/802.11ac, 40 MHz.
///
/// ```
/// # use csi::ieee80211::{subcarrier_type_40mhz, SubcarrierType};
/// let usable = (-128..127).filter(|&i| subcarrier_type_40mhz(i) == SubcarrierType::Data).count();
///
/// assert_eq!(usable, 108);
/// ```
pub const fn subcarrier_type_40mhz(i: i8) -> SubcarrierType {
    match i {
        -53 | -25 | -11 | 11 | 25 | 53 => SubcarrierType::Pilot,
        -58..=-2 | 2..=58 => SubcarrierType::Data,
        _ => SubcarrierType::Zero,
    }
}

/// Subcarrier type for 802.11ac, 80 MHz.
///
/// ```
/// # use csi::ieee80211::{subcarrier_type_80mhz, SubcarrierType};
/// let usable = (-128..127).filter(|&i| subcarrier_type_80mhz(i) == SubcarrierType::Data).count();
///
/// assert_eq!(usable, 234);
/// ```
pub const fn subcarrier_type_80mhz(i: i8) -> SubcarrierType {
    match i {
        -103 | -75 | -39 | -11 | 11 | 39 | 75 | 103 => SubcarrierType::Pilot,
        -122..=-2 | 2..=122 => SubcarrierType::Data,
        _ => SubcarrierType::Zero,
    }
}

/// Subcarrier type for 802.11ac, 160 MHz.
///
/// ```
/// # use csi::ieee80211::{subcarrier_type_160mhz, SubcarrierType};
/// let usable = (-256..255).filter(|&i| subcarrier_type_160mhz(i) == SubcarrierType::Data).count();
///
/// assert_eq!(usable, 468);
/// ```
pub const fn subcarrier_type_160mhz(i: i16) -> SubcarrierType {
    match i {
        -231 | -203 | -167 | -139 | -117 | -89 | -53 | -25 | 25 | 53 | 89 | 117 | 139 | 167
        | 203 | 231 => SubcarrierType::Pilot,
        -250..=-130 | -126..=-6 | 6..=126 | 130..=250 => SubcarrierType::Data,
        _ => SubcarrierType::Zero,
    }
}

fn channel_mhz(channel: u8) -> u32 {
    5000 + 5 * channel as u32
}

/// Returns the subcarrier frequencies (in Hz) for a given center frequency and bandwidth.
///
/// Not all returned subcarriers are usable.
///
/// ```
/// # use csi::ieee80211::{subcarrier_freqs, Bandwidth};
/// let freqs = subcarrier_freqs(58, Bandwidth::Bw80);
/// assert_eq!(freqs.len(), 256);
/// assert_eq!(freqs[0], 5.250e9);
/// assert_eq!(freqs[255], 5.330e9);
/// ```
pub fn subcarrier_freqs(center: u8, bandwidth: Bandwidth) -> Array1<f64> {
    let center = channel_mhz(center) as f64 * 1e6;
    let half_bw = bandwidth.freq().get::<hertz>() / 2.;

    Array1::linspace(center - half_bw, center + half_bw, bandwidth.nsub_pow2())
}

/// Returns the subcarrier wavelengths for a given center frequency and bandwidth.
pub fn subcarrier_lambda(center: u8, bandwidth: Bandwidth) -> Array1<f64> {
    let mut v = subcarrier_freqs(center, bandwidth);
    v.mapv_inplace(|f| C / f);
    v
}

#[cfg(test)]
mod tests {
    use crate::ieee80211::channel_mhz;

    #[test]
    fn channel_freq() {
        assert_eq!(channel_mhz(32), 5160);
        assert_eq!(channel_mhz(120), 5600);
    }
}
