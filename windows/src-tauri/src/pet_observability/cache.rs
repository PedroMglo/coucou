//! Single-flight metadata cache. No provider, filesystem or GUI access.
use std::{collections::BTreeMap, time::{Duration, Instant}};
use super::core::{Inventory, InventoryInput};

const REFRESH_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Default)]
pub struct Cache { slots: BTreeMap<String, Slot> }
struct Slot {
    inventory: Inventory,
    attempted: Option<Instant>,
    in_flight: bool,
    generation: u64,
}

pub struct Ticket { provider: String, generation: u64 }

impl Cache {
    /// A failed refresh also consumes the cooldown, preventing repeated native
    /// requests from multiple cards or open surfaces.
    pub fn begin(&mut self, provider: &str, scope: &str, now: Instant) -> Option<Ticket> {
        let slot = self.slots.entry(provider.into()).or_insert_with(|| Slot {
            inventory: Inventory::build(InventoryInput::unavailable(scope, "notObserved"), 0, 0, false),
            attempted: None, in_flight: false, generation: 0 });
        if slot.in_flight || slot.attempted.is_some_and(|t| now.saturating_duration_since(t) < REFRESH_INTERVAL) {
            return None;
        }
        slot.attempted = Some(now);
        slot.in_flight = true;
        slot.generation = slot.generation.wrapping_add(1);
        Some(Ticket { provider: provider.into(), generation: slot.generation })
    }

    pub fn snapshot(&self, provider: &str, scope: &str, now: u64, max_age: u64) -> Inventory {
        let mut inventory = self.slots.get(provider).map(|slot| {
            let mut inventory = slot.inventory.clone();
            inventory.refreshing = slot.in_flight;
            inventory
        }).unwrap_or_else(|| Inventory::build(InventoryInput::unavailable(scope, "notObserved"), now, max_age, false));
        inventory.refresh_freshness(now, max_age);
        inventory
    }

    pub fn finish(&mut self, ticket: Ticket, result: Result<Inventory, String>) {
        let Some(slot) = self.slots.get_mut(&ticket.provider) else { return };
        if slot.generation != ticket.generation { return; }
        slot.in_flight = false;
        match result {
            Ok(inventory) => slot.inventory = inventory,
            Err(_) => {
                // Preserve the previous collection time and observed rows.
                // A new error must not make old evidence look freshly fetched.
                slot.inventory.mark_refresh_failed();
            }
        }
    }
}
