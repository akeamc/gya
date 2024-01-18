use std::pin::pin;

use async_stream::try_stream;
use csi::{
    frame::Frame,
    proc::{FrameGrouper, WifiCsi},
};
use futures::Stream;

pub fn wifi_csi(reader: impl tokio::io::AsyncRead) -> impl Stream<Item = anyhow::Result<WifiCsi>> {
    try_stream! {
      let reader = pin!(reader);
        let mut frames = pcap_file_tokio::pcap::PcapReader::new(reader).await?;
        let mut grouper = FrameGrouper::new();

        while let Some(frame) = frames.next_packet().await.transpose()? {
          let frame = Frame::from_slice(&frame.data)?;

            if let Some(group) = grouper.add(frame) {
                yield group;
            }
        }

        if let Some(group) = grouper.take() {
            yield group;
        }
    }
}
