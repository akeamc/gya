//! CSI processing.

use ndarray::{Array1, ArrayBase, Data, Dim};

use num_complex::{Complex, ComplexFloat};
use rustfft::Fft;
use uom::si::f64::Time;

use crate::{
    frame::Frame,
    ieee80211::{subcarrier_lambda, Bandwidth},
    params::ChanSpec,
};

/// CSI information for a single Wi-Fi frame.
///
/// Each Wi-Fi frame generates multiple CSI frames, one for each
/// spatial stream. This struct contains all CSI frames for a single
/// Wi-Fi frame.
#[derive(Debug, Clone)]
pub struct WifiCsi {
    frames: [[Option<Array1<Complex<f64>>>; 4]; 4],
    /// See the documentation for [`ChanSpec`].
    pub chan_spec: ChanSpec,
    /// Received signal strength indicator (dBi).
    pub rssi: i8,
}

impl WifiCsi {
    pub fn frames(&self) -> &[[Option<Array1<Complex<f64>>>; 4]; 4] {
        &self.frames
    }

    pub fn get(&self, core: usize, spatial: usize) -> Option<&Array1<Complex<f64>>> {
        self.frames[core][spatial].as_ref()
    }
}

/// Groups CSI frames by Wi-Fi frame.
///
/// ```
/// # let mut frames = std::iter::empty();
/// let mut groups = vec![];
/// let mut grouper = csi::proc::FrameGrouper::new();
///
/// for frame in frames {
///     if let Some(group) = grouper.add(frame) {
///         groups.push(group);
///     }
/// }
///
/// if let Some(group) = grouper.take() {
///     groups.push(group);
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct FrameGrouper(Option<(WifiCsi, u16)>);

impl FrameGrouper {
    /// Creates a new `FrameGrouper`.
    pub fn new() -> Self {
        Self::default()
    }

    fn seq_cnt(&self) -> Option<u16> {
        self.0.as_ref().map(|(_, seq_cnt)| *seq_cnt)
    }

    /// Adds a CSI frame to the grouper.
    ///
    /// Returns `Some` if the grouper is full and should be yielded.
    pub fn add(&mut self, frame: Frame) -> Option<WifiCsi> {
        let ret = if Some(frame.seq_cnt) != self.seq_cnt() {
            let group = self.take();
            self.0 = Some((
                WifiCsi {
                    frames: [
                        [None, None, None, None],
                        [None, None, None, None],
                        [None, None, None, None],
                        [None, None, None, None],
                    ],
                    chan_spec: frame.chan_spec,
                    rssi: frame.rssi,
                },
                frame.seq_cnt,
            ));
            group
        } else {
            None
        };

        let (group, _) = self.0.as_mut().unwrap();
        let core = frame.core as usize;
        let spatial = frame.spatial as usize;
        group.frames[core][spatial] = Some(frame.csi);

        ret
    }

    /// Takes the current group, provided it is not empty.
    ///
    /// To ensure that the last group is yielded, this method should be
    /// called after the stream of CSI frames has ended.
    pub fn take(&mut self) -> Option<WifiCsi> {
        let (csi, _) = self.0.take()?;
        if csi.frames.iter().flatten().all(Option::is_none) {
            return None;
        }

        Some(csi)
    }
}

fn phase_shift_to_angle(
    phase: &ArrayBase<impl Data<Elem = f64>, Dim<[usize; 1]>>,
    wavelength: &ArrayBase<impl Data<Elem = f64>, Dim<[usize; 1]>>,
    antenna_distance: f64,
) -> Array1<f64> {
    (phase * wavelength / (std::f64::consts::TAU * antenna_distance)).mapv(|x| x.asin())
}

/// Calculate the angle of arrival (AoA) of a Wi-Fi frame. In radians, of course.
///
/// Antenna indices:
///
/// <img src="https://user-images.githubusercontent.com/57238941/115536641-50408100-a29a-11eb-9ee7-866e654e6969.png" width="200" />
///
/// (0, 3, 1) from left to right.
pub fn aoa(csi: &WifiCsi, d: f64) -> Option<[Array1<f64>; 2]> {
    // https://user-images.githubusercontent.com/57238941/115536641-50408100-a29a-11eb-9ee7-866e654e6969.png
    const LEFT_ANTENNA: usize = 0;
    const CENTER_ANTENNA: usize = 3;
    const RIGHT_ANTENNA: usize = 1;

    let a0 = csi.get(RIGHT_ANTENNA, 0)?;
    let a1 = csi.get(CENTER_ANTENNA, 0)?;
    let a2 = csi.get(LEFT_ANTENNA, 0)?;

    let wavelengths = subcarrier_lambda(csi.chan_spec.center(), csi.chan_spec.bandwidth());

    Some([
        phase_shift_to_angle(&(a1 / a0).map(|z| z.arg()), &wavelengths, d),
        phase_shift_to_angle(&(a2 / a0).map(|z| z.arg()), &wavelengths, 2. * d),
    ])
}

fn tof_in_place(csi: &mut [Complex<f64>], bandwidth: Bandwidth) -> Time {
    rustfft::algorithm::Radix4::new(csi.len(), rustfft::FftDirection::Inverse).process(csi);
    let half = &csi[..csi.len() / 2];
    let (peak_idx, _) = half
        .iter()
        .map(|z| z.abs())
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap();

    peak_idx as f64 / bandwidth.freq()
}

pub fn tof(csi: &WifiCsi) -> Vec<Time> {
    let mut tofs = vec![];

    for core in 0..4 {
        if let Some(buf) = csi.get(core, 0) {
            tofs.push(tof_in_place(&mut buf.to_vec(), csi.chan_spec.bandwidth()));
        }
    }

    tofs
}
