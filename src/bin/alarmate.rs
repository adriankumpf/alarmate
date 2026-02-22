use clap::Parser;

use std::net::Ipv4Addr;

use alarmate::{Area, Client, Mode, Result};

#[derive(Parser, Debug)]
struct ConnectionArgs {
    /// The IP address
    #[arg(value_name = "IP_ADDRESS", env = "ALARMATE_IP_ADDRESS", short = 'I')]
    ip_address: Ipv4Addr,

    /// The password
    #[arg(value_name = "PASSWORD", env = "ALARMATE_PASSWORD", short = 'P')]
    password: String,

    /// The user name
    #[arg(value_name = "USERNAME", env = "ALARMATE_USERNAME", short = 'U')]
    username: String,
}

impl ConnectionArgs {
    /// Create a [`Client`] from these connection arguments.
    fn into_client(self) -> Result<Client> {
        Client::new(&self.username, &self.password, self.ip_address)
    }
}

#[derive(Parser, Debug)]
enum Opt {
    /// List devices
    #[command(name = "devices")]
    Devices {
        #[command(flatten)]
        conn: ConnectionArgs,
    },

    /// Get current status
    #[command(name = "status")]
    Status {
        #[command(flatten)]
        conn: ConnectionArgs,
    },

    /// Change mode
    #[command(name = "mode")]
    Mode {
        #[command(flatten)]
        conn: ConnectionArgs,

        /// The area
        #[arg(value_enum, ignore_case = true, default_value_t = Area::Area1, short, long)]
        area: Area,

        /// The mode
        #[arg(value_enum, ignore_case = true, value_name = "MODE")]
        mode: Mode,
    },
}

#[tokio::main]
async fn main() -> Result {
    match Opt::parse() {
        Opt::Devices { conn } => {
            let mut client = conn.into_client()?;
            let devices = client.list_devices().await?;
            println!("{devices:#?}");
        }

        Opt::Status { conn } => {
            let mut client = conn.into_client()?;
            let status = client.get_status().await?;
            println!("{status:#?}");
        }

        Opt::Mode { conn, mode, area } => {
            let mut client = conn.into_client()?;
            client.change_mode(area, mode).await?;
            println!("{mode:#?}");
        }
    }

    Ok(())
}
