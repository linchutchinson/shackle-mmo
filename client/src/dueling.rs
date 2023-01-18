use common::{messages::ClientMessage, NetworkID};

use crate::{connection::ConnectionInterface, Client, ClientError};

pub trait DuelingClient {
    fn send_challenge(&mut self, target_id: NetworkID) -> Result<(), ClientError>;

    fn respond_to_challenge(
        &mut self,
        target_id: NetworkID,
        response: bool,
    ) -> Result<(), ClientError>;
}

impl<T: ConnectionInterface> DuelingClient for Client<T> {
    fn send_challenge(&mut self, target_id: NetworkID) -> Result<(), ClientError> {
        let conn = self.get_connection_mut()?;
        conn.send_message(ClientMessage::IssueChallenge(target_id))?;
        Ok(())
    }

    fn respond_to_challenge(
        &mut self,
        target_id: NetworkID,
        response: bool,
    ) -> Result<(), ClientError> {
        let conn = self.get_connection_mut()?;
        conn.send_message(ClientMessage::RespondToChallenge(target_id, response))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common::messages::ClientMessage;

    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_send_challenge() {
        let mut client = TestClient::already_connected();
        let target_id = NetworkID::new(1);
        client
            .send_challenge(target_id.clone())
            .expect("This should work.");
        let binding = client.get_sent_messages();
        let last_message = binding.last().unwrap();
        assert_eq!(*last_message, ClientMessage::IssueChallenge(target_id))
    }
}
