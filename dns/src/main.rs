use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::{net::SocketAddr, net::UdpSocket};

use clap::Parser;

mod dns_message;

use dns_message::definitions::header;
use dns_message::definitions::question;
use dns_message::definitions::response::Response;
use dns_message::definitions::rr;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    resolver: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx): (
        Sender<([u8; 512], SocketAddr)>,
        Receiver<([u8; 512], SocketAddr)>,
    ) = mpsc::channel();

    let args = Args::parse();

    let udp_socket =
        UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address 127.0.0.1:2053");
    udp_socket.set_write_timeout(Some(Duration::from_secs(1)))?;

    let mut buf = [0; 512];

    match args.resolver {
        Some(ref address) => {
            let udp_socket = udp_socket.try_clone()?;
            let resolver_address = address.clone();

            thread::spawn(move || {
                let resolver_socket = UdpSocket::bind("127.0.0.1:2054")
                    .expect("Failed to bind to address 127.0.0.1:2054");

                for (bytes, socket_addr) in rx {
                    let thread_resolver_socket = resolver_socket.try_clone().unwrap();
                    let thread_udp_socket = udp_socket.try_clone().unwrap();
                    let thread_socket_addr = socket_addr.clone();
                    let thread_resolver_address = resolver_address.clone();

                    thread::spawn(move || {
                        let Ok(request) = dns_message::parse_into_req(&bytes)
                            .inspect_err(|e| eprintln!("Error while parsing incoming bytes: {e}"))
                        else {
                            return;
                        };

                        let requests = request.split_into_multiple();

                        let id = requests[0].header.id.to_number().unwrap();

                        let mut number_of_requests_to_resolver = requests.len();

                        for request in &requests {
                            thread_resolver_socket
                                .send_to(&request.to_buf(), &thread_resolver_address)
                                .unwrap();
                        }

                        let mut responses_from_resolver: Vec<Response> = vec![];

                        let mut buf = [0; 512];

                        loop {
                            match thread_resolver_socket.recv_from(&mut buf) {
                                Ok((_, source)) => {
                                    println!(
                                        "Receiving response from source {source}, resolver {thread_resolver_address}, id {id}"
                                    );

                                    let Ok(response) = dns_message::parse_into_res(&buf)
                                        .inspect_err(|e| eprintln!("Cannot parse buf into valid response for id {id}: {e}"))
                                    else { return };

                                    if response.header.id.to_number().unwrap() != id {
                                        println!(
                                            "Skipping id {id}, because of inconsistency with original id from request"
                                        );

                                        continue;
                                    }

                                    number_of_requests_to_resolver -= 1;

                                    responses_from_resolver.push(response);

                                    if number_of_requests_to_resolver > 0 {
                                        continue;
                                    }

                                    println!("Send response back from resolver, id {id}");

                                    let mut header = responses_from_resolver[0].header.clone();

                                    let names: Vec<question::QuestionRecord> =
                                        responses_from_resolver
                                            .iter()
                                            .flat_map(|r| r.question.records.to_vec())
                                            .collect();
                                    let question = question::Question::from_records(names);

                                    header.set_header(header::HeaderField::Qdcount(
                                        question.records.len().try_into().unwrap_or(0),
                                    ));

                                    let rrs: Vec<rr::Rr> = responses_from_resolver
                                        .iter()
                                        .flat_map(|r| r.answer.to_vec())
                                        .collect();

                                    header.set_header(header::HeaderField::Ancount(
                                        rrs.len().try_into().unwrap_or(0),
                                    ));

                                    let response = Response::new(header, question, rrs);

                                    thread_udp_socket
                                        .send_to(&response.to_buf(), thread_socket_addr)
                                        .expect("Failed to forward to response from resolver");

                                    break;
                                }
                                Err(e) => {
                                    eprintln!("Error receiving data from resolver: {e}");

                                    break;
                                }
                            }
                        }
                    });
                }
            });
        }
        _ => {}
    }

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {size} bytes from {source}");

                match args.resolver {
                    Some(_) => {
                        tx.send((buf, source))?;
                    }
                    None => {
                        udp_socket.send_to(&dns_message::reply(&buf)?, source)?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {e}");

                break;
            }
        }
    }

    Ok(())
}
