use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use arti_client::*;
use tor_hsrproxy::{config::{Encapsulation, ProxyAction, ProxyConfigBuilder, ProxyPattern, ProxyRule, TargetAddr}, OnionServiceReverseProxy};
use tor_hsservice::{config::OnionServiceConfigBuilder, Anonymity};
use tor_rtcompat::PreferredRuntime;

use anyhow::Result;

async fn hidden_forward(service_name: &str) -> Result<()> {


    let config = TorClientConfig::default();

    println!("connecting to Tor...");
    let tor_client = TorClient::create_bootstrapped(config).await?;

    let onion_config = OnionServiceConfigBuilder::default()
        .nickname(service_name.to_owned().try_into().unwrap())
        .anonymity(Anonymity::Anonymous)
        .build()?;

        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 22);

    let ports = vec![ProxyRule::new(ProxyPattern::one_port(22)?, ProxyAction::Forward(Encapsulation::Simple, TargetAddr::Inet(socket)))];
    let mut proxy_config = ProxyConfigBuilder::default();
    proxy_config.set_proxy_ports(ports);
    let proxy_config = proxy_config.build()?;
    
    let proxy = OnionServiceReverseProxy::new(proxy_config);

    let (onion_service, rend_requests) = tor_client.launch_onion_service(onion_config)?;

    println!("serving at: {}", onion_service.onion_name().unwrap());
    
    let runtime = PreferredRuntime::current()?;

    proxy.handle_requests(runtime, service_name.to_owned().try_into().unwrap(), rend_requests).await?;

    Ok(())

}
 

#[tokio::main]
async fn main() -> Result<()> {
    let mut handles = vec![];
    handles.push(tokio::spawn(async {hidden_forward("ssh-proxy").await}));
    handles.push(tokio::spawn(async {
        println!("Running after spawn");
        Ok(())
    }));

    futures::future::join_all(handles).await;
    Ok(())
}