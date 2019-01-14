use alarmate::{Area, Client, Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new("admin", "changeme", "192.168.178.10".parse()?);
    client.change_mode(Area::Area1, Mode::Disarmed)?;
    Ok(())
}
