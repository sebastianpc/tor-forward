use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use arti_client::*;
use tor_hsrproxy::{
    config::{Encapsulation, ProxyAction, ProxyConfigBuilder, ProxyPattern, ProxyRule, TargetAddr},
    OnionServiceReverseProxy,
};
use tor_hsservice::{config::OnionServiceConfigBuilder, Anonymity};
use tor_rtcompat::PreferredRuntime;

use anyhow::Result;

#[derive(Debug, PartialEq)]
pub struct Ports {
    rules: Vec<ProxyRule>,
}

impl Ports {
    pub fn builder() -> PortsBuilder {
        PortsBuilder::new()
    }

    pub fn get_rules(self) -> Vec<ProxyRule> {
        self.rules
    }
}

#[derive(Default)]
pub struct PortsBuilder {
    rules: Vec<ProxyRule>,
}

impl PortsBuilder {
    pub fn new() -> PortsBuilder {
        PortsBuilder::default()
    }

    pub fn forward(mut self, remote: u16, local: u16) -> PortsBuilder {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), local);
        let rule = ProxyRule::new(
            ProxyPattern::one_port(remote).unwrap(),
            ProxyAction::Forward(Encapsulation::Simple, TargetAddr::Inet(socket_addr)),
        );
        self.rules.push(rule);
        self
    }

    pub fn expose(mut self, port: u16) -> PortsBuilder {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let rule = ProxyRule::new(
            ProxyPattern::one_port(port).unwrap(),
            ProxyAction::Forward(Encapsulation::Simple, TargetAddr::Inet(socket_addr)),
        );
        self.rules.push(rule);
        self
    }

    pub fn build(self) -> Ports {
        Ports { rules: self.rules }
    }
}

pub async fn hidden_forward(service_name: &str, ports: Ports) -> Result<()> {
    let config = TorClientConfig::default();

    println!("connecting to Tor...");

    let tor_client = TorClient::create_bootstrapped(config).await?;

    let onion_config = OnionServiceConfigBuilder::default()
        .nickname(service_name.to_owned().try_into().unwrap())
        .anonymity(Anonymity::Anonymous)
        .build()?;

    let mut proxy_config = ProxyConfigBuilder::default();
    proxy_config.set_proxy_ports(ports.get_rules());
    let proxy_config = proxy_config.build()?;

    let proxy = OnionServiceReverseProxy::new(proxy_config);

    let (onion_service, rend_requests) = tor_client.launch_onion_service(onion_config)?;

    println!("Serving at: {}", onion_service.onion_name().unwrap());

    let runtime = PreferredRuntime::current()?;

    proxy
        .handle_requests(
            runtime,
            service_name.to_owned().try_into().unwrap(),
            rend_requests,
        )
        .await?;

    Ok(())
}
