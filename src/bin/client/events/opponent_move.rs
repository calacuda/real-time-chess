use bevy::prelude::*;
use real_time_chess::Location;
use std::time::Duration;

#[derive(Debug, Clone, Event)]
pub struct OpponentMoveNotif {
    pub from: Location,
    pub to: Location,
    pub cooldown: Duration,
}
