use std::{path::PathBuf, pin::pin, sync::Arc};

use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use atomic_counter::{AtomicCounter, RelaxedCounter};
use clap::{Args, Parser, Subcommand};
use csi::{
    ieee80211::{subcarrier_type_80mhz, Band, Bandwidth, SubcarrierType},
    params::{ChanSpec, Cores, Params, SpatialStreams},
    proc::{aoa, WifiCsi},
};
use egui::load::SizedTexture;
use futures::{StreamExt, TryStreamExt};
use macaddr::MacAddr6;
use num_complex::{Complex, ComplexFloat};
use rt_ac86u::RtAc86u;
use sensor::proc::wifi_csi;
use tokio::sync::mpsc;

const DIAGRAMS: usize = 16;
const MACBOOK: MacAddr6 = MacAddr6::new(0x50, 0xED, 0x3C, 0x2E, 0x04, 0x00);

type Values = WifiCsi;

struct Waterfall {
    image: image::RgbaImage,
    pos: u32,
    prev_pos: u32,
}

impl Waterfall {
    fn new(width: u32, height: u32) -> Self {
        Self {
            image: image::RgbaImage::new(width, height),
            pos: 0,
            prev_pos: 0,
        }
    }

    fn add(&mut self, values: &[&[Complex<f64>]]) {
        let grad = colorgrad::sinebow();

        for (i, csi) in values.iter().enumerate() {
            let max = csi
                .iter()
                .map(|z| z.norm_sqr())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                .sqrt();

            for (j, v) in csi.iter().enumerate() {
                let mut c = grad.at((v.arg() + std::f64::consts::PI) / std::f64::consts::TAU);
                let scale = v.norm() / max;
                c.r *= scale;
                c.g *= scale;
                c.b *= scale;

                self.image
                    .put_pixel(self.pos, (i * 256 + j) as _, image::Rgba(c.to_rgba8()));
            }

            // border
            self.image
                .put_pixel(self.pos, (i * 256) as _, image::Rgba([255, 255, 255, 255]));
        }

        self.prev_pos = self.pos;
        self.pos += 1;
        if self.pos == self.image.width() {
            self.pos = 0;
        }
    }
}

struct App {
    _rt: tokio::runtime::Runtime,
    rx: mpsc::Receiver<Values>,
    cnt: Arc<RelaxedCounter>,
    texture: Option<egui::TextureHandle>,
    waterfall: Waterfall,
}

impl App {
    fn new(cli: Cli) -> Self {
        let (tx, rx) = mpsc::channel(1000);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let cnt = Arc::new(RelaxedCounter::new(0));

        rt.spawn({
            let cnt = cnt.clone();
            async move {
                data(cli, tx, &cnt).await.unwrap();
            }
        });

        Self {
            _rt: rt,
            rx,
            cnt,
            texture: None,
            waterfall: Waterfall::new(6000, 256 * DIAGRAMS as u32),
        }
    }
}

fn egui_image(image: &image::RgbaImage) -> egui::ColorImage {
    egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as _, image.height() as _],
        image.as_flat_samples().as_slice(),
    )
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let tex = self.texture.get_or_insert_with(|| {
        //     ctx.load_texture("img", egui_image(&self.waterfall.image), Default::default())
        // });

        // while let Ok(csi) = self.rx.try_recv() {
        //     dbg!(aoa(&csi));
        //     let frames = csi
        //         .frames()
        //         .iter()
        //         .flatten()
        //         .filter_map(Option::as_ref)
        //         .collect::<Vec<_>>();
        //     if frames.len() != DIAGRAMS {
        //         continue;
        //     }
        //     self.waterfall.add(&frames);
        // }

        // tex.set(egui_image(&self.waterfall.image), Default::default());

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.label(format!("{} packets", self.cnt.get()));
        //     ui.add(egui::Image::new(SizedTexture::from(&*tex)).shrink_to_fit());
        // });
        // ctx.request_repaint();
    }
}

