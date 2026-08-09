use clap::Parser;

use std::net::Ipv4Addr;
use std::process::ExitCode;

use alarmate::{Area, Client, Mode, Result};

// `Debug` is deliberately not derived: these hold the password in plaintext.
#[derive(Parser)]
struct ConnectionArgs {
    /// The IP address
    #[arg(env = "ALARMATE_IP_ADDRESS", short = 'I', long)]
    ip_address: Ipv4Addr,

    /// The user name
    #[arg(env = "ALARMATE_USERNAME", short = 'U', long)]
    username: String,

    /// The password. Prefer ALARMATE_PASSWORD: arguments are visible via `ps`
    #[arg(env = "ALARMATE_PASSWORD", short = 'P', long, hide_env_values = true)]
    password: String,
}

impl ConnectionArgs {
    fn into_client(self) -> Result<Client> {
        Client::new(&self.username, &self.password, self.ip_address)
    }
}

#[derive(Parser)]
#[command(version, about)]
enum Opt {
    /// List devices
    Devices {
        #[command(flatten)]
        conn: ConnectionArgs,
    },

    /// Get current status
    Status {
        #[command(flatten)]
        conn: ConnectionArgs,
    },

    /// Change mode
    Mode {
        #[command(flatten)]
        conn: ConnectionArgs,

        /// The area
        #[arg(value_enum, ignore_case = true, default_value_t = Area::Area1, short, long)]
        area: Area,

        /// The mode
        #[arg(value_enum, ignore_case = true)]
        mode: Mode,
    },
}

// The CLI issues a couple of sequential requests and exits; a worker pool per
// invocation buys nothing.
#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn run() -> Result {
    match Opt::parse() {
        Opt::Devices { conn } => {
            let devices = conn.into_client()?.list_devices().await?;
            println!("{devices:#?}");
        }

        Opt::Status { conn } => {
            let status = conn.into_client()?.get_status().await?;
            println!("{status:#?}");
        }

        Opt::Mode { conn, mode, area } => {
            conn.into_client()?.change_mode(area, mode).await?;
            println!("{mode}");
        }
    }

    Ok(())
}
