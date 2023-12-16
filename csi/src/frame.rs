//! CSI extractor for [Nexmon](https://github.com/seemoo-lab/nexmon_csi)-patched
//! BCM4366c0 chips.
//!
//! Nexmon CSI is encoded in UDP packets, which in turn are defined
//! as follows:
//!
//! ```c
//! struct csi_udp_frame {
//!     struct ethernet_ip_udp_header hdrs;
//!     uint16 kk1; // magic bytes 0x1111
//!     int8 rssi;
//!     uint8 fc; //frame control
//!     uint8 SrcMac[6];
//!     uint16 seqCnt;
//!     uint16 csiconf;
//!     uint16 chanspec;
//!     uint16 chip;
//!     uint32 csi_values[];
//! } __attribute__((packed));
//! ```
//!
//! [GitHub source](https://github.com/seemoo-lab/nexmon_csi/blob/fdb25ef0e4e1402e968bb644d4914ad1a3d0a84d/src/csi_extractor.c#L135-L146)

use macaddr::MacAddr6;
use num_complex::Complex;
use num_traits::Zero;

use crate::params::{Bandwidth, ChanSpec};

/// Error returned when the chip ID is invalid.
#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid chip id")]
pub struct InvalidChip;

/// Chip ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chip {
    /// Broadcom BCM4366c0, used in the Asus RT-AC86U router.
    Bcm4366c0,
}

impl TryFrom<u16> for Chip {
    type Error = InvalidChip;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            106 => Ok(Self::Bcm4366c0),
            _ => Err(InvalidChip),
        }
    }
}

/// A reported CSI frame.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Received signal strength indicator (dBi).
    pub rssi: i8,
    pub frame_control: u8,
    /// Transmitter MAC address.
    pub source_mac: MacAddr6,
    /// "The two byte sequence number of the Wi-Fi frame that triggered
    /// the collection of the CSI contained in this packet."
    pub seq_cnt: u16,
    /// Core number.
    pub core: u8,
    /// Spatial stream number.
    pub spatial: u8,
    /// See the documentation for [`ChanSpec`].
    pub chan_spec: ChanSpec,
    /// Chip ID.
    pub chip: Chip,
    /// Complex CSI values.
    pub csi_values: Vec<Complex<f64>>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("not enough bytes")]
    NotEnoughBytes,
    #[error("not a Nexmon packet")]
    NotANexmonPacket,
    #[error("missing magic bytes")]
    MissingMagicBytes,
    #[error("invalid chip")]
    InvalidChip(#[from] InvalidChip),
}

impl Frame {
    pub fn from_slice(b: &[u8]) -> Result<Self, Error> {
        if b.len() < 60 {
            return Err(Error::NotEnoughBytes);
        }

        if &b[6..12] != b"NEXMON" {
            return Err(Error::NotANexmonPacket);
        }

        if b[42..44] != [0x11, 0x11] {
            return Err(Error::MissingMagicBytes);
        }

        let config_bytes = [b[54], b[55]];
        let mut config = u16::from_le_bytes(config_bytes);
        // Some versions of nexutil seem to encode the config in big endian.
        // If the config is larger than the maximum possible value, assume it's
        // big endian.
        if config > 0b111111 {
            config = u16::from_be_bytes(config_bytes);
        }
        let core = (config & 0b111) as u8;
        let spatial = ((config >> 3) & 0b111) as u8;

        let chan_spec = ChanSpec(u16::from_le_bytes([b[56], b[57]]));
        let chip = u16::from_le_bytes([b[58], b[59]]).try_into()?;

        let mut csi_values = unpack_csi(chan_spec.bandwidth(), &b[60..]);
        let n = csi_values.len() / 2;
        csi_values.rotate_right(n);

        Ok(Self {
            rssi: b[44] as i8,
            frame_control: b[45],
            source_mac: MacAddr6::new(b[46], b[47], b[48], b[49], b[50], b[51]),
            seq_cnt: u16::from_le_bytes([b[52], b[53]]),
            core,
            spatial,
            chan_spec,
            chip,
            csi_values,
        })
    }
}

/// Unpacks a complex value from the given 32-bit integer.
pub fn unpack_complex(i: u32) -> Complex<f64> {
    // unpack_float_acphy(
    //   nbits: 10,
    //   autoscale: 0,
    //   shft: 0,
    //   fmt: 1,
    //   nman: 12,
    //   nexp: 6,
    //   *nfftp,
    //   H,
    //   Hout,
    // );
    // https://github.com/seemoo-lab/nexmon_csi/blob/fdb25ef0e4e1402e968bb644d4914ad1a3d0a84d/utils/matlab/unpack_float.c#L119

    // const MAN_MASK: u32 = 0b111111111111; // 12 bits
    const MAN_MASK: u32 = 0b11111111111; // 11 bits
    const E_MASK: u32 = 0b111111; // 6 bits

    let exp = {
        let mut exp = (i & E_MASK) as i32;
        if exp >= 1 << 5 {
            // exponent is negative
            exp -= 1 << 6;
        }
        exp + 42
    };

    // exp < e_zero = -nman
    if exp < -12 {
        // exponent is too small
        return Complex::zero();
    }

    let mut re = ((i >> 18) & MAN_MASK) as i32;
    if i & 1 << 29 != 0 {
        // sign bit for real part is set
        re = -re;
    }

    let mut im = ((i >> 6) & MAN_MASK) as i32;
    if i & 1 << 17 != 0 {
        // sign bit for imaginary part is set
        im = -im;
    }

    if exp < 0 {
        re = re.overflowing_shr(-exp as _).0;
        im = im.overflowing_shr(-exp as _).0;
    } else {
        re = re.overflowing_shl(exp as _).0;
        im = im.overflowing_shl(exp as _).0;
    }

    Complex::new(re as f64, im as f64)
}

/// Unpacks the CSI values from the given buffer.
///
/// # Panics
///
/// Panics if the buffer is too short (less than `3.2 * <bandwidth in MHz>`).
pub fn unpack_csi(bw: Bandwidth, b: &[u8]) -> Vec<Complex<f64>> {
    // nsub = bw * 3.2
    let nsub = match bw {
        Bandwidth::Bw20 => 64,
        Bandwidth::Bw40 => 128,
        Bandwidth::Bw80 => 256,
        Bandwidth::Bw160 => 512,
    };

    assert!(b.len() >= nsub * 4, "not enough data");

    b.chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .map(unpack_complex)
        .collect::<Vec<_>>()
}
