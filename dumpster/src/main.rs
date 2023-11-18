use pcap::PacketCodec;

fn main() {
    let mut capture = pcap::Capture::from_file("trace.pcap").unwrap();

    dbg!(capture.get_datalink());

    let mut cnt = 0;

    loop {
        let packet = match capture.next_packet() {
            Ok(packet) => packet,
            Err(pcap::Error::NoMorePackets) => break,
            Err(err) => panic!("error while reading packet: {}", err),
        };

        dbg!(packet.header);

        todo!();

        cnt += 1;
    }

    println!("{} packets", cnt);
}
