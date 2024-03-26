mod chunk;

pub struct Game {
    ecs: serverx_ecs::world::World,
}

impl Game {
    pub fn tick(&mut self) {}
}

#[cfg(test)]
mod tests {}
