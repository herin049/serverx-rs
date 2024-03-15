use std::{fs::File, io, path::Path};

use serverx_core::{nbt, nbt::decode::NbtDecodeErr};
use tracing::instrument;

pub struct Resources {
    pub registry_data: nbt::Tag,
}

pub enum LoadResourcesErr {
    IoErr(io::Error),
    NbtErr(NbtDecodeErr),
}

pub fn load_registry_data(resource_path: &Path) -> Result<nbt::Tag, LoadResourcesErr> {
    let registry_path = resource_path.join("registries.nbt");
    let mut file = File::open(registry_path).map_err(|err| LoadResourcesErr::IoErr(err))?;
    nbt::io::read_tag(&mut file).map_err(|err| LoadResourcesErr::NbtErr(err))
}

#[instrument]
pub fn load(resource_path: &Path) -> Result<Resources, LoadResourcesErr> {
    tracing::debug!("loading resources");
    let registry_data = load_registry_data(resource_path)?;
    Ok(Resources { registry_data })
}
