use crate::common::protocol;

use rand::Rng;

pub struct MemoryAccount {
    id: protocol::Id,
    alter_ids: Vec<protocol::Id>,
    security: protocol::SecurityType,
}

impl MemoryAccount {
    // AnyValidID returns an ID that is either the main ID or one of the alternative IDs if any.
    // func (a *MemoryAccount) AnyValidID() *protocol.ID {
    // 	if len(a.AlterIDs) == 0 {
    // 		return a.ID
    // 	}
    // 	return a.AlterIDs[dice.Roll(len(a.AlterIDs))]
    // }

    pub fn any_valid_id(&self) -> &protocol::Id {
        if self.alter_ids.is_empty() {
            return &self.id;
        }

        &self.alter_ids[rand::thread_rng().gen_range(0..self.alter_ids.len())]
    }
}
