use tracing::{debug, info, instrument, Span};

pub use async_ssh2_tokio;

pub struct RtAc86u {
    client: async_ssh2_tokio::Client,
}

impl RtAc86u {
    pub fn new(client: async_ssh2_tokio::Client) -> Self {
        Self { client }
    }

    #[instrument(skip_all, fields(command))]
    pub async fn exec(&self, command: impl AsRef<str>) -> anyhow::Result<()> {
        let command = command.as_ref();
        // display the command in the logs
        Span::current().record("command", command);
        let async_ssh2_tokio::client::CommandExecutedResult {
            exit_status,
            stderr,
            stdout,
        } = self.client.execute(command).await?;
        info!("exited with status {exit_status}");
        debug!("stdout: {stdout}");
        debug!("stderr: {stderr}");
        if exit_status != 0 {
            anyhow::bail!("{command:?} returned exit status {exit_status}: {stderr}");
        }
        Ok(())
    }

    pub async fn configure(&self, params: csi::params::Params, rmmod: bool) -> anyhow::Result<()> {
        if rmmod {
            self.exec("/sbin/rmmod dhd; /sbin/insmod /jffs/dhd.ko")
                .await?;
        }

        dbg!(params.chan_spec);

        self.exec("/usr/sbin/wl -i eth6 down").await?;
        self.exec("/usr/sbin/wl -i eth6 up").await?;
        self.exec("/usr/sbin/wl -i eth6 radio on").await?;
        self.exec("/usr/sbin/wl -i eth6 country UG").await?;
        self.exec(format!(
            "/usr/sbin/wl -i eth6 chanspec {}/{}",
            params.chan_spec.original().unwrap(),
            params.chan_spec.bandwidth().mhz(),
        ))
        .await?;
        self.exec("/usr/sbin/wl -i eth6 monitor 1").await?;
        self.exec("/sbin/ifconfig eth6 up").await?;

        let params = params.to_string();

        self.exec(format!(
            "/jffs/nexutil -I eth6 -s 500 -b -l {len} -v {params}",
            len = params.len()
        ))
        .await?;

        // "unsupported"??
        // execute(&client, "/usr/sbin/wl -i eth6 shmem 0x172a 2").await?;
        // execute(&client, "/usr/sbin/wl -i eth6 shmem 0x172c 0").await?;

        Ok(())
    }

    pub async fn tcpdump(&self) -> Result<impl tokio::io::AsyncRead, async_ssh2_tokio::Error> {
        let channel = self.client.get_channel().await?;
        // dump CSI packets to stdout
        channel
            .exec(true, "/jffs/tcpdump -i eth6 -nn -s 0 -w - port 5500")
            .await?;
        info!("tcpdump started");
        Ok(channel.into_stream())
    }
}
