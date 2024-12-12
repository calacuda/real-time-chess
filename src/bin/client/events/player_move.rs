use std::time::Duration;

use bevy::prelude::*;
use real_time_chess::Location;

#[derive(Debug, Clone, Event)]
pub struct PlayerMoveNotif {
    pub from: Location,
    pub to: Location,
    pub cooldown: Duration,
}
