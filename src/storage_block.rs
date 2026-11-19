//! The 1024k storage block: its saved compound, and what a player does with it.
//!
//! The block is a block entity of the type this mod declares in `register`
//! (`extradisks:item_storage`, held by `extradisks:1024k_item_storage_block`).
//! Its storage lives in the saved compound under one key, as the list of lines
//! [`Storage::encode`] makes; the engine saves and reloads that compound with the
//! chunk, which is what keeps the contents across a relaunch.
//!
//! Upstream the block is a Refined Storage network node with a GUI. This port has
//! no network and no screen for a mod menu, so its interface is the right-click
//! (a held stack goes in, an empty hand takes a stack out), the `/extradisks`
//! commands, and the break (the contents go to the breaker). The capacity rule,
//! the resource identity and the encoding are [`crate::runtime`]'s; this module
//! only moves them in and out of the game.

use mod_api::{
    BlockClick, BlockKey, Hand, Host, ItemStack, LevelRef, PlayerRef, ServerHost, Tag, Text,
};

use crate::codec;
use crate::format::{self, grouped, summary};
use crate::runtime::{Resource, Storage, Variant};

/// The saved compound key a storage block's contents live under.
pub const SAVED_KEY: &str = "ExtradisksStorage";
/// The rungs this slice implements; everything else in the jar is a gap.
const RUNGS: [Variant; 1] = [Variant::K1024];

