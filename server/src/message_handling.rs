use common::{messages::ServerMessage, validation::validate_username, GameArchetype, NetworkID};
use crossbeam_channel::Sender;
use laminar::Packet;
use legion::systems::CommandBuffer;
use log::info;

use crate::{ClientInfo, ClientList, NetworkedEntities, PlayerInfo};

pub fn handle_connect_message(
    username: &str,
    next_id: &mut usize,
    clients: &mut ClientList,
    packet: &Packet,
    sender: &Sender<Packet>,
    networked_entities: &mut NetworkedEntities,
    commands: &mut CommandBuffer,
) {
    info!("{username} is attempting to connect...");

    // TODO: Also check for duplicates
    let validation_result = validate_username(username);
    if validation_result.is_ok() {
        info!("Connection accepted!");
        let txt = format!("{username} has connected");

        // Connect user successfully
        let player_id = NetworkID::new(*next_id);
        *next_id += 1;
        clients
            .addr_map
            .insert(packet.addr(), ClientInfo::new(username, player_id));

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
                let msg_packet = Packet::reliable_unordered(*addr, msg.to_payload());
                sender.send(msg_packet).expect("This should send.");
            } else {
                // Spawn remote player
                let msg = ServerMessage::SpawnNetworkedEntity(
                    player_id,
                    common::GameArchetype::Player,
                    false,
                );
                let msg_packet = Packet::reliable_unordered(*addr, msg.to_payload());
                sender.send(msg_packet).expect("This should send.");
            }
        });

        let e = commands.push((GameArchetype::Player, PlayerInfo(username.to_string())));
        networked_entities
            .0
            .insert(player_id, (e, GameArchetype::Player));

        let msg = ServerMessage::SendMessage("SERVER".to_string(), txt);
        clients.all_addresses().iter().for_each(|addr| {
            let msg_packet = Packet::unreliable(*addr, msg.to_payload());
            sender.send(msg_packet).expect("This should send.");
        });
    } else {
        // Disallowed username. Send a rejection message.
        info!("Rejecting Invalid Username");

        let msg =
            ServerMessage::DisconnectClient(common::messages::DisconnectReason::InvalidUsername);
        let addr = packet.addr();
        let msg_packet = Packet::reliable_unordered(addr, msg.to_payload());
        sender.send(msg_packet).expect("This should send.");
    }
}
