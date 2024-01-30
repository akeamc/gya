//! CSI processing.

use ndarray::Array1;
use num_complex::Complex;

use crate::{frame::Frame, params::ChanSpec};

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

/// Calculate the angle of arrival (AoA) of a Wi-Fi frame. In radians, of course.
pub fn aoa(csi: &WifiCsi) -> Option<f64> {
    // let m = ndarray::arr2(&[
    //     csi.frames[0][0].clone()?,
    //     csi.frames[1][0].clone()?,
    //     csi.frames[2][0].clone()?,
    // ]);

    let m = ndarray::stack![
        ndarray::Axis(0),
        csi.frames[0][0].clone()?,
        csi.frames[1][0].clone()?,
        csi.frames[2][0].clone()?
    ];

    let angles = m.map(|z| z.arg());

    println!("{}", angles);

    todo!();
}