async fn data(cli: Cli, tx: mpsc::Sender<Values>, cnt: &RelaxedCounter) -> anyhow::Result<()> {
    tracing::info!("connecting");

    let client = Client::connect(
        ("192.168.0.84", 22),
        "admin",
        AuthMethod::with_password("password"),
        ServerCheckMethod::NoCheck,
    )
    .await?;
    let client = RtAc86u::new(client);

    tracing::info!("connected!");

    match cli.command {
        Command::Run(args) => {
            let params = Params {
                chan_spec: ChanSpec::new(args.channel, Band::Band5G, Bandwidth::Bw80).unwrap(),
                csi_collect: true,
                cores: Cores::all(),
                spatial_streams: SpatialStreams::all(),
                first_pkt_byte: None,
                mac_addrs: vec![MACBOOK],
                delay_us: 0,
            };

            client.configure(params, args.rmmod).await?;

            if let Some(path) = args.output {
                let mut file = tokio::fs::File::create(path).await?;
                tokio::io::copy(&mut client.tcpdump().await?, &mut file).await?;
            } else {
                let mut stream = pin!(wifi_csi(client.tcpdump().await?));

                while let Some(group) = stream.try_next().await? {
                    tx.send(group).await.unwrap();

                    cnt.inc();
                }
            }
        }
        Command::Reboot => {
            client.exec("/sbin/reboot").await?;

            std::process::exit(0);
        }
        _ => unimplemented!(),
    }

    Ok(())
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run(RunArgs),
    Reboot,
    Aoa {
        #[clap(short, long)]
        input: PathBuf,
    },
}

#[derive(Debug, Args)]
struct RunArgs {
    /// Channel to use
    #[clap(short, long, default_value = "100")]
    channel: u8,
    /// Remove and reinsert the dhd kernel module
    #[clap(short, long, default_value = "false")]
    rmmod: bool,
    /// PCAP output file
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn plot(csi: &WifiCsi) -> anyhow::Result<()> {
    use plotters::prelude::*;

    let root = SVGBackend::new("plot.svg", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("CSI", ("sans-serif", (5).percent_height()))
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
        .margin((1).percent())
        .build_cartesian_2d(
            // (-s..s),
            // -16i32..16,
            // (0u32..u32::MAX)
            //     .log_scale()
            // -s..s,
            -128i32..128,
            0f64..4.,
        )?;

    chart
        .configure_mesh()
        .x_desc("Subcarrier")
        .y_desc("Amplitude")
        .draw()?;

    for (idx, data) in csi
        .frames()
        .iter()
        .flat_map(|n| n.iter())
        .filter_map(Option::as_ref)
        .enumerate()
    {
        // if [0, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15].contains(&idx) {
        if [0, 2, 3, 6, 7, 10, 11, 14, 15].contains(&idx) {
            // continue;
        }

        // let data = data.iter().enumerate().map(|(row, value)| {
        //     if subcarrier_type_80mhz((row as i16 - 128) as i8) == SubcarrierType::Zero {
        //         use num_traits::Zero;
        //         Complex::zero()
        //     } else {
        //         *value
        //     }
        // });

        let scale = 10f64.powf(csi.rssi as f64 / 20.) / data.mapv(Complex::abs).sum().sqrt() / 100.;

        let color = Palette99::pick(idx).mix(0.9);
        chart
            .draw_series(LineSeries::new(
                data.iter().enumerate().map(
                    |(x, z)| (x as i32 - 128, scale * z.abs()), // |(_, z)| (z.im as i32, z.re as i32)
                                                                // |(x, z)| (z.re.log(100.), z.im.log(100.))
                ),
                color.stroke_width(3),
            ))?
            .label(idx.to_string())
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
    }

    chart.configure_series_labels().border_style(BLACK).draw()?;

    root.present()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Command::Aoa { input } = &cli.command {
        tokio::runtime::Builder::new_multi_thread()
            .build()?
            .block_on(async {
                let file = tokio::fs::File::open(input).await.unwrap();
                let mut frames = pin!(wifi_csi(file));
                // let mut frames = frames.skip(100);

                let frame = frames.try_next().await.unwrap().unwrap();

                plot(&frame).unwrap();

                while let Some(frame) = frames.try_next().await.unwrap() {
                    if let Some(aoa) = aoa(&frame) {
                        println!("{}", aoa);
                    }
                }
            })
    } else {
        let app = App::new(cli);

        eframe::run_native("SENSOR", Default::default(), Box::new(|_| Box::new(app))).unwrap();
    }

    Ok(())
}
