use bevy::prelude::*;

use crate::country::resources::RelationStatus;

#[derive(Message)]
pub struct ChangeRelationMessage {
    pub country_a_idx: usize,
    pub country_b_idx: usize,
    pub relation: RelationStatus,
}

#[derive(Message)]
pub struct ProposePeaceMessage {
    pub from: usize,
    pub to: usize,
}

#[derive(Message)]
pub struct AcceptPeaceMessage {
    pub from: usize,
    pub to: usize,
}

#[derive(Message)]
pub struct RejectPeaceMessage {
    pub from: usize,
    pub to: usize,
}
