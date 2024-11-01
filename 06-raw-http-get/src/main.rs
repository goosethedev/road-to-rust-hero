use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use clap::Parser;
use smoltcp::phy::{Medium, TunTapInterface};
use url::Url;

mod dns;
mod ethernet;
mod http;

#[derive(Parser)]
#[command(name = "raw-http-get")]
#[command(version, about = "GET a webpage, manually", long_about = None)]
struct Cli {
    url: String,
    tap_device: String,

    #[arg(default_value = "1.1.1.1")]
    dns_server: String,
}

fn main() {
    let cli = Cli::parse();

    // Parse args
    let url = Url::parse(cli.url.as_str()).expect("Couldn't parse URL");
    let tap_device = TunTapInterface::new("tap-rust", Medium::Ethernet)
        .expect("Unable to use <tap_device> as a network interface");
    let dns_ip =
        Ipv4Addr::from_str(&cli.dns_server).expect("DNS server is not a valid IPv4 address");

    // Validate URL
    if url.scheme() != "http" {
        panic!("Error: only HTTP protocol supported");
    }
    let domain_name = url.host_str().expect("Domain name required");

    // Send HTTP request
    let dns_server = SocketAddrV4::new(dns_ip, 53);
    let addr = dns::resolve(dns_server, domain_name).unwrap().unwrap();
    let mac = ethernet::MacAddress::new();

    http::get(tap_device, mac, addr, url).unwrap();
}
