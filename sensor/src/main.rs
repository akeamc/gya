use std::{path::PathBuf, pin::pin, sync::Arc, time::Instant};

use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use atomic_counter::{AtomicCounter, RelaxedCounter};
use clap::{Args, Parser, Subcommand};
use csi::{
    ieee80211::{Band, Bandwidth},
    params::{ChanSpec, Cores, Params, SpatialStreams},
    proc::{aoa, WifiCsi},
};
use egui::Vec2;
use egui_plot::{Line, Plot, PlotPoints};
use futures::{Stream, TryStreamExt};
use macaddr::MacAddr6;

use ndhistogram::{axis::BinInterval, Histogram};
use num_complex::{Complex, ComplexFloat};
use rt_ac86u::RtAc86u;
use sensor::read::{read_wifi_csi, PcapSource};
use tokio::sync::mpsc;
use uom::si::f64::Length;

const MACBOOK: MacAddr6 = MacAddr6::new(0x50, 0xED, 0x3C, 0x2E, 0x04, 0x00);

const BANDWIDTH: Bandwidth = Bandwidth::Bw40;

type Values = WifiCsi;

struct App {
    _rt: tokio::runtime::Runtime,
    rx: mpsc::Receiver<Values>,
    cnt: Arc<RelaxedCounter>,
    data: Vec<WifiCsi>,
    last: bool,
    i: usize,
    prev_i: usize,
    core: usize,
    spatial: usize,
    antenna_spacing: f64,
    aoas: Vec<Vec<f64>>,
    distances: Vec<Length>,
}

impl App {
    fn new(args: RunArgs) -> Self {
        let (tx, rx) = mpsc::channel(10000);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let cnt = Arc::new(RelaxedCounter::new(0));

        rt.spawn({
            let cnt = cnt.clone();
            async move {
                run(args, tx, &cnt).await.unwrap();
            }
        });

        Self {
            _rt: rt,
            rx,
            cnt,
            data: vec![],
            last: true,
            i: 0,
            prev_i: 0,
            core: 0,
            spatial: 0,
            antenna_spacing: 0.088,
            aoas: vec![],
            distances: vec![],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.prev_i != self.i {
            self.prev_i = self.i;
            self.last = false;
        }

        while let Ok(csi) = self.rx.try_recv() {
            // if csi.frames().iter().flatten().all(Option::is_some) {
            // let tof = tof(&csi);
            // let avg = (tof[0] + tof[1] + tof[2] + tof[3]) / 4.;
            // self.distances.push(C * avg);

            if let Some(aoa) = aoa(&csi, self.antenna_spacing) {
                let aoa = aoa
                    .iter()
                    .flat_map(|x| x.iter())
                    .filter(|x| x.is_finite())
                    .copied()
                    .collect::<Vec<f64>>();
                // let n = aoa.len();
                // let (_, &mut median, _) = aoa.select_nth_unstable_by(n/ 2, |a, b| a.total_cmp(b));
                // self.aoas.push(median);
                self.aoas.push(aoa);
            }

            self.data.push(csi);
            // } else {
            // println!(":(")
            // }
        }

        // if let Some(ref csi) = self.csi {
        //     for core in 0..4 {
        //         let Some(data) = csi.get(core, 0) else { continue; };
        //     }
        // }

        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.add(egui::Checkbox::new(&mut self.last, "last"));
            ui.add(
                egui::Slider::new(&mut self.i, 0..=(self.data.len().saturating_sub(1))).text("i"),
            );
            ui.add(egui::Slider::new(&mut self.core, 0..=3).text("core"));
            ui.add(egui::Slider::new(&mut self.spatial, 0..=3).text("spatial"));
            ui.add(
                egui::Slider::new(&mut self.antenna_spacing, 0.01..=0.2).text("antenna spacing"),
            );
            ui.label(format!("{} packets", self.cnt.get()));
        });

        let data = if self.last {
            self.data.last()
        } else {
            self.data.get(self.i)
        };
        let Some(data) = data else {
            return;
        };

        let half_nsub = (data.chan_spec.bandwidth().nsub_pow2() / 2) as f64;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    const PLOT_SIZE: Vec2 = Vec2 { x: 500., y: 250. };

                    let core_n = data.get(self.core, self.spatial);

                    Plot::new("amplitude")
                        .auto_bounds(egui::Vec2b::FALSE)
                        .include_x(-half_nsub)
                        .include_x(half_nsub)
                        .include_y(0.)
                        .include_y(4.)
                        .min_size(PLOT_SIZE)
                        .show(ui, |plot_ui| {
                            let Some(core_n) = core_n else {
                                return;
                            };

                            let scale = 10f64.powf(data.rssi as f64 / 20.)
                                / core_n.mapv(Complex::abs).sum().sqrt()
                                / 100.;

                            let points = PlotPoints::from_iter(
                                core_n
                                    .indexed_iter()
                                    .map(|(i, z)| [i as f64 - half_nsub, scale * z.abs()]),
                            );
                            plot_ui.line(Line::new(points));
                        });

