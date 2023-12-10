use std::fmt::Display;

use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use macaddr::MacAddr6;

pub enum Band {
    Band2G,
    Band5G,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bandwidth {
    Bw20,
    Bw40,
    Bw80,
    Bw160,
}

impl Bandwidth {
    pub const fn as_mhz(&self) -> u8 {
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
        let lowest = center - (bw.as_mhz() - 20) / 10;

        if (ctl_ch - lowest) % 4 != 0 {
            continue; // center channel must be a multiple of 4
        }

        let sb = (ctl_ch - lowest) / 4;

        if sb >= bw.as_mhz() / 20 {
            continue; // ctl_ch too high for this center channel
        }

        return (*center, sb);
    }

    panic!("invalid channel");
}

#[derive(Debug, Clone, Copy)]
/// A chanspec holds the channel number, band, bandwidth and control sideband.
pub struct ChanSpec(pub(crate) u16);

impl ChanSpec {
    const CENTER_SHIFT: u8 = 0;
    const SIDEBAND_SHIFT: u8 = 8;
    const BW_MASK: u16 = 0x3800;

    pub const fn bandwidth(&self) -> Bandwidth {
        match self.0 & Self::BW_MASK {
            0x1000 => Bandwidth::Bw20,
            0x1800 => Bandwidth::Bw40,
            0x2000 => Bandwidth::Bw80,
            0x2800 => Bandwidth::Bw160,
            _ => unreachable!(),
        }
    }

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

pub struct CsiParams {
    pub chan_spec: ChanSpec,
    pub csi_collect: bool,
    pub core_mask: u8,
    pub nss_mask: u8,
    pub first_pkt_byte: Option<u8>,
    /// Source MAC addresses to filter on. Maximum length is 4.
    pub mac_addrs: Vec<MacAddr6>,
    pub delay: u16,
}

impl CsiParams {
    pub fn to_bytes(&self) -> [u8; 34] {
        let mut out = [0u8; 34];

        out[0..2].copy_from_slice(&self.chan_spec.to_inner().to_le_bytes());

        if self.csi_collect {
            out[2] = 1;
        }

        out[3] = (self.core_mask & 0x0f) | (self.nss_mask << 4);

        if let Some(first_pkt_byte) = self.first_pkt_byte {
            out[4] = 1;
            out[5] = first_pkt_byte;
        }

        out[6..8].copy_from_slice(&(self.mac_addrs.len() as u16).to_le_bytes());
        for (i, mac) in self.mac_addrs.iter().enumerate() {
            out[8 + i * 6..8 + (i + 1) * 6].copy_from_slice(mac.as_bytes());
        }

        out[32..34].copy_from_slice(&self.delay.to_le_bytes());

        out
    }
}

impl Display for CsiParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Base64Display::new(&self.to_bytes(), &STANDARD).fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Band, Bandwidth, ChanSpec, CsiParams};

    #[test]
    fn it_works() {
        let params = CsiParams {
            chan_spec: ChanSpec::new(36, Band::Band5G, Bandwidth::Bw40).unwrap(),
            csi_collect: true,
            core_mask: 0x5,
            nss_mask: 0x7,
            first_pkt_byte: None,
            mac_addrs: vec![],
            delay: 0,
        };

        assert_eq!(
            // (&["makecsiparams", "-c", "36/40", "-C", "0x5", "-N", "0x7"]),
            params.to_string(),
            "JtgBdQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
        );
    }

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
