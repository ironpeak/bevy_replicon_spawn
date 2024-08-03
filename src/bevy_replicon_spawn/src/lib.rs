pub mod prelude {
    pub(crate) use bevy_app::prelude::*;
    pub(crate) use bevy_ecs::prelude::*;

    pub use crate::{BevyRepliconSpawnPlugin, SpawnEvent};
}

use crate::prelude::*;

pub struct BevyRepliconSpawnPlugin;

impl Plugin for BevyRepliconSpawnPlugin {
    fn build(&self, _: &mut App) {}
}

#[derive(Event)]
pub struct SpawnEvent<T: Component> {
    pub entity: Entity,
    pub data: T,
}
