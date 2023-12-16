use std::{pin::pin, sync::Arc};

use async_ssh2_tokio::client::{AuthMethod, Client, CommandExecutedResult, ServerCheckMethod};
use async_stream::try_stream;
use atomic_counter::{AtomicCounter, RelaxedCounter};
use csi::{
    frame::Frame,
    params::{Band, Bandwidth, ChanSpec, Cores, Params, SpatialStreams},
};
use egui::load::SizedTexture;
use futures::{Stream, StreamExt, TryStreamExt};
use macaddr::MacAddr6;
use pcap_file_tokio::pcap::PcapReader;
use tokio::{io::AsyncRead, sync::mpsc};
use tracing::info;

const MACBOOK: MacAddr6 = MacAddr6::new(0x50, 0xED, 0x3C, 0x2E, 0x04, 0x00);

const DIAGRAMS: usize = 16;

async fn execute(client: &Client, command: impl AsRef<str>) -> anyhow::Result<()> {
    let command = command.as_ref();
    info!(%command, "executing command");
    let CommandExecutedResult {
        exit_status,
        stderr,
        ..
    } = client.execute(command).await?;
    if exit_status != 0 {
        anyhow::bail!("{command:?} returned exit status {exit_status}: {stderr}");
    }
    Ok(())
}

async fn config(client: &Client, channel: u8, reload: bool) -> anyhow::Result<()> {
    if reload {
        execute(client, "/sbin/rmmod dhd; /sbin/insmod /jffs/dhd.ko").await?;
    }

    execute(client, "/usr/sbin/wl -i eth6 down").await?;
    execute(client, "/usr/sbin/wl -i eth6 up").await?;
    execute(client, "/usr/sbin/wl -i eth6 radio on").await?;
    execute(client, "/usr/sbin/wl -i eth6 country UG").await?;
    execute(
        client,
        format!("/usr/sbin/wl -i eth6 chanspec {channel}/80"),
    )
    .await?;
    execute(client, "/usr/sbin/wl -i eth6 monitor 1").await?;
    execute(client, "/sbin/ifconfig eth6 up").await?;

    let params = Params {
        chan_spec: ChanSpec::new(channel, Band::Band5G, Bandwidth::Bw80).unwrap(),
        csi_collect: true,
        cores: Cores::CORE0 | Cores::CORE1 | Cores::CORE2 | Cores::CORE3,
        spatial_streams: SpatialStreams::S0
            | SpatialStreams::S1
            | SpatialStreams::S2
            | SpatialStreams::S3,
        first_pkt_byte: None,
        mac_addrs: vec![MACBOOK],
        // mac_addrs: vec![],
        delay_us: 0,
    };

    execute(
        client,
        format!("/jffs/nexutil -I eth6 -s 500 -b -l 34 -v {params}"),
    )
    .await?;

    // "unsupported"??
    // execute(&client, "/usr/sbin/wl -i eth6 shmem 0x172a 2").await?;
    // execute(&client, "/usr/sbin/wl -i eth6 shmem 0x172c 0").await?;

    Ok(())
}

fn csi_stream(pcap: impl AsyncRead + Unpin) -> impl Stream<Item = anyhow::Result<Frame>> {
    try_stream! {
        let mut reader = PcapReader::new(pcap).await?;

        while let Some(res) = reader.next_packet().await {
            let pkt = res?;
            let frame = Frame::from_slice(&pkt.data)?;

            yield frame;
        }
    }
}

fn group_frames(
    frames: impl Stream<Item = anyhow::Result<Frame>>,
) -> impl Stream<Item = anyhow::Result<Vec<Frame>>> {
    try_stream! {
        let mut frames = pin!(frames.peekable());

        while let Some(frame) = frames.try_next().await? {
            let seq_cnt = frame.seq_cnt;
            let mut group = vec![frame];

            while let Some(Ok(f)) = frames.as_mut().peek().await {
                if f.seq_cnt != seq_cnt {
                    break;
                }

                group.push(frames.as_mut().next().await.unwrap().unwrap());
            }

            yield group;
        }
    }
}

