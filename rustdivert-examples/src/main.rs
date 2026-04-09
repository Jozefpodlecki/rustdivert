use anyhow::Result;
use flexi_logger::Logger;
use log::*;

use crate::{examples::{param::*, recv::*}, prompt::*};

mod server;
mod client;
mod examples;
mod prompt;

async fn run() -> Result<AppMode> {
    match prompt_mode()? {
        AppMode::GetSetParams => {
            let windivert = create_basic_windivert()?;
            print_params(&windivert)?;
            modify_param(&windivert)?;
            print_params(&windivert)?;

            Ok(AppMode::Continue)
        }
        AppMode::ClientServer => {
            let ip = prompt_ip()?;
            let port = prompt_port()?;

            let addr = format!("{ip}:{port}");

            // if let Err(err) = setup_client_server(port) {
            //     error!("Failed to start client/server: {e}");
            // }

            Ok(AppMode::Continue)
        }
        AppMode::RustDivert => {
            let port = prompt_port()?;
            let filter = prompt_filter(port)?;

            // if let Err(e) = run_using_rustdivert_with_filter(filter).await {
            //     error!("RustDivert error: {e}");
            // }

            Ok(AppMode::Continue)
        }

        AppMode::Exit => Ok(AppMode::Exit),
        AppMode::Continue => unreachable!(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Logger::try_with_str(
        "debug,rustdivert_examples::client=error,rustdivert_examples::server=error"
    )?.start()?;

    loop {
        match run().await {
            Ok(mode) => {
                if let AppMode::Exit = mode {
                    break;
                }
            },
            Err(err) => {
                error!("{err}");
            },
        }
    }

    Ok(())
}