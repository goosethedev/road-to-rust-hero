use std::net::{Ipv4Addr, SocketAddrV4};

use mini_redis::client;

const REDIS_PORT: u16 = 6379;

#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    // Open a connection to the mini-redis address.
    let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, REDIS_PORT);
    let mut client = client::connect(addr).await?;

    // Set the 'hello' key with the value 'world'
    client.set("hello", "world".into()).await?;

    // Get the 'hello' key
    let result = client.get("hello").await?;

    println!("Key: hello - Value: {:?}", result);

    Ok(())
}
