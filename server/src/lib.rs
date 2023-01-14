mod message_handling;

use std::{collections::HashMap, net::SocketAddr, thread, time::Duration};

use common::{
    messages::{ClientMessage, InfoRequestType, InfoSendType, ServerMessage},
    GameArchetype, NetworkID, PLAY_AREA_SIZE,
};
use crossbeam_channel::{Receiver, Sender};
use laminar::{Config, ErrorKind, Packet, Socket, SocketEvent};
use legion::{
    system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore, Query, Resources,
    Schedule, World,
};
use log::{error, info};

use crate::message_handling::handle_connect_message;

fn server_socket_config() -> Config {
    Config {
        idle_connection_timeout: Duration::from_secs(60),
        ..Default::default()
    }
}

pub fn server() -> Result<(), ErrorKind> {
    let addr = "0.0.0.0:27008";
    println!("Listening at port 27008");
    let mut socket = Socket::bind_with_config(addr, server_socket_config())?;

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

pub struct NetworkedEntities(HashMap<NetworkID, (Entity, GameArchetype)>);

/// Send the provided packet and write to log in the event of an
/// error.
fn logged_send(sender: &mut Sender<Packet>, packet: Packet) {
    let result = sender.send(packet);

    if let Err(err) = result {
        log::error!("Encountered an error when sending packet: {err}");
    }
}

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
            //TODO: Handle Timeouts
            SocketEvent::Disconnect(addr) => {
                if let Some(client_info) = clients.addr_map.remove(&addr) {
                    info!("{} has disconnected", client_info.username);

                    let id = client_info.player_id;

                    let chat_message = ServerMessage::SendMessage("SERVER".to_string(), format!("{} has disconnected.", client_info.username));
                    let delete_message = ServerMessage::DespawnNetworkedEntity(id);

                    clients.all_addresses().iter().for_each(|addr| {
                        let chat_packet = Packet::reliable_unordered(*addr, chat_message.to_payload());
                        let delete_packet = Packet::reliable_unordered(*addr, delete_message.to_payload());

                        logged_send(sender, chat_packet);
                        logged_send(sender, delete_packet);
                    });
                }
            }
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
                        if clients.addr_map.get(&packet.addr()).is_some() {
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
                        if clients.addr_map.get(&packet.addr()).is_some() {
                            if networked_entities.0.get(&id).is_some() {
                                info!("Marking an entity to send its info to a client with ID {id:?}");
                                commands.push((SendInfoRequest(id, packet.addr(), info),));
                            } else {
                                error!("Requested info for an entity that doesn't exist. {id:?}");
                            }
                        } else {
                            error!("Someone requested an entity's info without being properly connected.");
                        }
                    }
                    ClientMessage::IssueChallenge(_target) => {
                        unimplemented!()
                    }
                    ClientMessage::RespondToChallenge(_target, _accepted) => {
                        unimplemented!()
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
            if let Some((e, _)) = networked_entities.0.get(&send_request.0) {
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
