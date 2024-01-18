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
use ndarray::Array1;
use num_complex::Complex;
use num_traits::Zero;

use crate::{ieee80211::Bandwidth, params::ChanSpec};

/// Error returned when the chip ID does not correspond to any of
/// the [`Chip`] variants.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("unknown chip")]
pub struct UnknownChip;

/// Different types of WiFi chips.
///
/// `TryFrom<u16>` is implemented to convert a two-byte sequence into
/// a `Chip` variant:
/// ```
/// # use std::convert::TryFrom;
/// # use csi::frame::Chip;
/// assert_eq!(Chip::try_from(0x006a), Ok(Chip::Bcm4366c0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chip {
    /// Broadcom BCM4366c0, used in the Asus RT-AC86U router. This is represented
    /// by the two-byte sequence `0x006a`.
    Bcm4366c0,
}

impl TryFrom<u16> for Chip {
    type Error = UnknownChip;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            106 => Ok(Self::Bcm4366c0),
            _ => Err(UnknownChip),
        }
    }
}

/// A reported CSI frame.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Received signal strength indicator (dBi).
    pub rssi: i8,
    /// Transmitter MAC address.
    pub source_mac: MacAddr6,
    /// The two byte sequence number of the Wi-Fi frame that triggered
    /// the collection of the CSI contained in this packet.
    pub seq_cnt: u16,
    /// Core number.
    pub core: u8,
    /// Spatial stream number.
    pub spatial: u8,
    /// See the documentation for [`ChanSpec`].
    pub chan_spec: ChanSpec,
    /// Chip that generated the CSI frame.
    pub chip: Chip,
    /// Complex CSI values.
    pub csi: Array1<Complex<f64>>,
}

/// Error returned when parsing a CSI frame.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// The given byte slice is too short.
    #[error("not enough bytes")]
    NotEnoughBytes,
    /// A Nexmon packet should have the magic bytes `NEXMON` at offset 6.
    #[error("not a Nexmon packet")]
    NotANexmonPacket,
    /// A Nexmon packet should have the magic bytes `0x1111` at offset 42.
    #[error("missing magic bytes")]
    MissingMagicBytes,
    /// See [`UnknownChip`].
    #[error(transparent)]
    UnknownChip(#[from] UnknownChip),
    /// See [`crate::params::ParseChanSpecError`].
    #[error(transparent)]
    InvalidChanSpec(#[from] crate::params::ParseChanSpecError),
}

impl Frame {
    /// Parses a CSI frame from the given byte slice.
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

        // BCM4366c0 always sets the frame control to 0, so it's not very
        // useful.
        // let frame_control = b[45];

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

        let chan_spec: ChanSpec = u16::from_le_bytes([b[56], b[57]]).try_into()?;
        let chip = u16::from_le_bytes([b[58], b[59]]).try_into()?;

        let csi = &b[60..];

        if csi.len() < chan_spec.bandwidth().nsub() * 4 {
            // not enough bytes
            return Err(Error::NotEnoughBytes);
        }

        let mut csi = unpack_csi(csi).collect::<Vec<_>>();
        let n = csi.len() / 2;
        csi.rotate_right(n);

        Ok(Self {
            rssi: b[44] as i8,
            source_mac: MacAddr6::new(b[46], b[47], b[48], b[49], b[50], b[51]),
            seq_cnt: u16::from_le_bytes([b[52], b[53]]),
            core,
            spatial,
            chan_spec,
            chip,
            csi: csi.into(),
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
pub fn unpack_csi(b: &[u8]) -> impl Iterator<Item = Complex<f64>> + '_ {
    b.chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .map(unpack_complex)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_frame() {
        let bytes = b"\xff\xff\xff\xff\xff\xff\x4e\x45\x58\x4d\x4f\x4e\x08\x00\x45\x00\x01\x2e\x00\x01\x00\x00\x01\x11\xa4\xab\x0a\x0a\x0a\x0a\xff\xff\xff\xff\x15\x7c\x15\x7c\x01\x1a\x00\x00\x11\x11\xcd\x00\xf8\xab\x05\x66\x89\x5a\x30\xca\x00\x0a\x64\xd0\x6a\x00\x30\x80\xfc\x33\xf5\x8d\xdd\x27\xf6\xbe\x61\x02\x77\x0b\x31\x06\xf7\x31\x59\x0a\x77\x2c\x89\x0d\x37\x2c\x01\x11\x37\x16\xa5\x12\xb7\xfe\x64\x14\xb7\xd7\x5c\x15\x37\xb8\xa4\x16\xf7\x75\x4c\x16\x77\x48\x24\x16\xf7\x08\x0c\x14\xb7\x27\xc6\x12\x36\xbc\x8a\x1f\x36\x2c\x2b\x19\xf6\x79\x4b\x12\xf6\xc2\xfb\x0a\xf6\xfd\xdb\x02\x77\x0e\x7b\x23\xf7\x0b\x93\x27\xb6\xf0\x83\x38\x77\xdb\xce\x31\x77\x88\xb6\x36\x77\x1d\x5e\x3a\xb7\x6e\x68\x3b\xf1\x5f\xfd\x21\x25\x08\x0f\x02\x00\xcc\xcd\xc7\xcf\x04\x00\x00\xf0\xbf\x01\x04\x70\x00\xfe\x1f\x30\x40\xf8\x3f\xf0\x3f\x03\x2c\xb1\x3f\xff\x37\xf2\x0f\xfe\x36\x30\x00\xf9\x37\x77\x35\xf3\x0f\x77\x67\x9f\x09\xf7\x75\x8f\x04\xf7\x66\x37\x00\x77\x64\x93\x22\xf7\x58\x5b\x25\xf7\x44\xdb\x26\xf7\x3c\x5b\x28\x77\x2f\xb3\x29\x77\x24\xd3\x29\x77\x08\x7b\x2b\xb6\xf1\xa3\x36\xb6\xb6\x63\x36\xb6\x97\xe3\x36\xb6\x6a\x13\x34\xb6\x12\xff\x33\xf6\xe9\x0e\x31\xb5\x53\x1f\x3e\xb5\xdb\xfe\x33\xb4\xcb\x7e\x36\xb2\x8f\x03\x2d\xf4\x47\xbc\x10\xf5\x4f\x9c\x12\xb5\x5d\x9c\x1c\xf6\x05\xac\x10\x35\x88\xba\x1d";
        let frame = super::Frame::from_slice(bytes).unwrap();

        assert_eq!(frame.rssi, -51);
    }
}
