pub mod prelude {
    pub(crate) use bevy_app::prelude::*;
    pub(crate) use bevy_ecs::prelude::*;
    pub(crate) use bevy_replicon_spawn::prelude::*;
}

use crate::prelude::*;

#[derive(Event)]
pub struct AttackEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: u32,
}

#[derive(Event, ServerSpawnEvent)]
#[modifier(
    input = AttackEvent,
    metadata = Metadata,
    priority = Priority,
    component = Modifier,
    output = DamageEvent
)]
pub(crate) struct AttackEventContext {}
