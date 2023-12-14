//! CSI collection parameters passed to the firmware.

use std::fmt::Display;

use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use macaddr::MacAddr6;

/// Band.
pub enum Band {
    /// 2.4 GHz.
    Band2G,
    /// 5 GHz.
    Band5G,
}

/// Bandwidth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

fn bands(ctl_ch: u8, bw: Bandwidth) -> (u8, u8) {
    let channels: &[u8] = match bw {
        Bandwidth::Bw20 => return (ctl_ch, 0), // trivial case
        Bandwidth::Bw40 => &[38, 46, 54, 62, 102, 110, 118, 126, 134, 142, 151, 159],
        Bandwidth::Bw80 => &[42, 58, 106, 122, 138, 155],
        Bandwidth::Bw160 => &[50, 114],
    };

    for center in channels {
        let lowest = center - (bw.mhz() - 20) / 10;

        if (ctl_ch - lowest) % 4 != 0 {
            continue; // center channel must be a multiple of 4
        }

        let sb = (ctl_ch - lowest) / 4;

        if sb >= bw.mhz() / 20 {
            continue; // ctl_ch too high for this center channel
        }

        return (*center, sb);
    }

    panic!("invalid channel");
}

/// A chanspec holds the channel number, band, bandwidth and control sideband.
#[derive(Debug, Clone, Copy)]
pub struct ChanSpec(pub(crate) u16);

impl ChanSpec {
    const CENTER_SHIFT: u8 = 0;
    const SIDEBAND_SHIFT: u8 = 8;

    /// Extract the bandwidth.
    ///
    /// # Panics
    ///
    /// If the bandwidth is unknown (not a variant of [`Bandwidth`]),
    /// this function will panic.
    pub const fn bandwidth(&self) -> Bandwidth {
        // #define WL_CHANSPEC_BW_MASK             0x3800
        // #define WL_CHANSPEC_BW_SHIFT            11
        // #define WL_CHANSPEC_BW_5                0x0000
        // #define WL_CHANSPEC_BW_10               0x0800
        // #define WL_CHANSPEC_BW_20               0x1000
        // #define WL_CHANSPEC_BW_40               0x1800
        // #define WL_CHANSPEC_BW_80               0x2000
        // #define WL_CHANSPEC_BW_160              0x2800
        // #define WL_CHANSPEC_BW_8080             0x3000
        match self.0 & 0x3800 {
            0x1000 => Bandwidth::Bw20,
            0x1800 => Bandwidth::Bw40,
            0x2000 => Bandwidth::Bw80,
            0x2800 => Bandwidth::Bw160,
            _ => panic!("unknown bandwidth"),
        }
    }

    /// Center channel.
    pub const fn center(&self) -> u8 {
        ((self.0 >> Self::CENTER_SHIFT) & 0xff) as u8
    }

    /// Construct a new chanspec.
    pub fn new(channel: u8, band: Band, bandwidth: Bandwidth) -> Result<Self, ()> {
        let (center, sideband) = bands(channel, bandwidth);

        let mut out = 0;

        out |= (center as u16) << Self::CENTER_SHIFT;
        out |= match band {
            Band::Band2G => 0,
            Band::Band5G => 0xc000,
        };
        out |= (sideband as u16) << Self::SIDEBAND_SHIFT;
        out |= match bandwidth {
            Bandwidth::Bw20 => 0x1000,
            Bandwidth::Bw40 => 0x1800,
            Bandwidth::Bw80 => 0x2000,
            Bandwidth::Bw160 => 0x2800,
        };

        Ok(Self(out))
    }

    const fn to_inner(&self) -> u16 {
        self.0
    }
}

bitflags::bitflags! {
    /// Core filter.
    #[derive(Debug, Clone, Copy)]
    pub struct Cores: u8 {
        /// Enable core 0.
        const CORE0 = 0b0001;
        /// Enable core 1.
        const CORE1 = 0b0010;
        /// Enable core 2.
        const CORE2 = 0b0100;
        /// Enable core 3.
        const CORE3 = 0b1000;
    }

    /// Spatial stream filter.
    #[derive(Debug, Clone, Copy)]
    pub struct SpatialStreams: u8 {
        /// Enable spatial stream 0.
        const S0 = 0b0001;
        /// Enable spatial stream 1.
        const S1 = 0b0010;
        /// Enable spatial stream 2.
        const S2 = 0b0100;
        /// Enable spatial stream 3.
        const S3 = 0b1000;
    }
}

/// Default delay for the given cores and spatial streams.
/// See [`Params::delay_us`].
pub const fn default_delay_us(cores: Cores, spatial_streams: SpatialStreams) -> u16 {
    // int csi_to_capture = countbit (nssmask) * countbit (coremask);
    // if (csi_to_capture >= 12) {
    //    delay = DEFAULT_DELAY_US; // (50)
    //    st16le(delay, &p.delay);
    // }

    let n_cores = cores.bits().count_ones();
    let n_spatial_streams = spatial_streams.bits().count_ones();

    if n_cores * n_spatial_streams >= 12 {
        50
    } else {
        0
    }
}

