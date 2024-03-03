fn phase_shift_to_angle(
    phase: &ArrayBase<impl Data<Elem = f64>, Dim<[usize; 1]>>,
    wavelength: &ArrayBase<impl Data<Elem = f64>, Dim<[usize; 1]>>,
    antenna_distance: f64,
) -> Array1<f64> {
    (phase * wavelength / (std::f64::consts::TAU * antenna_distance)).mapv(|x| x.asin())
}

pub fn aoa(csi: &WifiCsi, d: f64) -> Option<[Array1<f64>; 2]> {
    let right = csi.get(1, 0)?.map(|z| z.arg());
    let center = csi.get(3, 0)?.map(|z| z.arg());
    let left = csi.get(0, 0)?.map(|z| z.arg());

    let wavelengths = subcarrier_lambda(
      csi.chan_spec.center(),
      csi.chan_spec.bandwidth(),
    );

    Some([
        phase_shift_to_angle(&(center - &right), &wavelengths, d),
        phase_shift_to_angle(&(left - &right), &wavelengths, 2. * d),
    ])
}