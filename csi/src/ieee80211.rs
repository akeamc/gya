//! IEEE 802.11 definitions.
//!
//! References:
//! - [802.11ac: A Survival Guide](https://www.oreilly.com/library/view/80211ac-a-survival/9781449357702/ch02.html)

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
