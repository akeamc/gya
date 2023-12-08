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

/// A reported CSI frame.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Received signal strength indicator (dBi).
    pub rssi: i8,
    pub frame_control: u8,
    pub source_mac: MacAddr6,
    /// "The two byte sequence number of the Wi-Fi frame that triggered
    /// the collection of the CSI contained in this packet."
    pub seq_cnt: u16,
    pub config: u16,
    // core: u8,
    // spatial: u8,
    pub chan_spec: u16,
    pub chip: u16,
    pub csi_values: Vec<Complex<f32>>,
}

/// Channel bandwidth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bandwidth {
    /// 20 MHz
    Bw20,
    /// 40 MHz
    Bw40,
    /// 80 MHz
    Bw80,
    /// 160 MHz
    Bw160,
}

impl Bandwidth {
    const fn from_chan_spec(chan_spec: u16) -> Self {
        match chan_spec & 0b11 {
            0b00 => Self::Bw20,
            0b01 => Self::Bw40,
            0b10 => Self::Bw80,
            0b11 => Self::Bw160,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    NotEnoughBytes,
    NotANexmonPacket,
    MissingMagicBytes,
}

impl Frame {
    pub const fn bandwidth(&self) -> Bandwidth {
        Bandwidth::from_chan_spec(self.chan_spec)
    }

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

        let config = u16::from_le_bytes([b[54], b[55]]);
        // let frame_no = u16::from_le_bytes([b[11], b[12]]);
        // let core = b[11] & 0b111;
        // let spatial = (b[11] >> 3) & 0b111;
        // let spatial = ((b[10] | b[11]) >> 3) & 0x7;

        let chan_spec = u16::from_le_bytes([b[56], b[57]]);
        let chip = u16::from_le_bytes([b[58], b[59]]);

        assert_eq!(chip, 106, "chip is not BCM4366c0");

        let csi_values = unpack_csi(Bandwidth::from_chan_spec(chan_spec), &b[60..]);

        Ok(Self {
            rssi: b[44] as i8,
            frame_control: b[45],
            source_mac: MacAddr6::new(b[46], b[47], b[48], b[49], b[50], b[51]),
            seq_cnt: u16::from_le_bytes([b[52], b[53]]),
            config,
            // core,
            // spatial,
            chan_spec,
            chip,
            csi_values,
        })
    }
}

/// Unpacks a complex value from the given 32-bit integer.
///
/// The BCM4366c0 encodes each CSI value in 32 bits:
///
/// ```text
/// | sign(1) | re(12) | sign(1) | im(12) | exp(6) |
/// ```
///
/// [GitHub source](https://github.com/seemoo-lab/nexmon_csi/blob/fdb25ef0e4e1402e968bb644d4914ad1a3d0a84d/src/csi_extractor.c#L244-L246)
///
/// ```
/// # use dumpster::csi::unpack_complex;
/// let z = unpack_complex(0b1_000000000001_0_000000000010_000011);
///
/// assert_eq!(z.re, -8.0);
/// assert_eq!(z.im, 16.0);
/// ```
pub fn unpack_complex(i: u32) -> Complex<f32> {
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

    const MAN_MASK: u32 = 0b111111111111; // 12 bits
    const E_MASK: u32 = 0b111111; // 6 bits

    let mut exp = (i & E_MASK) as i32;
    if exp >= 1 << 5 {
        // exponent is negative
        exp -= 1 << 6;
    }

    let mut re = ((i >> 19) & MAN_MASK) as i32;
    if i & 1 << 31 != 0 {
        // sign bit for real part is set
        re = -re;
    }

    let mut im = ((i >> 6) & MAN_MASK) as i32;
    if i & 1 << 18 != 0 {
        // sign bit for imaginary part is set
        im = -im;
    }

    // construct float from mantissas and exponent
    let re = (re as f32) * (2f32.powi(exp));
    let im = (im as f32) * (2f32.powi(exp));

    Complex::new(re, im)
}

/// Unpacks the CSI values from the given buffer.
///
/// The complex values are expected to be encoded in 32 bits each, with
/// 11 bits for each mantissa and 6 bits for the exponent.
///
/// # Panics
///
/// Panics if the buffer is too short (`< 3.2 * <bandwidth in MHz>`).
fn unpack_csi(bw: Bandwidth, b: &[u8]) -> Vec<Complex<f32>> {
    // nsub = bw * 3.2
    let nsub = match bw {
        Bandwidth::Bw20 => 64,
        Bandwidth::Bw40 => 128,
        Bandwidth::Bw80 => 256,
        Bandwidth::Bw160 => 512,
    };

    assert!(b.len() >= nsub * 4, "not enough data");

    let mut csi = b
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .map(unpack_complex)
        .collect::<Vec<_>>();

    let n = csi.len() / 2;
    csi.rotate_right(n);

    csi
}
