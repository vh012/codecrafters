use futures::{SinkExt, StreamExt};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

mod commands;
mod resp;

use resp::parser::RespCodec;

use crate::commands::commands::Command;

async fn handle_tcp_stream(stream: TcpStream) {
    let mut framed = Framed::new(stream, RespCodec::new());

    while let Some(parse_result) = framed.next().await {
        match parse_result {
            Ok(resp_value) => {
                let maybe_cmd = Command::try_perform(resp_value.clone()).await;

                match maybe_cmd {
                    Ok(cmd) => framed.send(cmd.into()).await.unwrap(),
                    Err(e) => eprintln!(
                        "cannot parse command from valid resp {:?}: {:?}",
                        resp_value, e
                    ),
                }
            }
            Err(e) => eprintln!("cannot parse request: {:?}", e),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let Ok((stream, _)) = listener.accept().await else {
            eprintln!("cannot accept stream");

            continue;
        };

        tokio::spawn(handle_tcp_stream(stream));
    }
}
