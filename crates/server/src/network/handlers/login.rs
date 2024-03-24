use std::fmt::{Debug, Formatter};

use tokio::net::TcpStream;
use tracing::instrument;
use uuid::Uuid;
use serverx_protocol::io::{AsyncPacketReader, AsyncPacketWriter, PacketReadErr, PacketWriteErr};
use serverx_protocol::packet::ConnectionState::Login;
use serverx_protocol::packet::PacketDirection::{ClientBound, ServerBound};
use serverx_protocol::v765::clientbound::LoginSuccess;
use serverx_protocol::v765::{PacketDecoderImpl, PacketEncoderImpl};
use serverx_protocol::v765::serverbound::{LoginAck, LoginStart};

use crate::client::profile::{Profile, ProfileErr};

#[derive(Debug, Clone)]
pub struct LoginResult {
    pub profile: Profile,
}

pub enum LoginErr {
    WriteErr(PacketWriteErr),
    ReadErr(PacketReadErr),
    InvalidProfile(ProfileErr),
    UnexpectedPacket,
}

impl Debug for LoginErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginErr::WriteErr(err) => write!(f, "write error: {}", err),
            LoginErr::ReadErr(err) => write!(f, "write error: {}", err),
            LoginErr::InvalidProfile(err) => write!(f, "invalid user profile: {}", err),
            LoginErr::UnexpectedPacket => write!(f, "unexpected packet"),
        }
    }
}

#[instrument(skip_all)]
pub async fn handle_login(
    socket: &mut TcpStream,
    reader: &mut AsyncPacketReader,
    writer: &mut AsyncPacketWriter,
) -> Result<LoginResult, LoginErr> {
    let login_start = reader
        .read::<TcpStream, PacketDecoderImpl>(socket, ServerBound, Login)
        .await
        .map_err(|err| LoginErr::ReadErr(err))?
        .into_any()
        .downcast::<LoginStart>()
        .map_err(|_| LoginErr::UnexpectedPacket)?;
    tracing::trace!(?login_start, "received connect start packet");
    let profile = Profile::try_from((login_start.name, login_start.uuid))
        .map_err(|err| LoginErr::InvalidProfile(err))?;
    tracing::debug!(?profile, "loaded user profile");
    let login_success = LoginSuccess {
        uuid: profile.uuid,
        username: profile.name.clone(),
        properties: vec![],
    };
    tracing::trace!(?login_success, "sending connect success packet");
    writer
        .write::<TcpStream, PacketEncoderImpl>(socket, ClientBound, Login, &login_success)
        .await
        .map_err(|err| LoginErr::WriteErr(err))?;
    let _ = reader
        .read::<TcpStream, PacketDecoderImpl>(socket, ServerBound, Login)
        .await
        .map_err(|err| LoginErr::ReadErr(err))?
        .into_any()
        .downcast::<LoginAck>()
        .map_err(|_| LoginErr::UnexpectedPacket)?;
    Ok(LoginResult { profile })
}
