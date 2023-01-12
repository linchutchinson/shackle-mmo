use std::{collections::HashMap, net::SocketAddr, thread, time::Duration};

use common::{
    validation::validate_username, ClientMessage, GameArchetype, NetworkID, ServerMessage,
    PLAY_AREA_SIZE,
};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use legion::{system, Resources, Schedule, World};
use log::{error, info};

pub fn server() -> Result<(), ErrorKind> {
    let addr = "0.0.0.0:27008";
    println!("Listening at port 27008");
    let mut socket = Socket::bind(addr)?;

    let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
    let _thread = thread::spawn(move || socket.start_polling());

    let clients = ClientList::new();

    let mut world = World::default();
    let mut resources = Resources::default();

    resources.insert(sender);
    resources.insert(receiver);
    resources.insert(clients);
    resources.insert(NetworkedEntities(HashMap::new()));

    let mut schedule = build_schedule();

    loop {
        schedule.execute(&mut world, &mut resources);
        thread::sleep(Duration::from_millis(16));
    }
}

fn build_schedule() -> Schedule {
    Schedule::builder()
        .add_system(parse_incoming_packets_system(0))
        .build()
}

struct ClientList {
    addr_map: HashMap<SocketAddr, ClientInfo>,
}

impl ClientList {
    fn new() -> Self {
        Self {
            addr_map: HashMap::new(),
        }
    }

    fn all_addresses(&self) -> Vec<SocketAddr> {
        let keys = self.addr_map.clone().into_keys();
        keys.collect()
    }
}

#[derive(Clone)]
struct ClientInfo {
    username: String,
    player_id: NetworkID,
}

struct NetworkedEntities(HashMap<NetworkID, GameArchetype>);

#[system]
fn parse_incoming_packets(
    #[state] next_id: &mut usize,
    #[resource] receiver: &mut Receiver<SocketEvent>,
    #[resource] sender: &mut Sender<Packet>,
    #[resource] clients: &mut ClientList,
    #[resource] networked_entities: &mut NetworkedEntities,
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
                            let txt = format!("{username} has connected");

                            // Connect user successfully
                            let player_id = NetworkID::new(*next_id);
                            *next_id += 1;
                            clients.addr_map.insert(
                                packet.addr(),
                                ClientInfo {
                                    username,
                                    player_id,
                                },
                            );

                            let msg = ServerMessage::ConnectionAccepted;
                            let addr = packet.addr();
                            let msg_packet = Packet::reliable_unordered(addr, msg.to_payload());
                            sender.send(msg_packet).expect("This should send.");

                            clients.all_addresses().iter().for_each(|addr| {
                                if *addr == packet.addr() {
                                    // Spawn owned player
                                    let msg = ServerMessage::SpawnNetworkedEntity(
                                        player_id,
                                        common::GameArchetype::Player,
                                        true,
                                    );
                                    let msg_packet =
                                        Packet::reliable_unordered(*addr, msg.to_payload());
                                    sender.send(msg_packet).expect("This should send.");
                                } else {
                                    // Spawn remote player
                                    let msg = ServerMessage::SpawnNetworkedEntity(
                                        player_id,
                                        common::GameArchetype::Player,
                                        false,
                                    );
                                    let msg_packet =
                                        Packet::reliable_unordered(*addr, msg.to_payload());
                                    sender.send(msg_packet).expect("This should send.");
                                }
                            });

                            networked_entities
                                .0
                                .insert(player_id, GameArchetype::Player);

                            let msg = ServerMessage::SendMessage("SERVER".to_string(), txt);
                            clients.all_addresses().iter().for_each(|addr| {
                                let msg_packet = Packet::unreliable(*addr, msg.to_payload());
                                sender.send(msg_packet).expect("This should send.");
                            });
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
                    ClientMessage::MoveTo(pos) => {
                        if let Some(client_info) = clients.addr_map.get(&packet.addr()) {
                            let mut clamped_pos = pos;
                            clamped_pos.x = clamped_pos.x.clamp(0.0, PLAY_AREA_SIZE.x);
                            clamped_pos.y = clamped_pos.y.clamp(0.0, PLAY_AREA_SIZE.y);

                            let msg = ServerMessage::RepositionNetworkedEntity(
                                client_info.player_id,
                                clamped_pos,
                            );
                            clients
                                .all_addresses()
                                .iter()
                                .filter(|addr| **addr != packet.addr())
                                .for_each(|addr| {
                                    let msg_packet = Packet::unreliable(*addr, msg.to_payload());
                                    sender.send(msg_packet).expect("This should send.");
                                });

                            // Lock the active player in if they try to go out of bounds.
                            if clamped_pos != pos {
                                let msg_packet =
                                    Packet::unreliable(packet.addr(), msg.to_payload());
                                sender.send(msg_packet).expect("This should send.");
                            }
                        } else {
                            error!("Someone attempted to send a move packet without having properly connected...");
                        }
                    }
                    ClientMessage::Disconnect => {
                        if let Some(client_info) = clients.addr_map.remove(&packet.addr()) {
                            println!("{} has disconnected", client_info.username);
                        }
                    }
                    ClientMessage::RequestArchetype(id) => {
                        if let Some(_) = clients.addr_map.get(&packet.addr()) {
                            if let Some(_archetype) = networked_entities.0.get(&NetworkID::new(id))
                            {
                                let msg = ServerMessage::SpawnNetworkedEntity(
                                    NetworkID::new(id),
                                    common::GameArchetype::Player,
                                    false,
                                );

                                let msg_packet =
                                    Packet::unreliable(packet.addr(), msg.to_payload());
                                sender.send(msg_packet).expect("This should send.");
                            } else {
                                error!("Requested an entity ID that doesn't exist. {id}");
                            }
                        } else {
                            error!("Someone attempted to send a move packet without having properly connected...");
                        }
                    }
                    ClientMessage::SendMessage(msg) => {
                        if let Some(client_info) = clients.addr_map.get(&packet.addr()) {
                            info!("CHAT - {}: {msg}", client_info.username.to_owned());
                            let msg =
                                ServerMessage::SendMessage(client_info.username.to_owned(), msg);
                            clients.all_addresses().iter().for_each(|addr| {
                                let msg_packet = Packet::unreliable(*addr, msg.to_payload());
                                sender.send(msg_packet).expect("This should send.");
                            });
                        } else {
                            error!("Someone attempted to send a message packet without having properly connected...");
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
