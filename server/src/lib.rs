use std::{collections::HashMap, net::SocketAddr, thread, time::Duration};

use common::{validation::validate_username, ClientMessage, ServerMessage};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use legion::{system, systems::CommandBuffer, Resources, Schedule, World};
use log::{error, info};

pub fn server() -> Result<(), ErrorKind> {
    let addr = "127.0.0.1:12352";
    println!("Listening at port 12352");
    let mut socket = Socket::bind(addr)?;

    let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
    let _thread = thread::spawn(move || socket.start_polling());

    let clients = ClientList::new();

    let mut world = World::default();
    let mut resources = Resources::default();

    resources.insert(sender);
    resources.insert(receiver);
    resources.insert(clients);

    let mut schedule = build_schedule();

    loop {
        schedule.execute(&mut world, &mut resources);
        thread::sleep(Duration::from_millis(16));
    }
}

fn build_schedule() -> Schedule {
    Schedule::builder()
        .add_system(parse_incoming_packets_system())
        .build()
}

struct ClientList {
    addr_map: HashMap<SocketAddr, String>,
}

impl ClientList {
    fn new() -> Self {
        Self {
            addr_map: HashMap::new(),
        }
    }
}

#[system]
fn parse_incoming_packets(
    #[resource] receiver: &mut Receiver<SocketEvent>,
    #[resource] sender: &mut Sender<Packet>,
    #[resource] clients: &mut ClientList,
    commands: &mut CommandBuffer,
) {
    if let Ok(event) = receiver.try_recv() {
        match event {
            SocketEvent::Packet(packet) => {
                let msg = ClientMessage::from_payload(packet.payload());

                if msg.is_err() {
                    error!("Received an invalid message from ip {}. This may be a result of malicious activity.\nErr: {}", packet.addr(), msg.as_ref().unwrap_err());
                }

                match msg.unwrap() {
                    ClientMessage::Connect(username) => {
                        info!("{username} is attempting to connect...");

                        // TODO: Also check for duplicates
                        let validation_result = validate_username(&username);
                        if let Ok(_) = validation_result {
                            info!("Connection accepted!");
                            // Connect user successfully
                            clients.addr_map.insert(packet.addr(), username);

                            let msg = ServerMessage::ConnectionAccepted;
                            let addr = packet.addr();
                            let msg_packet = Packet::reliable_unordered(addr, msg.to_payload());
                            sender.send(msg_packet).expect("This should send.");
                        } else {
                            // Disallowed username. Send a rejection message.
                            info!("Rejecting Invalid Username");

                            let msg = ServerMessage::DisconnectClient(
                                common::DisconnectReason::InvalidUsername,
                            );
                            let addr = packet.addr();
                            let msg_packet = Packet::reliable_unordered(addr, msg.to_payload());
                            sender.send(msg_packet).expect("This should send.");
                        }
                    }
                    ClientMessage::Disconnect => {
                        if let Some(username) = clients.addr_map.remove(&packet.addr()) {
                            println!("{username} has disconnected");
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
