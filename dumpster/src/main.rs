use std::f32::consts::PI;

use image::{ImageBuffer, Rgb, RgbImage};
use num_complex::Complex;
use palette::{rgb::Rgba, Hsl, IntoColor};

pub mod csi;

// fn unpack_float(buf: &[u8], csi: &mut Vec<Complex<f32>>, nfft: usize, M: usize, E: usize, endian: char) {
//     let nbits = 10;
//     let autoscale = 1;
//     let e_p = 1 << (E - 1);
//     let e_shift = 1;
//     let e_zero = -(M as i32);
//     let mut maxbit = -e_p;
//     let k_tof_unpack_sgn_mask = 1 << 31;
//     let ri_mask = (1 << (M - 1)) - 1;
//     let e_mask = (1 << E) - 1;
//     let sgnr_mask = 1 << (E + 2 * M - 1);
//     let sgni_mask = sgnr_mask >> M;
//     let mut he = vec![0; 256];
//     let mut hout = vec![0; 512];

//     for i in 0..nfft {
//         let h = u32::from_le_bytes(buf[4 * i..4 * i + 4].try_into().unwrap());

//         let v_real = (h >> (E + M)) & ri_mask;
//         let v_imag = (h >> E) & ri_mask;
//         let mut e = h & e_mask;
//         if e >= e_p {
//             e -= e_p << 1;
//         }
//         he[i] = e;
//         let x = v_real | v_imag;

//         if autoscale != 0 && x != 0 {
//             let mut m = 0xffff0000;
//             let mut b = 0xffff;
//             let mut s = 16;
//             while s > 0 {
//                 if x & m != 0 {
//                     e += s;
//                     x >>= s;
//                 }
//                 s >>= 1;
//                 m = (m >> s) & b;
//                 b >>= s;
//             }
//             if e > maxbit {
//                 maxbit = e;
//             }
//         }
//         if h & sgnr_mask != 0 {
//             hout[i << 1] |= k_tof_unpack_sgn_mask;
//         }
//         if h & sgni_mask != 0 {
//             hout[(i << 1) + 1] |= k_tof_unpack_sgn_mask;
//         }
//         hout[i << 1] |= v_real;
//         hout[(i << 1) + 1] |= v_imag;
//     }

//     let shft = nbits - maxbit;
//     for i in 0..nfft * 2 {
//         let e = he[i >> e_shift] + shft;
//         let mut sgn: i32 = 1;
//         if hout[i] & k_tof_unpack_sgn_mask != 0 {
//             sgn = -1;
//             hout[i] &= !k_tof_unpack_sgn_mask;
//         }
//         if e < e_zero {
//             hout[i] = 0;
//         } else if e < 0 {
//             let e = -e;
//             hout[i] = hout[i] >> e;
//         } else {
//             hout[i] = hout[i] << e;
//         }
//         hout[i] *= sgn;
//     }

//     for i in 0..nfft {
//         csi[i] = Complex::new(hout[i * 2] as f32, hout[i * 2 + 1] as f32);
//     }
// }

/// Plot a complex vector as an image.
fn plot_complex(nums: &[Complex<f32>], width: u32, height: u32) -> RgbImage {
    let mut image = RgbImage::new(width, height);

    let max_norm = nums
        .iter()
        .map(|c| c.norm())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    for (i, c) in nums.iter().enumerate() {
        let x = i as u32 % width;
        let y = i as u32 / width;

        let l = (1. - c.norm() / max_norm) / 2.;
        // let l = 0.5 * c.norm() / max_norm;
        // let l = 0.5;

        let color: Rgba = Hsl::new((c.arg() + PI).to_degrees(), 1.0, l).into_color();
        // let color: Rgba = Rgba::new(l, l, l, 1.0);
        let pixel = Rgb([
            (color.red * 255.0) as u8,
            (color.green * 255.0) as u8,
            (color.blue * 255.0) as u8,
        ]);

        image.put_pixel(x, y, pixel);
    }

    image
}

fn main() -> anyhow::Result<()> {
    let mut capture = pcap::Capture::from_file("trace.pcap").unwrap();

    let mut cnt = 0;
    let mut csi = Vec::new();

    while cnt < 4000 {
        let packet = match capture.next_packet() {
            Ok(packet) => packet,
            Err(pcap::Error::NoMorePackets) => break,
            Err(err) => panic!("error while reading packet: {}", err),
        };

        let frame = csi::Frame::from_slice(&packet.data).unwrap();

        // bcm4366c0 returns floating point CSI

        // dbg!(csi.len());

        csi.extend_from_slice(&frame.csi_values);

        // for (i, c) in csi.iter().enumerate() {
        //     let x = cnt;
        //     let y = i as u32;

        //     println!("{y}");

        //     let color: Rgba = Hsl::new(c.arg() / TAU, 1.0, 0.5).into_color();
        //     let pixel = Rgb([
        //         (color.red * 255.0) as u8,
        //         (color.green * 255.0) as u8,
        //         (color.blue * 255.0) as u8,
        //     ]);

        //     image.put_pixel(x, y, pixel);
        // }

        cnt += 1;
    }

    let image = plot_complex(&csi, cnt, csi.len() as u32 / cnt);

    println!("{} packets", cnt);

    image.save("csi.png")?;

    Ok(())
}
