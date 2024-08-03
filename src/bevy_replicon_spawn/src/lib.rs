pub mod prelude {
    pub(crate) use bevy_app::prelude::*;
    pub use bevy_replicon_spawn_macros::{ClientSpawnEvent, ServerSpawnEvent};

    pub use crate::BevyRepliconSpawnPlugin;
}

use crate::prelude::*;

pub struct BevyRepliconSpawnPlugin;

impl Plugin for BevyRepliconSpawnPlugin {
    fn build(&self, _: &mut App) {}
}
