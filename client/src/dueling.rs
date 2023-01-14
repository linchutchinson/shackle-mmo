use common::NetworkID;

use crate::{Client, ClientError};

pub trait DuelingClient {
    fn send_challenge(&mut self, target_id: NetworkID) -> Result<(), ClientError>;
}

impl DuelingClient for Client {
    fn send_challenge(&mut self, target_id: NetworkID) -> Result<(), ClientError> {
        let conn = self.get_connection_mut()?;
        Ok(())
    }
}