                    ui.end_row();

                    Plot::new("phase")
                        .auto_bounds(egui::Vec2b::FALSE)
                        .include_x(-half_nsub)
                        .include_x(half_nsub)
                        .include_y(-4.)
                        .include_y(4.)
                        .min_size(PLOT_SIZE)
                        .show(ui, |plot_ui| {
                            let Some(core_n) = core_n else {
                                return;
                            };

                            let core_0 = data.get(0, self.spatial).unwrap();

                            let raw = PlotPoints::from_iter(
                                core_n
                                    .indexed_iter()
                                    .map(|(i, z)| [i as f64 - half_nsub, (z / core_0[i]).arg()]),
                            );
                            plot_ui.line(Line::new(raw));
                            let asin =
                                PlotPoints::from_iter(core_n.indexed_iter().map(|(i, z)| {
                                    let z = z / core_0[i];
                                    [i as f64 - half_nsub, z.arg().sin()]
                                }));
                            plot_ui.line(Line::new(asin));
                        });

                    ui.end_row();

                    // Plot::new("aoas")
                    //     // .auto_bounds(egui::Vec2b::FALSE)
                    //     // .include_y(-4.)
                    //     // .include_y(4.)
                    //     .min_size(PLOT_SIZE)
                    //     .show(ui, |plot_ui| {
                    //         if self.aoas.is_empty() {
                    //             return;
                    //         }

                    //         for i in 0..self.aoas[0].len() {
                    //             let raw = PlotPoints::from_iter(
                    //                 self.aoas.iter().enumerate().filter_map(|(t, vec)| {
                    //                     let y = vec.get(i)?.to_degrees();
                    //                     if y.is_finite() {
                    //                         Some([t as f64, y])
                    //                     } else {
                    //                         None
                    //                     }
                    //                 }),
                    //             );
                    //             plot_ui.points(Points::new(raw));
                    //         }

                    //         // let raw = PlotPoints::from_iter(
                    //         //     self.aoas
                    //         //         .iter()
                    //         //         .enumerate()
                    //         //         .flat_map(|(i, vec)| vec.iter().filter(|v| v.is_finite()).map(move |angle| [i as f64, angle.to_degrees()])),
                    //         // );
                    //         // plot_ui.line(Line::new(raw));
                    //     });

                    // ui.end_row();

                    Plot::new("distances")
                        .min_size(PLOT_SIZE)
                        .show(ui, |plot_ui| {
                            let raw = PlotPoints::from_iter(
                                self.distances
                                    .iter()
                                    .enumerate()
                                    .map(|(i, d)| [i as f64, d.get::<uom::si::length::meter>()]),
                            );
                            let window = 20;
                            let moving_average = PlotPoints::from_iter(
                                self.distances.iter().enumerate().map(|(i, _)| {
                                    let start = (i as isize - window).max(0) as usize;
                                    let end = (i + window as usize).min(self.distances.len() - 1);
                                    let sum =
                                        self.distances[start..end].iter().copied().sum::<Length>();
                                    [
                                        i as f64,
                                        (sum / (end - start) as f64)
                                            .get::<uom::si::length::meter>(),
                                    ]
                                }),
                            );
                            plot_ui.line(Line::new(raw));
                            plot_ui.line(Line::new(moving_average));
                        });
                });

                // tex.set(egui_image(&self.waterfall.image), Default::default());
            });
        });
        ctx.request_repaint();
    }
}

async fn connect() -> anyhow::Result<RtAc86u> {
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

    Ok(client)
}

