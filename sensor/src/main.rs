use std::{marker::PhantomData, path::PathBuf, pin::pin, sync::Arc};

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

use num_complex::{Complex, ComplexFloat};
use rt_ac86u::RtAc86u;
use sensor::read::{read_wifi_csi, PcapSource};
use tokio::sync::mpsc;
use uom::si::f64::{Length, Velocity};

const MACBOOK: MacAddr6 = MacAddr6::new(0x50, 0xED, 0x3C, 0x2E, 0x04, 0x00);

type Values = WifiCsi;

const C: Velocity = Velocity {
    dimension: PhantomData,
    units: PhantomData,
    value: 299_792_458.,
};

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
    aoas: Vec<(f64, f64)>,
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

                    Plot::new("aoas").min_size(PLOT_SIZE).show(ui, |plot_ui| {
                        let a1 = PlotPoints::from_iter(
                            self.aoas
                                .iter()
                                .enumerate()
                                .map(|(i, (a, _))| [i as f64, a.to_degrees()]),
                        );
                        plot_ui.line(Line::new(a1));
                        let a2 = PlotPoints::from_iter(
                            self.aoas
                                .iter()
                                .enumerate()
                                .map(|(i, (_, a))| [i as f64, a.to_degrees()]),
                        );
                        plot_ui.line(Line::new(a2));
                    });

                    ui.end_row();

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
    let mut stream = pin!(get_input(&args).await?);

    while let Some(group) = stream.try_next().await? {
        tx.send(group).await.unwrap();
        cnt.inc();
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
    /// Dump PCAP data
    #[clap(short, long)]
    dump: Option<PathBuf>,
    /// PCAP input file to replay
    #[clap(long)]
    replay: Option<PathBuf>,
}

const RT_AC86U_EXTERNAL: Cores =
    Cores::from_bits_truncate(Cores::CORE0.bits() | Cores::CORE1.bits() | Cores::CORE3.bits());

async fn get_input(args: &RunArgs) -> anyhow::Result<impl Stream<Item = anyhow::Result<WifiCsi>>> {
    let (mut pcap, add_delay) = if let Some(path) = &args.replay {
        (PcapSource::File(tokio::fs::File::open(path).await?), true)
    } else {
        let client = connect().await?;
        client
            .configure(
                &Params {
                    chan_spec: ChanSpec::new(args.channel, Band::Band5G, Bandwidth::Bw40).unwrap(),
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
