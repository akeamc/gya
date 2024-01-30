use std::pin::{pin, Pin};

use async_stream::try_stream;
use csi::{
    frame::Frame,
    proc::{FrameGrouper, WifiCsi},
};
use futures::Stream;
use tokio::{io::AsyncRead, time::Instant};

/// Read CSI from a pcap stream.
pub fn read_wifi_csi(
    reader: impl AsyncRead,
    add_delay: bool,
) -> impl Stream<Item = anyhow::Result<WifiCsi>> {
    try_stream! {
        let reader = pin!(reader);
        let mut frames = pcap_file_tokio::pcap::PcapReader::new(reader).await?;
        let mut grouper = FrameGrouper::new();
        let mut t_off = None;
        let start = Instant::now();

        while let Some(pkt) = frames.next_packet().await.transpose()? {
            let t_off = *t_off.get_or_insert(pkt.timestamp);

            if add_delay {
                tokio::time::sleep_until(start + pkt.timestamp - t_off).await;
            }

            let frame = Frame::from_slice(&pkt.data)?;

            if let Some(group) = grouper.add(frame) {
                yield group;
            }
        }

        if let Some(group) = grouper.take() {
            yield group;
        }
    }
}

pub enum PcapSource {
    Router(rt_ac86u::Tcpdump),
    File(tokio::fs::File),
}

impl AsyncRead for PcapSource {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            PcapSource::Router(r) => Pin::new(r).poll_read(cx, buf),
            PcapSource::File(f) => Pin::new(f).poll_read(cx, buf),
        }
    }
}