async fn run(args: RunArgs, tx: mpsc::Sender<Values>, cnt: &RelaxedCounter) -> anyhow::Result<()> {
    use ndhistogram::{axis::Uniform, ndhistogram};
    use plotters::prelude::*;

    let mut stream = pin!(get_input(&args).await?);
    // let mut stream = stream.take(args.samples.unwrap_or(usize::MAX));
    let mut writer = args.aoa.as_ref().map(csv::Writer::from_path).transpose()?;
    let t0 = Instant::now();
    let mut data = vec![];

    let low = -50.;
    let high = 50.;
    let n_bins = 100;

    while let Some(group) = stream.try_next().await? {
        if let Some(aoa) = aoa(&group, 0.088) {
            let mut hist = ndhistogram!(Uniform::new(n_bins, low, high));

            for v in aoa.iter().flat_map(|x| x.iter()) {
                if v.is_finite() {
                    hist.fill(&v.to_degrees());
                }
            }

            if let Some(ref mut writer) = writer.as_mut() {
                writer.write_field((Instant::now() - t0).as_secs_f64().to_string())?;

                for aoa in aoa.iter().flat_map(|x| x.iter()) {
                    writer.write_field(aoa.to_string())?;
                }

                writer.write_record(None::<&[u8]>)?;
                writer.flush()?;
            }

            data.push(hist);
        }

        tx.send(group).await.unwrap();
        cnt.inc();
    }

    let root = BitMapBackend::new("aoa.png", (1920, 1080)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .y_label_area_size(80)
        .x_label_area_size(80)
        .top_x_label_area_size(80)
        .build_cartesian_2d(0..data.len(), low..high)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc("Sample #")
        .y_desc("Angle (degrees)")
        .x_label_style(TextStyle::from(("sans-serif", 24)))
        .y_label_style(TextStyle::from(("sans-serif", 24)))
        .draw()?;

    // draw the histogram

    chart.draw_series(data.iter().enumerate().flat_map(|(t, hist)| {
        let max = hist
            .iter()
            .max_by(|a, b| a.value.total_cmp(b.value))
            .unwrap();

        println!("{:?}", max);

        let BinInterval::Bin { start, end } = max.bin else {
            panic!();
        };

        hist.iter()
            .filter_map(move |item| {
                let g = colorgrad::magma();
                let BinInterval::Bin { start, end } = item.bin else {
                    return None;
                };
                let [r, g, b, a] = g.at(item.value / max.value).to_rgba8();

                Some(Rectangle::new(
                    [(t, start), (t + 1, end)],
                    RGBAColor(r, g, b, a as f64 / 255.).filled(),
                ))
            })
            .chain(std::iter::once(Rectangle::new(
                [(t, start), (t + 1, end)],
                RGBAColor(0, 255, 255, 1.),
            )))
    }))?;

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
    #[clap(short, long)]
    channel: u8,
    /// Remove and reinsert the dhd kernel module
    #[clap(short, long, default_value = "false")]
    rmmod: bool,
    /// Dump PCAP data
    #[clap(short, long)]
    dump: Option<PathBuf>,
    /// PCAP input file to replay
    #[clap(long)]
    replay: Option<PathBuf>,
    /// Dump AOA data
    #[clap(long)]
    aoa: Option<PathBuf>,
    /// Don't add delay to replay
    #[clap(long, default_value = "false")]
    replay_quick: bool,
    /// Number of samples to collect. If not specified, will collect indefinitely
    #[clap(short, long)]
    samples: Option<usize>,
}

const RT_AC86U_EXTERNAL: Cores =
    Cores::from_bits_truncate(Cores::CORE0.bits() | Cores::CORE1.bits() | Cores::CORE3.bits());

async fn get_input(args: &RunArgs) -> anyhow::Result<impl Stream<Item = anyhow::Result<WifiCsi>>> {
    let (mut pcap, add_delay) = if let Some(path) = &args.replay {
        (
            PcapSource::File(tokio::fs::File::open(path).await?),
            !args.replay_quick,
        )
    } else {
        let client = connect().await?;
        client
            .configure(
                &Params {
                    chan_spec: ChanSpec::new(args.channel, Band::Band5G, BANDWIDTH).unwrap(),
                    csi_collect: true,
                    cores: RT_AC86U_EXTERNAL,
                    // spatial_streams: SpatialStreams::all(),
                    spatial_streams: SpatialStreams::S0,
                    first_pkt_byte: None,
                    mac_addrs: vec![MACBOOK],
                    delay_us: 0,
                },
                args.rmmod,
            )
            .await?;
        (PcapSource::Router(client.tcpdump().await?), false)
    };

    if let Some(path) = &args.dump {
        let mut file = tokio::fs::File::create(path).await?;
        tokio::io::copy(&mut pcap, &mut file).await?;
        unreachable!()
    } else {
        Ok(read_wifi_csi(pcap, add_delay))
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Run(args) => {
            let app = App::new(args);

            eframe::run_native("🤓", Default::default(), Box::new(|_| Box::new(app))).unwrap();
        }
        Command::Reboot => {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let client = connect().await.unwrap();
                    client.exec("/sbin/reboot").await.unwrap();
                });
        }
        _ => unimplemented!(),
    }

    // if let Command::Aoa { input } = &cli.command {
    //     tokio::runtime::Builder::new_multi_thread()
    //         .build()?
    //         .block_on(async {
    //             let file = tokio::fs::File::open(input).await.unwrap();
    //             let mut frames = pin!(wifi_csi(file));
    //             // let mut frames = frames.skip(100);

    //             let frame = frames.try_next().await.unwrap().unwrap();

    //             plot(&frame).unwrap();

    //             while let Some(frame) = frames.try_next().await.unwrap() {
    //                 if let Some(aoa) = aoa(&frame) {
    //                     println!("{}", aoa);
    //                 }
    //             }
    //         })
    // } else {

    // }

    Ok(())
}
