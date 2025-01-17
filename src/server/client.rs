use crate::protocol::{
    error::ProtoError,
    packets::{Disconnect, Handshake, State},
    PacketReadExtAsync, PacketWriteExtAsync,
};
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Client {
    pub address: SocketAddr,
    pub stream: TcpStream,
    pub data: Handshake,
}

impl Client {
    pub fn destination(&self) -> &str {
        &self.data.server_address
    }

    pub async fn disconnect(mut self, reason: impl Into<String>) {
        if !matches!(self.data.next_state, State::Login) {
            return;
        }

        self.stream
            .write_serialize(Disconnect::new(reason))
            .await
            .ok();
        drop(self.stream)
    }

    pub async fn handshake(
        (mut stream, address): (TcpStream, SocketAddr),
    ) -> Result<Self, ProtoError> {
        let data = stream
            .read_packet()
            .await?
            .deserialize_owned::<Handshake>()?;

        Ok(Client {
            address,
            stream,
            data,
        })
    }
}
