use std::os::unix::io::AsRawFd;
use std::{error::Error, net::Ipv4Addr};

use smoltcp::{
    iface::{Config, Interface, SocketSet},
    phy::{wait as phy_wait, Device, Medium, TunTapInterface},
    socket::tcp,
    time::Instant,
    wire::{IpAddress, IpCidr, Ipv4Address},
};
use url::Url;

use crate::ethernet::MacAddress;

// Both the IP and URL are needed.
// The URL to fill the request, the IP to send it.
// TODO: Change Box<dyn Error> for custom error type
pub fn get(
    mut tap: TunTapInterface,
    mac: MacAddress,
    ip_addr: Ipv4Addr,
    url: Url,
) -> Result<(), Box<dyn Error>> {
    // Helper variables
    let domain_name = url.host_str().unwrap();
    let fd = tap.as_raw_fd();

    // Configuration for the interface
    let mut config = match tap.capabilities().medium {
        Medium::Ethernet => Config::new(mac.into()),
        Medium::Ip => Config::new(smoltcp::wire::HardwareAddress::Ip),
        Medium::Ieee802154 => todo!(),
    };
    config.random_seed = rand::random();

    // Create the interface
    let mut iface = Interface::new(config, &mut tap, Instant::now());
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(
                IpAddress::Ipv4(Ipv4Address([192, 168, 42, 1])),
                24,
            ))
            .unwrap()
    });
    iface
        .routes_mut()
        .add_default_ipv4_route(Ipv4Address([192, 168, 42, 100]))
        .unwrap();

    // Create sockets
    let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);

    let mut sockets = SocketSet::new(vec![]);
    let tcp_handle = sockets.add(tcp_socket);

    // Build the HTTP GET template
    let http_get = format!(
        "GET {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n\r\n",
        url.path(),
        domain_name
    );

    // Go through the request states
    enum State {
        Connect,
        Request,
        Response,
    }
    let mut state = State::Connect;

    loop {
        let timestamp = Instant::now();
        iface.poll(timestamp, &mut tap, &mut sockets);

        let socket = sockets.get_mut::<tcp::Socket>(tcp_handle);
        let cx = iface.context();

        state = match state {
            State::Connect if !socket.is_active() => {
                dbg!("connecting");
                let local_port = 49152 + rand::random::<u16>() % 16384;
                socket
                    .connect(cx, (ip_addr, url.port().unwrap_or(80)), local_port)
                    .unwrap();
                State::Request
            }
            State::Request if socket.may_send() => {
                dbg!("sending request");
                socket
                    .send_slice(http_get.as_bytes())
                    .expect("cannot send request");
                socket.send_slice(b"\r\n").expect("cannot send");
                State::Response
            }
            // TODO: split headers and payload. Return payload.
            State::Response if socket.can_recv() => {
                socket
                    .recv(|data| {
                        println!(
                            "{}",
                            String::from_utf8(data.to_owned()).unwrap_or("(invalid utf8)".into())
                        );
                        (data.len(), ())
                    })
                    .unwrap();
                State::Response
            }
            State::Response if !socket.may_recv() => {
                dbg!("received complete response");
                break;
            }
            _ => state,
        };

        phy_wait(fd, iface.poll_delay(timestamp, &sockets)).expect("wait error");
    }
    Ok(())
}
