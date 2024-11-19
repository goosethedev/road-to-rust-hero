use std::net::{Ipv4Addr, SocketAddrV4};

use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

const REDIS_PORT: u16 = 6379;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    // Establish a channel for communication between tasks
    // At most 32 requests can be taken. After that, tasks will be blocked
    // until previous messages are processed.
    let (tx, mut rx) = mpsc::channel(32);

    // Create tasks for senders
    let tx2 = tx.clone();
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".into(),
            resp: resp_tx,
        };

        // Send the SET command
        tx.send(cmd).await.unwrap();

        // Await a response
        println!("Response: {:?}", resp_rx.await);
    });
    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".into(),
            val: "bar".into(),
            resp: resp_tx,
        };

        // Send the SET command
        tx2.send(cmd).await.unwrap();

        // Await a response
        println!("Response: {:?}", resp_rx.await);
    });

    // Create the receiver task
    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, REDIS_PORT);
        let mut client = client::connect(addr).await.unwrap();

        // Listen for received commands until all senders are closed (None)
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    // Make sure to wait on the tasks to finish before terminating.
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();

    // IMPORTANT: Make sure that all tx have been closed (dropped)
    // for the recv loop in the manager to stop (get None).
}
