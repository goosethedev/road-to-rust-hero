use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
    str::FromStr,
    time::Duration,
};

use hickory_proto::{
    op::{Message, MessageType, OpCode, Query},
    rr::{Name, RData, RecordType},
    serialize::binary::{BinEncodable, BinEncoder},
};

fn random_message_id() -> u16 {
    loop {
        let id = rand::random();
        if id != 0 {
            break id;
        };
    }
}

// TODO: Change Box for custom error
pub fn resolve(
    dns_server: SocketAddrV4,
    domain_name: &str,
) -> Result<Option<Ipv4Addr>, Box<dyn Error>> {
    // Parse domain name
    let domain_name = Name::from_str(domain_name)?;

    // Setup message container
    let mut message = Message::new();
    message
        .add_query(Query::query(domain_name, RecordType::A))
        .set_id(random_message_id())
        .set_message_type(MessageType::Query)
        .set_op_code(OpCode::Query)
        .set_recursion_desired(true);

    // Setup local port binding to send the request
    let localhost = UdpSocket::bind("0.0.0.0:0")?;
    localhost.set_read_timeout(Some(Duration::from_secs(5)))?;
    localhost.set_nonblocking(false)?;

    // Initialize buffers
    let mut request_buffer: Vec<u8> = Vec::with_capacity(64);
    let mut response_buffer = vec![0_u8; 512];
    let mut encoder = BinEncoder::new(&mut request_buffer);
    message.emit(&mut encoder)?;

    let _n_bytes_sent = localhost.send_to(&request_buffer, dns_server);

    // Loop until the server responds (other UDP packets may interfere)
    loop {
        let (_b_bytes_recv, remote_port) = localhost.recv_from(&mut response_buffer)?;

        if remote_port == dns_server.into() {
            break;
        }
    }

    // Extract result from response
    // TODO: Maybe improve this?
    let result = Message::from_vec(&response_buffer)?;
    let first_record = result
        .answers()
        .iter()
        .find(|a| a.record_type() == RecordType::A);

    if let Some(a) = first_record {
        if let Some(RData::A(ip)) = a.data() {
            return Ok(Some(**ip));
        }
    }
    Ok(None)
}
