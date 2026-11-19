//! `/extradisks …`: the block's admin interface and the held disk's whole one.
//!
//! Upstream both are GUI wizards. The block commands move items without going
//! through the inventory, which is how the exact 1,048,576 capacity is proved;
//! the disk commands work on the held disk, which is the only way a disk's
//! storage leaves or enters it in this port (there is no Disk Drive).
//!
//! The arguments are the game's own parsers: `block_pos` resolves `~` and `^`,
//! and `item` arrives as a stack *with its components*, so a named resource can
//! be deposited and withdrawn by command as well as by right-click.

use mod_api::{
    Command, CommandArgumentValue, CommandInvocation, Host, ItemStack, LevelRef, PlayerRef,
    ServerHost, Text,
};

use crate::disk;
use crate::format::grouped;
use crate::storage_block;

/// The commands the mod adds, as one usage each.
pub fn register(commands: &mut Vec<Command>) {
    commands.push(Command::new("extradisks info <pos:block_pos>"));
    commands.push(Command::new(
        "extradisks insert <pos:block_pos> <item:item> <count:integer>",
    ));
    commands.push(Command::new(
        "extradisks extract <pos:block_pos> <item:item> <count:integer>",
    ));
    commands.push(Command::new("extradisks disk info"));
    commands.push(Command::new(
        "extradisks disk insert <item:item> <count:integer>",
    ));
    commands.push(Command::new(
        "extradisks disk extract <item:item> <count:integer>",
    ));
}

/// Run one of the mod's parsed commands; answers whether it was ours.
pub fn run(host: &mut dyn ServerHost, command: &CommandInvocation) -> bool {
    let words: Vec<&str> = command.literals.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["extradisks", "info"] => info(host, command),
        ["extradisks", "insert"] => insert(host, command),
        ["extradisks", "extract"] => extract(host, command),
        ["extradisks", "disk", rest @ ..] => disk_command(host, command, rest),
        _ => return false,
    }
    true
}

fn say(host: &mut dyn ServerHost, line: String) {
    let _ = host.commands().feedback(Text::literal(line));
}

fn usage(host: &mut dyn ServerHost, usage: &str) {
    say(host, format!("Usage: /{usage}"));
}

/// The execution level of a command, after `execute in`.
fn level_of(host: &dyn Host, command: &CommandInvocation) -> Option<LevelRef> {
    let id = command.dimension.id(host)?;
    host.level(&id)
}

/// The player a command acts for: told by the game as the hook's acting player.
fn acting(host: &mut dyn ServerHost) -> Option<PlayerRef> {
    host.players().acting_player()
}

fn block_pos(command: &CommandInvocation) -> Option<[i32; 3]> {
    match command.argument("pos")? {
        CommandArgumentValue::BlockPos(pos) => Some(*pos),
        _ => None,
    }
}

fn item(command: &CommandInvocation) -> Option<ItemStack> {
    match command.argument("item")? {
        CommandArgumentValue::Item(stack) => Some(stack.clone()),
        _ => None,
    }
}

fn count(command: &CommandInvocation) -> Option<u64> {
    match command.argument("count")? {
        CommandArgumentValue::Integer(count) => u64::try_from(*count).ok(),
        _ => None,
    }
}

/// `extradisks info <pos>`: what the block at `pos` holds.
fn info(host: &mut dyn ServerHost, command: &CommandInvocation) {
    let (Some(pos), Some(level)) = (block_pos(command), level_of(host, command)) else {
        return usage(host, "extradisks info <x> <y> <z>");
    };
    let Some(rung) = storage_block::rung_at(host, level, pos) else {
        return say(host, "No Extra Disks storage block there.".to_owned());
    };
    let storage = storage_block::read(host, level, pos);
    say(host, disk::block_line(rung, pos, &storage));
}

/// `extradisks insert <pos> <item> <count>`: what fits of `count` goes in.
fn insert(host: &mut dyn ServerHost, command: &CommandInvocation) {
    let (Some(pos), Some(level), Some(stack), Some(count)) = (
        block_pos(command),
        level_of(host, command),
        item(command),
        count(command),
    ) else {
        return usage(host, "extradisks insert <x> <y> <z> <item> <count>");
    };
    if storage_block::rung_at(host, level, pos).is_none() {
        return say(host, "No Extra Disks storage block there.".to_owned());
    }
    let resource = storage_block::resource_of(host, &stack);
    let mut storage = storage_block::read(host, level, pos);
    let took = storage.insert(resource, count);
    storage_block::write(host, level, pos, &storage);
    let name = stack.item().id(host);
    let line = if took == 0 {
        format!("Refused: {name} does not fit (the storage takes no more).")
    } else {
        format!(
            "Inserted {} x {name} (refused {}).",
            grouped(took),
            grouped(count - took)
        )
    };
    say(
        host,
        format!("{line}{}", storage_block::free_note(&storage)),
    );
}

