//! The 1024k storage disk: the same storage, carried in the item's component.
//!
//! Upstream the disk is inert without a Refined Storage Disk Drive, and the
//! drive reads its storage component. This port has no drive (the parent port's
//! drive is another crate, and there is no cross-mod API), so the disk's whole
//! interface is `/extradisks disk …` on the held disk: `info`, `insert` and
//! `extract`.

use mod_api::{Component, Host, ItemStack, Tag};

use crate::codec;
use crate::format;
use crate::runtime::{Storage, Variant};

/// The item component that carries a disk's contents (`component_types`).
pub const COMPONENT: &str = "extradisks:storage";

/// The component's descriptor: the storage's saved lines, as a plain list tag.
///
/// The value is the NBT itself rather than a decoded type, because the
/// runtime's text form is the only shape both the block entity and the item
/// keep, and one encoding for both is what stops them drifting apart.
pub struct SavedStorage;

impl Component for SavedStorage {
    type Value = Tag;

    fn id(&self) -> &str {
        COMPONENT
    }

    fn decode(&self, value: &Tag) -> Option<Tag> {
        Some(value.clone())
    }

    fn encode(&self, value: Tag) -> Tag {
        value
    }
}

/// The storage a disk carries; empty when it has no component (a freshly made
/// or creative-given disk).
pub fn read(stack: &ItemStack) -> Storage {
    let saved = stack
        .get(&SavedStorage)
        .map(|tag| codec::saved_lines(Some(&tag)))
        .unwrap_or_default();
    Storage::parse(format::capacity(), &saved)
}

/// Write a storage into a disk's component; an empty disk loses the component
/// rather than keeping an empty list.
pub fn write(stack: &mut ItemStack, storage: &Storage) {
    if storage.is_empty() {
        stack.remove(&SavedStorage);
    } else {
        let _ = stack.set(&SavedStorage, codec::lines_tag(&storage.encode()));
    }
}

/// Whether `stack` is this slice's disk.
pub fn is_disk(host: &dyn Host, stack: &ItemStack) -> bool {
    Variant::from_disk_id(&stack.item().id(host)) == Some(Variant::K1024)
}

/// One line about the held disk, at `storage`.
pub fn describe(storage: &Storage) -> String {
    format::summary("The held 1024k storage disk", storage)
}

/// One line about the block whose contents are being reported.
pub fn block_line(rung: Variant, pos: [i32; 3], storage: &Storage) -> String {
    format::summary(
        &format!(
            "Extra Disks {} storage block at {},{},{}",
            rung.name(),
            pos[0],
            pos[1],
            pos[2]
        ),
        storage,
    )
}