/// The block id at `pos`, none when it is not loaded.
pub fn block_id_at(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Option<String> {
    if !level.is_loaded(host, pos) {
        return None;
    }
    let block: BlockKey = level.block(host, pos)?;
    Some(block.id(host))
}

/// The rung of the storage block at `pos`, none for any other block.
pub fn rung_at(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Option<Variant> {
    let rung = Variant::from_block_id(&block_id_at(host, level, pos)?)?;
    RUNGS.contains(&rung).then_some(rung)
}

/// The storage a block entity's saved compound holds; empty when there is no
/// block entity or no saved key.
pub fn read(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Storage {
    let saved = host
        .block_entity(level, pos)
        .and_then(|entity| entity.data(host))
        .map(|data| codec::saved_lines(Tag::from(&data).get(SAVED_KEY)))
        .unwrap_or_default();
    Storage::parse(format::capacity(), &saved)
}

/// Write a storage into a block entity's saved compound; an empty one keeps no
/// key at all, so a fresh block entity's compound stays empty.
pub fn write(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3], storage: &Storage) {
    let Some(entity) = host.block_entity(level, pos) else {
        return;
    };
    let Some(mut data) = entity.data(host) else {
        return;
    };
    if storage.is_empty() {
        data.retain(|key, _| key.to_string_lossy() != SAVED_KEY);
    } else {
        data.insert(SAVED_KEY.into(), codec::lines_tag(&storage.encode()));
    }
    // A stale handle (the block went away mid-interaction) writes nothing; the
    // storage then simply did not move, which the caller's feedback states.
    let _ = entity.replace_data(host, data);
}

/// The resource a stack is: its item and its components, without its count.
/// Upstream's `ItemResource` is the same pair, which is why two named diamonds
/// are two resources.
pub fn resource_of(host: &dyn Host, stack: &ItemStack) -> Resource {
    let id = stack.item().id(host);
    let key = codec::component_key(stack.components().map(|components| components.as_nbt()));
    Resource::with_extra(id, key)
}

/// A stack of `resource` at `count`; none when the item id is unknown to this
/// world or `count` is zero.
pub fn stack_of(host: &dyn Host, resource: &Resource, count: u32) -> Option<ItemStack> {
    let id = mod_api::Identifier::parse(resource.id()).ok()?;
    let item = host.item_key(&id)?;
    let mut stack = ItemStack::new(item, count)?;
    if let Some(patch) = codec::patch(resource.extra()) {
        // A patch the game refuses leaves a plain stack: the resource is still
        // the item, only its decoration was damaged.
        let _ = stack.set_components(patch);
    }
    Some(stack)
}

/// The item's own stack size, at least one.
fn stack_size(host: &dyn Host, resource: &Resource) -> u32 {
    stack_of(host, resource, 1)
        .and_then(|stack| host.max_stack_size(stack.item()))
        .unwrap_or(1)
        .max(1)
}

/// Hand up to `count` of `resource` to `player`, answering how many were handed
/// over. `/give` semantics: what the inventory cannot hold is thrown at the
/// player's feet and stays theirs, so nothing is lost and the whole count counts
/// as given.
pub fn give(host: &mut dyn ServerHost, player: PlayerRef, resource: &Resource, count: u64) -> u64 {
    let per = u64::from(stack_size(host, resource));
    let mut given = 0;
    while given < count {
        let piece = (count - given).min(per);
        let Ok(piece) = u32::try_from(piece) else {
            break;
        };
        let Some(stack) = stack_of(host, resource, piece) else {
            break;
        };
        let asked = u64::from(stack.count());
        if player.give_stack(host, stack).is_err() {
            break;
        }
        given += asked;
    }
    given
}

/// Tell `player` one line, as the block's feedback.
fn tell(host: &mut dyn ServerHost, player: PlayerRef, line: String) {
    let _ = player.tell(host, Text::literal(line));
}

/// `" … 1,048,576 free"` for a feedback line.
pub fn free_note(storage: &Storage) -> String {
    format!(
        "; {} of {} free",
        grouped(storage.free()),
        grouped(storage.capacity())
    )
}

/// The hand slot a click came from: the selected hotbar slot for the main hand,
/// 40 for the off hand.
fn hand_slot(host: &dyn Host, player: PlayerRef, hand: Hand) -> Option<u32> {
    if matches!(hand, Hand::Off) {
        return Some(40);
    }
    u32::try_from(player.selected_slot(host)?).ok()
}

/// A player right-clicked a storage block: a held stack goes in (what fits of
/// it), an empty hand takes one stack of the first resource out. Answers whether
/// the click was a storage block's, so the caller can consume it.
pub fn on_right_click(host: &mut dyn ServerHost, click: &BlockClick<'_>) -> bool {
    let Some(level) = host.level(click.dimension.key().id()) else {
        return false;
    };
    let Some(rung) = rung_at(host, level, click.pos) else {
        return false;
    };
    let Some(player) = host.players().acting_player() else {
        return false;
    };
    match click.stack.filter(|stack| stack.count() > 0) {
        Some(stack) => deposit(host, player, level, click, stack, rung),
        None => withdraw(host, player, level, click.pos, rung),
    }
    true
}

/// The held stack goes in, and what did not fit stays in the hand.
///
/// The hand is changed through one inventory write, so an unreadable hand or a
/// refused write leaves the storage as it was: the items are never in both
/// places and never in neither.
fn deposit(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    level: LevelRef,
    click: &BlockClick<'_>,
    stack: &ItemStack,
    rung: Variant,
) {
    let Some(slot) = hand_slot(host, player, click.hand) else {
        tell(
            host,
            player,
            "Extra Disks: the storage block could not reach your hand, so nothing was stored."
                .to_owned(),
        );
        return;
    };
    let resource = resource_of(host, stack);
    let name = stack.item().id(host);
    let mut storage = read(host, level, click.pos);
    let accepted = storage.accepts(&resource);
    let took = storage.insert(resource.clone(), u64::from(stack.count()));
    if took == 0 {
        // Two different refusals: no room left, or the 256-resource bound.
        let line = if accepted {
            format!("The {} storage block is full.", rung.name())
        } else {
            format!(
                "The {} storage block already holds its 256 resources.",
                rung.name()
            )
        };
        tell(host, player, line);
        return;
    }
    let took = u32::try_from(took).unwrap_or(stack.count());
    let left = stack.count().saturating_sub(took);
    let remainder = if left == 0 {
        None
    } else {
        let mut remainder = stack.clone();
        // `left` is at least one, so the count is representable.
        let _ = remainder.set_count(left);
        Some(remainder)
    };
    if player.set_inventory_slot(host, slot, remainder).is_err() {
        // The hand did not change: put back what the storage just took.
        storage.extract(&resource, u64::from(took));
        write(host, level, click.pos, &storage);
        tell(
            host,
            player,
            "Extra Disks: the storage block could not reach your hand, so nothing was stored."
                .to_owned(),
        );
        return;
    }
    write(host, level, click.pos, &storage);
    tell(
        host,
        player,
        format!(
            "Stored {} x {} ({} left in hand){}",
            grouped(u64::from(took)),
            name,
            left,
            free_note(&storage)
        ),
    );
}

/// An empty hand takes one stack of the first resource out.
fn withdraw(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    level: LevelRef,
    pos: [i32; 3],
    rung: Variant,
) {
    let mut storage = read(host, level, pos);
    let Some(entry) = storage.entries().first().cloned() else {
        tell(
            host,
            player,
            format!("The {} storage block is empty.", rung.name()),
        );
        return;
    };
    let want = entry
        .amount
        .min(u64::from(stack_size(host, &entry.resource)));
    let took = storage.extract(&entry.resource, want);
    if took == 0 {
        tell(
            host,
            player,
            format!("The {} storage block is empty.", rung.name()),
        );
        return;
    }
    let given = give(host, player, &entry.resource, took);
    if given < took {
        // The player could not take them all: what stayed goes back in, so an
        // unfillable hand never destroys items.
        storage.insert(entry.resource.clone(), took - given);
    }
    write(host, level, pos, &storage);
    tell(
        host,
        player,
        format!(
            "Took {} x {}{}",
            grouped(given),
            entry.resource.id(),
            free_note(&storage)
        ),
    );
}

/// A player broke a storage block: the contents go to the breaker, so a filled
/// block never turns into a plain block item and loses 1,048,576 items.
///
/// `/give` throws what the inventory cannot hold at the player's feet and keeps
/// it theirs, so the break is normally never refused. It is refused when the
/// breaker cannot be reached at all (a stale or dead player): then only what did
/// go out is removed from the block, and the block stays, holding the rest.
///
/// Answers whether the break may go on.
pub fn on_break(host: &mut dyn ServerHost, pos: [i32; 3]) -> bool {
    let Some(player) = host.players().acting_player() else {
        return true;
    };
    let Some(level) = player.level(host) else {
        return true;
    };
    let Some(rung) = rung_at(host, level, pos) else {
        return true;
    };
    let mut storage = read(host, level, pos);
    if storage.is_empty() {
        return true;
    }
    let before = storage.clone();
    for entry in before.entries() {
        let given = give(host, player, &entry.resource, entry.amount);
        if given < entry.amount {
            storage.extract(&entry.resource, given);
        }
    }
    if storage == before {
        return true;
    }
    // Something (at least one item) could not be handed over.
    write(host, level, pos, &storage);
    tell(
        host,
        player,
        format!(
            "Extra Disks: the {} storage block could not give all of its contents, so it stays and keeps {}.",
            rung.name(),
            summary("its storage", &storage)
        ),
    );
    false
}
