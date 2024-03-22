use std::any::TypeId;

pub trait TypeTuple {
    type TypeIdArray: AsMut<[TypeId]>
        + AsRef<[TypeId]>
        + Into<Box<[TypeId]>>
        + IntoIterator<Item = TypeId>;
    fn type_ids() -> Self::TypeIdArray;
}

impl TypeTuple for () {
    type TypeIdArray = [TypeId; 0];

    fn type_ids() -> Self::TypeIdArray {
        []
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::tuple::type_tuple::TypeTuple;

    #[test]
    fn test() {
        println!("{:?}", <(i32, i64, f32, f64)>::type_ids());
    }
}
