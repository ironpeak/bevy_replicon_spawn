pub mod prelude {
    pub(crate) use bevy_app::prelude::*;
    pub(crate) use bevy_ecs::{prelude::*, system::EntityCommands};
    pub(crate) use bevy_replicon::prelude::*;
    pub(crate) use bevy_replicon_spawn::prelude::*;
    pub(crate) use glam::Vec2;
    pub(crate) use serde::{Deserialize, Serialize};
}

use crate::prelude::*;

#[derive(Component)]
pub struct Health {
    pub position: Vec2,
}

#[derive(Component, Serialize, Deserialize)]
pub struct SpawnPlayerEventComponent {
    pub position: Vec2,
}

#[derive(Event, SpawnContext)]
#[modifier(
    component = SpawnPlayerEventComponent,
    spawner = spawner,
)]
pub struct SpawnPlayerEventContext<'w, 's> {
    pub q_health: Query<'w, 's, &'static Health>,
}

fn spawner<'w, 's>(
    _commands: EntityCommands,
    _context: &mut SpawnPlayerEventContext<'w, 's>,
    _event: &SpawnPlayerEventComponent,
) {
}
