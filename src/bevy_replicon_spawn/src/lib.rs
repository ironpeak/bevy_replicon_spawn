pub mod prelude {
    pub(crate) use bevy_app::prelude::*;
    pub(crate) use bevy_ecs::{prelude::*, system::EntityCommands};
    pub(crate) use bevy_replicon::prelude::*;
    pub use bevy_replicon_spawn_macros::SpawnContext;

    pub use crate::{context::SpawnContext, RepliconSpawnAppExt, SpawnEvent};
}

use crate::prelude::*;

mod context;

pub trait RepliconSpawnAppExt {
    fn replicate_spawn<T>(&mut self, spawn: fn(EntityCommands, &T)) -> &mut Self
    where
        T: Component + Clone;
}

impl RepliconSpawnAppExt for App {
    fn replicate_spawn<T>(&mut self, spawn: fn(EntityCommands, &T)) -> &mut Self
    where
        T: Component + Clone,
    {
        self.add_event::<SpawnEvent<T>>();
        self.insert_resource(SpawnEventResource::<T> { spawn });
        self.add_systems(Update, system::<T>);
        self
    }
}

fn system<T: Component + Clone>(
    mut commands: Commands,
    resource: Res<SpawnEventResource<T>>,
    query: Query<(Entity, &T), (Added<T>, Added<Replicated>)>,
    mut events: EventWriter<SpawnEvent<T>>,
) {
    for (entity, event) in &query {
        (resource.spawn)(commands.entity(entity), event);
        events.send(SpawnEvent {
            entity,
            data: event.clone(),
        });
    }
}

#[derive(Resource)]
struct SpawnEventResource<T: Component> {
    pub spawn: fn(EntityCommands, &T),
}

#[derive(Event)]
pub struct SpawnEvent<T: Component> {
    pub entity: Entity,
    pub data: T,
}
