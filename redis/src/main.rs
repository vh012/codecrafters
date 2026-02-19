use futures::{SinkExt, StreamExt};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

mod commands;
mod config;
mod rdb;
mod resp;

use clap::Parser;

use crate::{
    commands::commands::Command,
    config::CONFIG,
    resp::{parser::RespCodec, types::RespDataType},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    dir: Option<String>,
    #[arg(short, long)]
    dbfilename: Option<String>,
}

async fn send_frame(framed: &mut Framed<TcpStream, RespCodec>, resp: RespDataType) {
    if let Err(e) = framed.send(resp).await {
        eprintln!("cannot send frame {e}");
    }
}

async fn handle_tcp_stream(stream: TcpStream) {
    let mut framed = Framed::new(stream, RespCodec::new());

    while let Some(parse_result) = framed.next().await {
        match parse_result {
            Ok(resp_value) => {
                let maybe_cmd = Command::try_perform(resp_value.clone()).await;

                match maybe_cmd {
                    Ok(cmd) => send_frame(&mut framed, cmd.into()).await,
                    Err(e) => {
                        eprintln!("cannot parse command from valid RESP {resp_value:?}: {e:?}");

                        send_frame(&mut framed, e.into()).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("cannot parse request: {e:?}");

                send_frame(&mut framed, e.into()).await;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    // let mut file = File::open("foo.txt").await?;

    // file.set_max_buf_size(max_buf_size);

    // let mut framed = Framed::new(file, RespCodec::new());

    let args = Args::parse();

    let mut config = CONFIG.write().await;

    config.dir = args.dir;
    config.dbfilename = args.dbfilename;

    drop(config);

    loop {
        let Ok((stream, _)) = listener.accept().await else {
            eprintln!("cannot accept stream");

            continue;
        };

        tokio::spawn(handle_tcp_stream(stream));
    }
}
