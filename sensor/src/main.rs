use std::{path::PathBuf, pin::pin, sync::Arc};

use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use atomic_counter::{AtomicCounter, RelaxedCounter};
use clap::{Args, Parser, Subcommand};
use csi::{
    ieee80211::{Band, Bandwidth},
    params::{ChanSpec, Cores, Params, SpatialStreams},
    proc::WifiCsi,
};
use egui::Vec2;
use egui_plot::{Line, Plot, PlotPoints};
use futures::{Stream, TryStreamExt};
use macaddr::MacAddr6;
use ndarray::{Array3, Axis};
use num_complex::{Complex, ComplexFloat};
use rt_ac86u::RtAc86u;
use rustfft::Fft;
use sensor::read::{read_wifi_csi, PcapSource};
use tokio::sync::mpsc;

const MACBOOK: MacAddr6 = MacAddr6::new(0x50, 0xED, 0x3C, 0x2E, 0x04, 0x00);

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
}

impl App {
    fn new(args: RunArgs) -> Self {
        let (tx, rx) = mpsc::channel(1000);

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
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let tex = self.texture.get_or_insert_with(|| {
        //     ctx.load_texture("img", egui_image(&self.waterfall.image), Default::default())
        // });

        if self.prev_i != self.i {
            self.prev_i = self.i;
            self.last = false;
        }

        while let Ok(csi) = self.rx.try_recv() {
            if csi.frames().iter().flatten().all(Option::is_some) {
                self.data.push(csi);
            }
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
            ui.label(format!("{} packets", self.cnt.get()));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("grid").show(ui, |ui| {
                let data = if self.last {
                    self.data.last()
                } else {
                    self.data.get(self.i)
                };
                let Some(data) = data else {
                    return;
                };

                const GRID_SIZE: Vec2 = Vec2 { x: 500., y: 250. };

                let core_n = data.get(self.core, self.spatial).unwrap();

                Plot::new("amplitude")
                    .auto_bounds(egui::Vec2b::FALSE)
                    .include_x(-128.)
                    .include_x(128.)
                    .include_y(0.)
                    .include_y(4.)
                    .min_size(GRID_SIZE)
                    .show(ui, |plot_ui| {
                        let scale = 10f64.powf(data.rssi as f64 / 20.)
                            / core_n.mapv(Complex::abs).sum().sqrt()
                            / 100.;

                        let points = PlotPoints::from_iter(
                            core_n
                                .indexed_iter()
                                .map(|(i, z)| [i as f64 - 128., scale * z.abs()]),
                        );
                        plot_ui.line(Line::new(points));
                    });

                ui.end_row();

                Plot::new("phase")
                    .auto_bounds(egui::Vec2b::FALSE)
                    .include_x(-128.)
                    .include_x(128.)
                    .include_y(-4.)
                    .include_y(4.)
                    .min_size(GRID_SIZE)
                    .show(ui, |plot_ui| {
                        let core_0 = data.get(0, self.spatial).unwrap();

                        let points = PlotPoints::from_iter(
                            core_n
                                .indexed_iter()
                                .map(|(i, z)| [i as f64 - 128., (z / core_0[i]).arg()]),
                        );
                        plot_ui.line(Line::new(points));
                    });

                ui.end_row();

                let mut fft = core_n.to_vec();
                rustfft::algorithm::Radix4::new(core_n.len(), rustfft::FftDirection::Forward)
                    .process(&mut fft);

                Plot::new("fft")
                    .auto_bounds(egui::Vec2b { x: false, y: true })
                    .include_x(-128.)
                    .include_x(128.)
                    .min_size(GRID_SIZE)
                    .show(ui, |plot_ui| {
                        let points = PlotPoints::from_iter(
                            fft.iter()
                                .enumerate()
                                .map(|(i, z)| [i as f64 - 128., z.norm()]),
                        );
                        plot_ui.line(Line::new(points));
                    });
            });

            // tex.set(egui_image(&self.waterfall.image), Default::default());
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

async fn get_input(args: &RunArgs) -> anyhow::Result<impl Stream<Item = anyhow::Result<WifiCsi>>> {
    let (mut pcap, add_delay) = if let Some(path) = &args.replay {
        (PcapSource::File(tokio::fs::File::open(path).await?), true)
    } else {
        let client = connect().await?;
        client
            .configure(
                &Params {
                    chan_spec: ChanSpec::new(args.channel, Band::Band5G, Bandwidth::Bw80).unwrap(),
                    csi_collect: true,
                    cores: Cores::all(),
                    spatial_streams: SpatialStreams::all(),
                    first_pkt_byte: None,
                    mac_addrs: vec![MACBOOK],
                    delay_us: 50,
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
