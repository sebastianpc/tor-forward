mod cli;
mod hidden_service;

use anyhow::Result;
use hidden_service::Ports;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = cli::parse();

    let mut portbuilder = Ports::builder();

    if opts.expose.is_some() {
        for port in opts.expose.unwrap() {
            println!("Forwarding {} to 127.0.0.1:{}4", port, port);
            portbuilder = portbuilder.expose(port);
        }
    }

    if opts.forward.is_some() {
        for forward in opts.forward.unwrap() {
            let mut s = forward.split(':');

            let remote = s.next().unwrap().parse::<u16>()?;
            let local = s.next().unwrap().parse::<u16>()?;

            println!("Forwarding {} to 127.0.0.1:{}", remote, local);
            portbuilder = portbuilder.forward(remote, local);
        }
    }

    let ports = portbuilder.build();

    hidden_service::hidden_forward(opts.service_name.as_str(), ports).await?;

    Ok(())
}
