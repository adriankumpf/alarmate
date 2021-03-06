use structopt::StructOpt;

use std::io::{self, Write};
use std::net::Ipv4Addr;

use alarmate::{Area, Client, Mode, Result};

#[derive(StructOpt, Debug)]
#[structopt()]
enum Opt {
    /// List devices
    #[structopt(name = "devices")]
    Devices {
        /// The IP address
        #[structopt(name = "IP_ADDRESS", env = "IP_ADDRESS", short)]
        ip_address: Ipv4Addr,

        /// The password
        #[structopt(name = "PASSWORD", env = "PASSWORD", short)]
        password: String,

        /// The user name
        #[structopt(name = "USERNAME", env = "USERNAME", short)]
        username: String,
    },

    /// Get current status
    #[structopt(name = "status")]
    Status {
        /// The IP address
        #[structopt(name = "IP_ADDRESS", env = "IP_ADDRESS", short)]
        ip_address: Ipv4Addr,

        /// The password
        #[structopt(name = "PASSWORD", env = "PASSWORD", short)]
        password: String,

        /// The user name
        #[structopt(name = "USERNAME", env = "USERNAME", short)]
        username: String,
    },

    /// Change mode
    #[structopt(name = "mode")]
    Mode {
        /// The IP address
        #[structopt(name = "IP_ADDRESS", env = "IP_ADDRESS", short)]
        ip_address: Ipv4Addr,

        /// The password
        #[structopt(name = "PASSWORD", env = "PASSWORD", short)]
        password: String,

        /// The user name
        #[structopt(name = "USERNAME", env = "USERNAME", short)]
        username: String,

        /// The area
        #[structopt(possible_values = &Area::variants(), case_insensitive = true, default_value = "Area1", short, long)]
        area: Area,

        /// The mode
        #[structopt(possible_values = &Mode::variants(), case_insensitive = true, name = "MODE")]
        mode: Mode,
    },
}

#[tokio::main]
async fn main() -> Result {
    match Opt::from_args() {
        Opt::Devices {
            username,
            password,
            ip_address,
        } => {
            let client = Client::new(&username, &password, ip_address);
            let devices = client.list_devices().await?;
            writeln!(io::stdout(), "{:#?}", devices)?;
        }

        Opt::Status {
            username,
            password,
            ip_address,
        } => {
            let client = Client::new(&username, &password, ip_address);
            let status = client.get_status().await?;
            writeln!(io::stdout(), "{:#?}", status)?;
        }

        Opt::Mode {
            username,
            password,
            ip_address,
            mode,
            area,
        } => {
            let mut client = Client::new(&username, &password, ip_address);
            let mode = client.change_mode(area, mode).await?;
            writeln!(io::stdout(), "{:#?}", mode)?;
        }
    }

    Ok(())
}
