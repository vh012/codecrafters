use futures::{SinkExt, StreamExt};
use std::error::Error;
use std::path::Path;
use tokio::fs::File;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

mod commands;
mod config;
mod rdb;
mod resp;

use clap::Parser;

use crate::commands::hash_map::HASH_MAP;
use crate::{
    commands::processor::Processor,
    config::CONFIG,
    rdb::parser::RdbCodec,
    resp::{parser::RespCodec, types::RespType},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    dir: Option<String>,
    #[arg(long)]
    dbfilename: Option<String>,
}

async fn send_frame(framed: &mut Framed<TcpStream, RespCodec>, resp: RespType) {
    if let Err(e) = framed.send(resp).await {
        eprintln!("cannot send frame: {e}");
    }
}

async fn handle_tcp_stream(stream: TcpStream) {
    let mut framed = Framed::new(stream, RespCodec::new());

    while let Some(parse_result) = framed.next().await {
        match parse_result {
            Ok(resp_value) => {
                let maybe_cmd = Processor::exec_from_resp(resp_value.clone()).await;

                match maybe_cmd {
                    Ok(cmd) => send_frame(&mut framed, cmd).await,
                    Err(e) => {
                        eprintln!(
                            "cannot parse command from valid resp type: {resp_value:?}: {e:?}"
                        );

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

async fn run_infinite_listener() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_tcp_stream(stream));
            }
            Err(e) => eprintln!("cannot accept stream: {e}"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut config = CONFIG.write().await;

    config.dir = args.dir;
    config.dbfilename = args.dbfilename;

    let (Some(dir), Some(dbfilename)) = (&config.dir, &config.dbfilename) else {
        drop(config);

        return run_infinite_listener().await;
    };

    let rdb_path = Path::new(&dir).join(dbfilename);

    if rdb_path.exists() {
        let commands_hash_map = HASH_MAP.write().await;

        let rdb_file_stream = File::open(rdb_path).await?;

        let rdb_codec = RdbCodec::new(commands_hash_map);
        let mut framed = Framed::new(rdb_file_stream, rdb_codec);

        match framed.next().await {
            Some(Ok(_)) => println!("rdb file was parsed successfully"),
            Some(Err(e)) => panic!("cannot parse rdb file: {e}"),
            None => panic!("cannot parse rdb file: unknown error"),
        }
    }

    drop(config);

    run_infinite_listener().await
}
