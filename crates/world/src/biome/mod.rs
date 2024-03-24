include!("generated/biomes.rs");

impl Default for Biome {
    fn default() -> Self {
        Biome::Plains
    }
}

#[cfg(test)]
mod tests {
    use crate::biome::Biome;

    #[test]
    fn test() {
        let b = Biome::ColdOcean;
        println!("{:?} {} {} {}", b, b.name(), b.id(), b.has_precipitation());
    }
}