/// `extradisks extract <pos> <item> <count>`: up to `count` goes to the player.
fn extract(host: &mut dyn ServerHost, command: &CommandInvocation) {
    let (Some(pos), Some(level), Some(stack), Some(count)) = (
        block_pos(command),
        level_of(host, command),
        item(command),
        count(command),
    ) else {
        return usage(host, "extradisks extract <x> <y> <z> <item> <count>");
    };
    if storage_block::rung_at(host, level, pos).is_none() {
        return say(host, "No Extra Disks storage block there.".to_owned());
    }
    let Some(player) = acting(host) else {
        return say(
            host,
            "Only a player can take items out of a storage block.".to_owned(),
        );
    };
    let resource = storage_block::resource_of(host, &stack);
    let mut storage = storage_block::read(host, level, pos);
    let took = storage.extract(&resource, count);
    let given = storage_block::give(host, player, &resource, took);
    if given < took {
        // What could not be handed over stays in the storage.
        storage.insert(resource.clone(), took - given);
    }
    storage_block::write(host, level, pos, &storage);
    let name = stack.item().id(host);
    say(
        host,
        format!(
            "Extracted {} x {name} (the storage had {}, {} stayed).{}",
            grouped(given),
            grouped(took),
            grouped(took - given),
            storage_block::free_note(&storage)
        ),
    );
}

/// The held stack and the slot it is in.
fn held(host: &dyn Host, player: PlayerRef) -> Option<(u32, ItemStack)> {
    let slot = u32::try_from(player.selected_slot(host)?).ok()?;
    player
        .inventory_slots(host)?
        .into_iter()
        .find(|(index, _)| *index == slot)
}

/// `/extradisks disk …`: the held 1024k storage disk.
fn disk_command(host: &mut dyn ServerHost, command: &CommandInvocation, words: &[&str]) {
    let Some(player) = acting(host) else {
        return say(host, "Hold a 1024k storage disk first.".to_owned());
    };
    let Some((slot, held)) = held(host, player) else {
        return say(host, "Hold a 1024k storage disk first.".to_owned());
    };
    if !disk::is_disk(host, &held) {
        let name = held.item().id(host);
        return say(
            host,
            format!("Hold a 1024k storage disk first (this is {name})."),
        );
    }
    match words {
        ["info"] => say(host, disk::describe(&disk::read(&held))),
        ["insert"] => {
            let (Some(stack), Some(count)) = (item(command), count(command)) else {
                return usage(host, "extradisks disk insert <item> <count>");
            };
            let resource = storage_block::resource_of(host, &stack);
            let mut storage = disk::read(&held);
            let took = storage.insert(resource, count);
            let mut updated = held.clone();
            disk::write(&mut updated, &storage);
            let _ = player.set_inventory_slot(host, slot, Some(updated));
            let name = stack.item().id(host);
            if took == 0 {
                say(
                    host,
                    format!("Refused: {name} does not fit (the disk takes no more)."),
                );
            } else {
                say(
                    host,
                    format!(
                        "Inserted {} x {name} (refused {}). {}",
                        grouped(took),
                        grouped(count - took),
                        disk::describe(&storage)
                    ),
                );
            }
        }
        ["extract"] => {
            let (Some(stack), Some(count)) = (item(command), count(command)) else {
                return usage(host, "extradisks disk extract <item> <count>");
            };
            let resource = storage_block::resource_of(host, &stack);
            let mut storage = disk::read(&held);
            let took = storage.extract(&resource, count);
            let given = storage_block::give(host, player, &resource, took);
            if given < took {
                // What could not be handed over stays on the disk.
                storage.insert(resource.clone(), took - given);
            }
            let mut updated = held.clone();
            disk::write(&mut updated, &storage);
            let shown = disk::read(&updated);
            let _ = player.set_inventory_slot(host, slot, Some(updated));
            let name = stack.item().id(host);
            let line = if given < took {
                format!(
                    "Extracted {given} x {name}; the inventory held no more ({} stayed on the disk).",
                    grouped(took - given)
                )
            } else {
                format!(
                    "Extracted {} x {name}. {}",
                    grouped(given),
                    disk::describe(&shown)
                )
            };
            say(host, line);
        }
        _ => say(
            host,
            "Usage: /extradisks disk info|insert <item> <count>|extract <item> <count>".to_owned(),
        ),
    }
}
