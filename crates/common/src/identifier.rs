use std::{
    fmt::{Debug, Display, Formatter},
    io::{Read, Seek, Write},
};

#[derive(Clone)]
pub struct Identifier {
    str: String,
    delim_pos: usize,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str)
    }
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

impl Identifier {
    pub fn new(str: String, delim_pos: usize) -> Self {
        Self { str, delim_pos }
    }

    pub fn namespace(&self) -> &str {
        &self.str[..self.delim_pos]
    }

    pub fn path(&self) -> &str {
        &self.str[(self.delim_pos + 1)..]
    }

    pub fn as_str(&self) -> &str {
        self.str.as_str()
    }

    pub fn string_ref(&self) -> &String {
        &self.str
    }
}

pub enum TryFromStrErr {
    MissingTag,
    InvalidNamespace,
    InvalidPath,
}

impl Display for TryFromStrErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TryFromStrErr::MissingTag => write!(f, "missing tag"),
            TryFromStrErr::InvalidNamespace => write!(f, "invalid identifier namespace"),
            TryFromStrErr::InvalidPath => write!(f, "invalid identifier path"),
        }
    }
}

impl Debug for TryFromStrErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Display>::fmt(self, f)
    }
}

impl TryFrom<&str> for Identifier {
    type Error = TryFromStrErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(delim_pos) = value.chars().position(|c| c == ':') {
            if !is_valid_namespace(&value[..delim_pos]) {
                Err(TryFromStrErr::InvalidNamespace)
            } else if !is_valid_path(&value[(delim_pos + 1)..]) {
                Err(TryFromStrErr::InvalidPath)
            } else {
                Ok(Self {
                    str: value.to_owned(),
                    delim_pos,
                })
            }
        } else if !is_valid_path(value) {
            Err(TryFromStrErr::InvalidPath)
        } else {
            let mut str = "minecraft:".to_string();
            let delim_pos = str.len() - 1;
            str.push_str(value);
            Ok(Self { str, delim_pos })
        }
    }
}

fn is_valid_namespace(namespace: &str) -> bool {
    namespace.len() > 0
        && namespace.chars().all(|c| {
            c == '_' || c == '-' || (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || c == '.'
        })
}

fn is_valid_path(path: &str) -> bool {
    path.len() > 0
        && path.chars().all(|c| {
            c == '_'
                || c == '-'
                || (c >= 'a' && c <= 'z')
                || (c >= '0' && c <= '9')
                || c == '/'
                || c == '.'
        })
}
