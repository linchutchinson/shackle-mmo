mod message_handling;

use std::{collections::HashMap, net::SocketAddr, thread, time::Duration};

use common::{
    ClientMessage, InfoRequestType, InfoSendType, NetworkID, ServerMessage, PLAY_AREA_SIZE,
};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use legion::{
    system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore, Query, Resources,
    Schedule, World,
};
use log::{error, info};

use crate::message_handling::handle_connect_message;

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
        .flush()
        .add_system(send_player_info_system())
        .build()
}

pub struct ClientList {
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

pub struct NetworkedEntities(HashMap<NetworkID, Entity>);

#[system]
fn parse_incoming_packets(
    #[state] next_id: &mut usize,
    #[resource] receiver: &mut Receiver<SocketEvent>,
    #[resource] sender: &mut Sender<Packet>,
    #[resource] clients: &mut ClientList,
    #[resource] networked_entities: &mut NetworkedEntities,
    commands: &mut CommandBuffer,
) {
    receiver.try_iter().for_each(|event|  {
        match event {
            SocketEvent::Packet(packet) => {
                let msg = ClientMessage::from_payload(packet.payload());

                if msg.is_err() {
                    error!("Received an invalid message from ip {}. This may be a result of malicious activity.\nErr: {}", packet.addr(), msg.as_ref().unwrap_err());
                }

                match msg.unwrap() {
                    ClientMessage::Connect(username) => {
                        handle_connect_message(
                            &username,
                            next_id,
                            clients,
                            &packet,
                            sender,
                            networked_entities,
                            commands,
                        );
                    }
                    ClientMessage::MoveTo(pos) => {
                        if let Some(client_info) = clients.addr_map.get(&packet.addr()) {
                            let mut clamped_pos = pos;
                            clamped_pos.x = clamped_pos.x.clamp(0.0, PLAY_AREA_SIZE.x);
                            clamped_pos.y = clamped_pos.y.clamp(0.0, PLAY_AREA_SIZE.y);

                            let msg = ServerMessage::SendNetworkedEntityInfo(
                                client_info.player_id,
                                InfoSendType::Position(clamped_pos),
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
                            if let Some(_archetype) = networked_entities.0.get(&id) {
                                let msg = ServerMessage::SpawnNetworkedEntity(
                                    id,
                                    common::GameArchetype::Player,
                                    false,
                                );

                                let msg_packet =
                                    Packet::unreliable(packet.addr(), msg.to_payload());
                                sender.send(msg_packet).expect("This should send.");
                            } else {
                                error!("Requested an entity ID that doesn't exist. {id:?}");
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
                    ClientMessage::RequestEntityInfo(id, info) => {
                        if let Some(_) = clients.addr_map.get(&packet.addr()) {
                            if let Some(_) = networked_entities.0.get(&id) {
                                info!("Marking an entity to send its info to a client with ID {id:?}");
                                commands.push((SendInfoRequest(id, packet.addr(), info),));
                            } else {
                                error!("Requested info for an entity that doesn't exist. {id:?}");
                            }
                        } else {
                            error!("Someone requested an entity's info without being properly connected.");
                        }
                    }
                }
            }
            _ => {}
        }
    });
}

/// Send the requested info of the specified entity to the client
/// at the given address.
struct SendInfoRequest(NetworkID, SocketAddr, InfoRequestType);
struct PlayerInfo(String);

#[system]
#[read_component(SendInfoRequest)]
#[read_component(PlayerInfo)]
fn send_player_info(
    query: &mut Query<(Entity, &SendInfoRequest)>,
    world: &mut SubWorld,
    #[resource] sender: &Sender<Packet>,
    #[resource] networked_entities: &mut NetworkedEntities,
    commands: &mut CommandBuffer,
) {
    query
        .iter(world)
        .for_each(|(message_entity, send_request)| {
            // FIXME: Right now this just silently fails if a send request is created incorrectly
            if let Some(e) = networked_entities.0.get(&send_request.0) {
                if let Ok(entry) = world.entry_ref(*e) {
                    if let Ok(info) = entry.get_component::<PlayerInfo>() {
                        let msg = ServerMessage::SendNetworkedEntityInfo(
                            send_request.0,
                            InfoSendType::Identity(info.0.clone()),
                        );

                        let packet = Packet::unreliable(send_request.1, msg.to_payload());

                        sender.send(packet).expect("This should send.");
                    }
                }
            }

            // TODO: Marking this as a handled message to be cleaned later
            // would allow for multiple types of messages being parsed off of one entity.
            commands.remove(*message_entity);
        });
}
