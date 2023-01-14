use common::NetworkID;

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
        unimplemented!()
    }

    fn respond_to_challenge(
        &mut self,
        target_id: NetworkID,
        response: bool,
    ) -> Result<(), ClientError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
