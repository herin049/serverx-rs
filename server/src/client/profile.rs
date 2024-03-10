use std::fmt::{Debug, Display, Formatter};

use uuid::Uuid;

pub const MAX_USERNAME_LEN: usize = 16;

#[derive(Clone, Debug)]
pub struct Profile {
    pub name: String,
    pub uuid: Uuid,
}

impl TryFrom<(String, Uuid)> for Profile {
    type Error = ProfileErr;

    fn try_from(value: (String, Uuid)) -> Result<Self, Self::Error> {
        if value.0.is_empty() || value.0.len() > MAX_USERNAME_LEN {
            Err(ProfileErr::InvalidUsername(value.0))
        } else {
            Ok(Self {
                name: value.0,
                uuid: value.1,
            })
        }
    }
}

pub enum ProfileErr {
    InvalidUsername(String),
}

impl Debug for ProfileErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileErr::InvalidUsername(name) => write!(f, "invalid username \"{}\"", name),
        }
    }
}

impl Display for ProfileErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Debug>::fmt(self, f)
    }
}
