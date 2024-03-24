use crate::properties::*;

include!("generated/blocks.rs");

impl Default for Block {
    fn default() -> Self {
        Self::Air
    }
}
