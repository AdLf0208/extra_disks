//! Extra Disks for Horizon: the 1024k item storage.
//!
//! Extra Disks (see `FINDINGS.md`) adds storage disks and blocks for Refined
//! Storage. This port carries the first rung, **1024k**: one block that holds
//! 1,048,576 items in up to 256 resources, and one disk that carries the same
//! storage in its item component. The capacity rule, the resource identity and
//! the saved encoding are [`runtime`]'s — a plain module with no `mod_api`
//! types, so `cargo test` exercises them on its own.
//!
//! The package is one flat `.hmod` carrying both halves: `assets/` and `data/`
//! declare the block, the disk item and the creative tab, and this file is the
//! code that makes them work. Upstream both halves are a Refined Storage network
//! node with a GUI; this port has no network and no menu, so the block is used by
//! right-click and the `/extradisks` commands, and the disk by
//! `/extradisks disk …`. Those gaps are listed in `notes.md` and are the honest
//! maximum at this pin.
pub mod codec;
pub mod commands;
pub mod disk;
pub mod format;
pub mod runtime;
pub mod storage_block;

use mod_api::{
    self, BreakBlockEvent, ComponentTypesEvent, Flow, Hooks, HostSide, InteractionResult,
    LogicalSide, RegisterCommandsEvent, Registrar, RegistrationContext, RightClickBlockEvent,
    RunModCommandEvent, ServerHost,
};

/// The mod has no session state: every storage's state is its own saved
/// compound or item component.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the block entity type the storage block uses. Upstream the block
    /// is a `StorageBlock` with its own `BlockEntity`; here the type is held by
    /// the one block of this slice and starts with an empty saved compound,
    /// which is where the storage lives (`storage_block::SAVED_KEY`).
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        // The refusal is the game's to report at load; a mod cannot do anything
        // useful with a key it did not get, and the block then keeps no storage.
        let _ = r
            .block_entity_type("item_storage")
            .blocks(&["1024k_item_storage_block"])
            .build();
    }

    /// Before content loads: register `extradisks:storage`, the component that
    /// carries a disk's contents in the item.
    fn component_types(&self, _cx: &RegistrationContext, event: &mut ComponentTypesEvent<'_>) {
        event.types.push(disk::COMPONENT.to_owned());
    }

    /// A held stack onto the storage block goes in (what fits of it); an empty
    /// hand takes one stack out. Every other block's click is left alone.
    fn right_click_block(&self, host: HostSide, event: &mut RightClickBlockEvent<'_>) -> Flow {
        let HostSide::Server(host) = host else {
            return Flow::Continue;
        };
        // The click is heard on the clicking client too; only the server may
        // move items, and only once.
        if event.click.side != LogicalSide::Server {
            return Flow::Continue;
        }
        if !storage_block::on_right_click(host, event.click) {
            return Flow::Continue;
        }
        // Consume the click: the storage block's own use is the interaction, so
        // neither the block's nor the held item's default use runs after it.
        *event.cancellation_result = InteractionResult::Success;
        Flow::Cancel
    }

    /// A player breaks the storage block: its contents go to the breaker, so a
    /// filled block never turns into a plain block item and loses them. The
    /// break is refused only when the breaker cannot be reached at all, and the
    /// block then keeps what did not leave.
    fn break_block(&self, host: &mut dyn ServerHost, event: &mut BreakBlockEvent<'_>) -> Flow {
        if storage_block::on_break(host, event.pos) {
            Flow::Continue
        } else {
            Flow::Cancel
        }
    }

    /// `/extradisks info|insert|extract` and `/extradisks disk …`.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        commands::register(event.commands);
    }

    /// The bodies of those commands; one that is not ours is left to the game.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        if commands::run(host, event.command) {
            Flow::Cancel
        } else {
            Flow::Continue
        }
    }
}