type Values = Vec<Frame>;

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

    fn add(&mut self, values: &[Frame]) {
        let grad = colorgrad::sinebow();

        for (i, frame) in values.iter().enumerate() {
            let max = frame
                .csi_values
                .iter()
                .map(|z| z.norm_sqr())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                .sqrt();

            for (j, v) in frame.csi_values.iter().enumerate() {
                let mut c = grad.at((v.arg() + std::f64::consts::PI) / std::f64::consts::TAU);
                let scale = v.norm() / max;
                c.r *= scale;
                c.g *= scale;
                c.b *= scale;

                self.image
                    .put_pixel(self.pos, (i * 256 + j) as _, image::Rgba(c.to_rgba8()));
            }
        }

        self.prev_pos = self.pos;
        self.pos += 1;
        if self.pos == self.image.width() {
            self.pos = 0;
        }
    }
}

struct App {
    rt: tokio::runtime::Runtime,
    rx: mpsc::Receiver<Values>,
    cnt: Arc<RelaxedCounter>,
    texture: Option<egui::TextureHandle>,
    waterfall: Waterfall,
}

impl App {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let cnt = Arc::new(RelaxedCounter::new(0));

        rt.spawn({
            let cnt = cnt.clone();
            async move {
                data(tx, &cnt).await.unwrap();
            }
        });

        Self {
            rt,
            rx,
            cnt,
            texture: None,
            waterfall: Waterfall::new(3000, 256 * DIAGRAMS as u32),
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
        let tex = self.texture.get_or_insert_with(|| {
            ctx.load_texture("img", egui_image(&self.waterfall.image), Default::default())
        });

        while let Ok(frames) = self.rx.try_recv() {
            if frames.len() != DIAGRAMS {
                continue;
            }
            self.waterfall.add(&frames);
        }

        // tex.set_partial(
        //     [self.waterfall.prev_pos as usize, 0],
        //     egui_image(&self.waterfall.image.sub_image(
        //         self.waterfall.prev_pos,
        //         0,
        //         1,
        //         self.waterfall.image.height(),
        //     ).to_image()),
        //     Default::default(),
        // );
        tex.set(egui_image(&self.waterfall.image), Default::default());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("{} packets", self.cnt.get()));
            ui.add(egui::Image::new(SizedTexture::from(&*tex)).shrink_to_fit());
        });
        ctx.request_repaint();
    }
}

async fn data(tx: mpsc::Sender<Values>, cnt: &RelaxedCounter) -> anyhow::Result<()> {
    // if you want to use key auth, then use following:
    // AuthMethod::with_key_file("key_file_name", Some("passphrase"));
    // or
    // AuthMethod::with_key_file("key_file_name", None);
    // or
    // AuthMethod::with_key(key: &str, passphrase: Option<&str>)
    let client = Client::connect(
        ("192.168.0.84", 22),
        "admin",
        AuthMethod::with_password("password"),
        ServerCheckMethod::NoCheck,
    )
    .await?;

    // for i in [
    //     36, 40, 44, 48, 52, 56, 60, 64, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136,
    // ] {
    // config(&client, i, false).await?;
    config(&client, 100, false).await?;

    let mut ssh = client.get_channel().await?;

    ssh.exec(true, "/jffs/tcpdump -i eth6 -nn -s 0 -w - port 5500")
        .await?;
    info!("got ssh");

    // let _ = tokio::time::timeout(Duration::from_secs(1), async {
    let frames = csi_stream(ssh.make_reader());
    let mut stream = pin!(group_frames(frames));

    while let Some(group) = stream.try_next().await? {
        // for frame in group {
        // println!(
        //     "{}\t{}\t{}\t{}\t{}",
        //     frame.source_mac,
        //     frame.core,
        //     frame.spatial,
        //     frame.seq_cnt,
        //     frame.csi_values.len(),
        // );

        // values[i] = frame.rssi as f32;
        // i += 1;

        // if i == values.len() {
        // i = 0;
        // }
        // }

        tx.send(group).await.unwrap();

        cnt.inc();
    }
    // })
    // .await;

    // ssh.signal(Sig::INT).await?;
    // }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = App::new();

    eframe::run_native("SENSOR", Default::default(), Box::new(|_| Box::new(app))).unwrap();

    Ok(())
}