/// CSI collection parameters used by
/// [nexutil](https://github.com/seemoo-lab/nexmon/blob/ae8addba003ceb68a4217c014242d5c747eeaf36/utilities/nexutil/README.md).
///
/// ```
/// # use csi::params::{Cores, SpatialStreams, Params, ChanSpec, Band, Bandwidth};
/// let params = Params {
///     chan_spec: ChanSpec::new(36, Band::Band5G, Bandwidth::Bw40).unwrap(),
///     csi_collect: true,
///     cores: Cores::CORE0 | Cores::CORE2,
///     spatial_streams: SpatialStreams::S0 | SpatialStreams::S1 | SpatialStreams::S2,
///     first_pkt_byte: None,
///     mac_addrs: vec![],
///     delay_us: 0,
/// };
///
/// assert_eq!(
///     params.to_string(),
///     // makecsiparams -c 36/40 -C 0x5 -N 0x7
///     "JtgBdQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
/// );
/// ```
pub struct Params {
    /// Channel specification. See [`ChanSpec`].
    pub chan_spec: ChanSpec,
    /// Whether to collect CSI.
    pub csi_collect: bool,
    /// Which cores to collect on.
    pub cores: Cores,
    /// Spatial streams to collect.
    pub spatial_streams: SpatialStreams,
    /// First packet byte to filter on.
    pub first_pkt_byte: Option<u8>,
    /// Source MAC addresses to filter on. Maximum length is 4.
    pub mac_addrs: Vec<MacAddr6>,
    /// Delay in microseconds after each CSI operation
    /// (really needed for 3x4, 4x3 and 4x4 configurations).
    pub delay_us: u16,
}

impl Params {
    /// Convert to a byte array that can be passed to nexutil. The format conforms to the
    /// following C struct (little endian):
    ///
    /// ```c
    /// struct csi_params {
    ///     uint16_t chanspec;            // chanspec to tune to
    ///     uint8_t  csi_collect;         // trigger csi collect (1: on, 0: off)
    ///     uint8_t  core_nss_mask;       // coremask and spatialstreammask./iperf -u -c 192.168.2.59 -i 1 -b 10M
    ///     uint8_t  use_pkt_filter;      // trigger first packet byte filter (1: on, 0: off)
    ///     uint8_t  first_pkt_byte;      // first packet byte to filter for
    ///     uint16_t n_mac_addr;          // number of mac addresses to filter for (0: off, 1-4: on,use src_mac_0-3)
    ///     uint16_t cmp_src_mac_0_0;     // filter src mac 0
    ///     uint16_t cmp_src_mac_0_1;
    ///     uint16_t cmp_src_mac_0_2;
    ///     uint16_t cmp_src_mac_1_0;     // filter src mac 1
    ///     uint16_t cmp_src_mac_1_1;
    ///     uint16_t cmp_src_mac_1_2;
    ///     uint16_t cmp_src_mac_2_0;     // filter src mac 2
    ///     uint16_t cmp_src_mac_2_1;
    ///     uint16_t cmp_src_mac_2_2;
    ///     uint16_t cmp_src_mac_3_0;     // filter src mac 3
    ///     uint16_t cmp_src_mac_3_1;
    ///     uint16_t cmp_src_mac_3_2;
    ///     uint16_t delay;
    /// };
    /// ```
    ///
    /// [GitHub source](https://github.com/seemoo-lab/nexmon_csi/blob/fdb25ef0e4e1402e968bb644d4914ad1a3d0a84d/utils/makecsiparams/makecsiparams.c#L44C8-L64)
    pub fn to_bytes(&self) -> [u8; 34] {
        let mut out = [0u8; 34];

        out[0..2].copy_from_slice(&self.chan_spec.to_inner().to_le_bytes());

        if self.csi_collect {
            out[2] = 1;
        }

        out[3] = (self.cores.bits() & 0x0f) | (self.spatial_streams.bits() << 4);

        if let Some(first_pkt_byte) = self.first_pkt_byte {
            out[4] = 1;
            out[5] = first_pkt_byte;
        }

        out[6..8].copy_from_slice(&(self.mac_addrs.len() as u16).to_le_bytes());
        for (i, mac) in self.mac_addrs.iter().enumerate() {
            out[8 + i * 6..8 + (i + 1) * 6].copy_from_slice(mac.as_bytes());
        }

        out[32..34].copy_from_slice(&self.delay_us.to_le_bytes());

        out
    }
}

impl Display for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Base64Display::new(&self.to_bytes(), &STANDARD).fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Band, Bandwidth, ChanSpec};

    #[test]
    fn chan_spec() {
        assert_eq!(
            ChanSpec::new(36, Band::Band5G, Bandwidth::Bw40)
                .unwrap()
                .to_inner(),
            0xd826
        );
    }
}
