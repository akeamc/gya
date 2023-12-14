use std::{collections::BTreeMap, time::Duration};

use async_ssh2_tokio::client::{AuthMethod, Client, CommandExecutedResult, ServerCheckMethod};
use csi::{
    frame,
    params::{Band, Bandwidth, ChanSpec, Cores, Params, SpatialStreams},
};
use macaddr::MacAddr6;
use pcap_file_tokio::pcap::PcapReader;
use russh::{client::Msg, Sig};
use tracing::info;

const MACBOOK: MacAddr6 = MacAddr6::new(0x50, 0xED, 0x3C, 0x2E, 0x04, 0x00);

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
        cores: Cores::CORE0 | Cores::CORE1 | Cores::CORE2,
        spatial_streams: SpatialStreams::S0 | SpatialStreams::S1 | SpatialStreams::S2,
        first_pkt_byte: None,
        // mac_addrs: vec![MACBOOK],
        mac_addrs: vec![],
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

async fn handle_ssh(
    ssh: &mut russh::Channel<Msg>,
    packets_by_source: &mut BTreeMap<MacAddr6, u32>,
    cnt: &mut u32,
) -> anyhow::Result<()> {
    let mut reader = PcapReader::new(ssh.make_reader()).await?;

    while let Some(res) = reader.next_packet().await {
        let pkt = res?;
        let frame = frame::Frame::from_slice(&pkt.data)?;

        *packets_by_source.entry(frame.source_mac).or_default() += 1;

        *cnt += 1;

        if *cnt % 100 == 0 {
            println!("----");
            for (mac, cnt) in packets_by_source.iter() {
                println!("{}: {}", mac, cnt);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // if you want to use key auth, then use following:
    // AuthMethod::with_key_file("key_file_name", Some("passphrase"));
    // or
    // AuthMethod::with_key_file("key_file_name", None);
    // or
    // AuthMethod::with_key(key: &str, passphrase: Option<&str>)
    let auth_method = AuthMethod::with_password("password");
    let client = Client::connect(
        ("192.168.0.84", 22),
        "admin",
        auth_method,
        ServerCheckMethod::NoCheck,
    )
    .await?;

    let mut packets_by_source = BTreeMap::<_, u32>::new();
    let mut cnt = 0;

    for i in [
        36, 40, 44, 48, 52, 56, 60, 64, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136,
    ] {
        config(&client, i, false).await?;

        let mut ssh = client.get_channel().await?;

        ssh.exec(true, "/jffs/tcpdump -i eth6 -nn -s 0 -w - port 5500")
            .await?;
        info!("got ssh");

        let _ = tokio::time::timeout(Duration::from_secs(1), async {
            handle_ssh(&mut ssh, &mut packets_by_source, &mut cnt).await
        })
        .await;

        ssh.signal(Sig::INT).await?;
    }

    Ok(())
}
