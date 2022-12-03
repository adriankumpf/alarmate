use alarmate::{Area, Client, Mode};

#[tokio::main]
async fn main() -> alarmate::Result {
    let mut client = Client::new("admin", "changeme", "192.168.178.10".parse().unwrap());
    client.change_mode(Area::Area1, &Mode::Disarmed).await?;
    Ok(())
}
