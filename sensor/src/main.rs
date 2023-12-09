use async_ssh2_tokio::client::{AuthMethod, Client, CommandExecutedResult, ServerCheckMethod};

async fn execute(client: &Client, command: impl AsRef<str>) -> anyhow::Result<()> {
    let command = command.as_ref();
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

async fn reload(client: &Client) -> anyhow::Result<()> {
    execute(client, "/sbin/rmmod dhd; /sbin/insmod /jffs/dhd.ko").await
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

    reload(&client).await?;

    execute(&client, "/usr/sbin/wl -i eth6 up").await?;
    execute(&client, "/usr/sbin/wl -i eth6 radio on").await?;
    execute(&client, "/usr/sbin/wl -i eth6 country UG").await?;
    execute(&client, "/usr/sbin/wl -i eth6 chanspec 40/80").await?;
    execute(&client, "/usr/sbin/wl -i eth6 monitor 1").await?;
    execute(&client, "/sbin/ifconfig eth6 up").await?;

    Ok(())
}
