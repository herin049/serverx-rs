pub use crate::{blocks::*, properties::*};

include!("generated/states.rs");

impl Default for BlockState {
    fn default() -> Self {
        BlockState(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{blocks::Block, states::BlockState};

    #[test]
    fn test() {
        let bs = BlockState::try_from(234).unwrap();
        println!("{:?}", bs);
        let b: Block = bs.into();
        println!("{:?}", b);
        let bs1 = BlockState::from(b);
        println!("{:?}", bs1);
    }
}
