use clap::Parser;

use std::io::{self, Write};
use std::net::Ipv4Addr;

use alarmate::{Area, Client, Mode, Result};

#[derive(Parser, Debug)]
enum Opt {
    /// List devices
    #[command(name = "devices")]
    Devices {
        /// The IP address
        #[arg(value_name = "IP_ADDRESS", env = "IP_ADDRESS", short = 'I')]
        ip_address: Ipv4Addr,

        /// The password
        #[arg(value_name = "PASSWORD", env = "PASSWORD", short = 'P')]
        password: String,

        /// The user name
        #[arg(value_name = "USERNAME", env = "USERNAME", short = 'U')]
        username: String,
    },

    /// Get current status
    #[command(name = "status")]
    Status {
        /// The IP address
        #[arg(value_name = "IP_ADDRESS", env = "IP_ADDRESS", short = 'I')]
        ip_address: Ipv4Addr,

        /// The password
        #[arg(value_name = "PASSWORD", env = "PASSWORD", short = 'P')]
        password: String,

        /// The user name
        #[arg(value_name = "USERNAME", env = "USERNAME", short = 'U')]
        username: String,
    },

    /// Change mode
    #[command(name = "mode")]
    Mode {
        /// The IP address
        #[arg(value_name = "IP_ADDRESS", env = "IP_ADDRESS", short = 'I')]
        ip_address: Ipv4Addr,

        /// The password
        #[arg(value_name = "PASSWORD", env = "PASSWORD", short = 'P')]
        password: String,

        /// The user name
        #[arg(value_name = "USERNAME", env = "USERNAME", short = 'U')]
        username: String,

        /// The area
        #[arg(value_enum, ignore_case = true, default_value = "Area1", short, long)]
        area: Area,

        /// The mode
        #[arg(value_enum, ignore_case = true, value_name = "MODE")]
        mode: Mode,
    },
}

#[tokio::main]
async fn main() -> Result {
    match Opt::parse() {
        Opt::Devices {
            username,
            password,
            ip_address,
        } => {
            let client = Client::new(&username, &password, ip_address);
            let devices = client.list_devices().await?;
            writeln!(io::stdout(), "{devices:#?}")?;
        }

        Opt::Status {
            username,
            password,
            ip_address,
        } => {
            let client = Client::new(&username, &password, ip_address);
            let status = client.get_status().await?;
            writeln!(io::stdout(), "{status:#?}")?;
        }

        Opt::Mode {
            username,
            password,
            ip_address,
            mode,
            area,
        } => {
            let mut client = Client::new(&username, &password, ip_address);
            client.change_mode(area, &mode).await?;
            writeln!(io::stdout(), "{mode:#?}")?;
        }
    }

    Ok(())
}
