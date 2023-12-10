use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use async_ssh2_tokio::client::{AuthMethod, Client, CommandExecutedResult, ServerCheckMethod};
use dumpster::{
    csi,
    params::{Band, Bandwidth, ChanSpec, CsiParams},
};
use macaddr::MacAddr6;
use pcap_file_tokio::pcap::PcapReader;
use russh::{client::Msg, Channel};
use tokio::io::{AsyncRead, AsyncReadExt};
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

async fn config(client: &Client, channel: u8) -> anyhow::Result<()> {
    // reload
    execute(client, "/sbin/rmmod dhd; /sbin/insmod /jffs/dhd.ko").await?;

    execute(&client, "/usr/sbin/wl -i eth6 down").await?;
    execute(&client, "/usr/sbin/wl -i eth6 up").await?;
    execute(&client, "/usr/sbin/wl -i eth6 radio on").await?;
    execute(&client, "/usr/sbin/wl -i eth6 country UG").await?;
    execute(
        &client,
        format!("/usr/sbin/wl -i eth6 chanspec {channel}/80"),
    )
    .await?;
    execute(&client, "/usr/sbin/wl -i eth6 monitor 1").await?;
    execute(&client, "/sbin/ifconfig eth6 up").await?;

    // 0x7 = 0b0111: 3 spatial streams
    let params = CsiParams {
        chan_spec: ChanSpec::new(100, Band::Band5G, Bandwidth::Bw80).unwrap(),
        csi_collect: true,
        core_mask: 0x7,
        nss_mask: 0x7,
        first_pkt_byte: None,
        mac_addrs: vec![MACBOOK],
        // mac_addrs: vec![],
        delay: 0,
    };

    execute(
        &client,
        format!("/jffs/nexutil -I eth6 -s 500 -b -l 34 -v {params}"),
    )
    .await?;

    // "unsupported"??
    // execute(&client, "/usr/sbin/wl -i eth6 shmem 0x172a 2").await?;
    // execute(&client, "/usr/sbin/wl -i eth6 shmem 0x172c 0").await?;

    Ok(())
}

async fn read_command(client: &Client, command: impl AsRef<str>) -> anyhow::Result<Channel<Msg>> {
    let channel = client.get_channel().await?;
    channel.exec(true, command.as_ref()).await?;

    Ok(channel)
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

    config(&client, 100).await?;

    let mut channel =
        read_command(&client, "/jffs/tcpdump -i eth6 -nn -s 0 -w - port 5500").await?;

    info!("got channel");

    let mut reader = PcapReader::new(channel.make_reader()).await?;

    let mut packets_by_source = BTreeMap::<_, u32>::new();
    let mut i = 0;

    while let Some(res) = reader.next_packet().await {
        let pkt = res?;
        let frame = csi::Frame::from_slice(&pkt.data)?;

        *packets_by_source.entry(frame.source_mac).or_default() += 1;

        i += 1;

        if i % 100 == 0 {
            println!("----");
            for (mac, cnt) in &packets_by_source {
                println!("{}: {}", mac, cnt);
            }
        }
    }

    Ok(())
}
