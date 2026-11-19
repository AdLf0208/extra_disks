# Extra Disks — findings from the jar and the upstream repository

Ported from **Extra Disks `1.21.1-4.0.15`** (jar `ExtraDisks-1.21.1-4.0.15.jar`,
kept at `ports/_shared/licence-cache/`), mod id `extradisks`, author MelanX.
Minecraft Java 1.21.1 / NeoForge. This file is the evidence the slice was
chosen from; `notes.md` is the record of what the port does with it.

## Licence (read from the jar, not a catalogue)

`META-INF/neoforge.mods.toml`:

```
license = "The Apache License, Version 2.0"
issueTrackerURL = "https://github.com/ChaoticTrials/ExtraDisks/issues"
[[mods]] modId = "extradisks" version = "1.21.1-4.0.15" displayName = "Extra Disks"
```

No `LICENSE`/`NOTICE` file ships inside the jar. **Apache-2.0, licence rank 0**
(permissive). The campaign source map says `MIT`; the jar wins
(ARCHITECTURE.md §8, "the jar wins over every catalogue").

Required dependencies declared in the same file: `neoforge [21.1.181,)`,
`minecraft [1.21.1,)`, **`refinedstorage [2.0.0,)` (required)**;
`refinedstorage_mekanism_integration [1.1.0,)` (optional).

## Jar inventory — measured, not assumed (`unzip -l`)

583 entries, 40 class files, 543 data/asset files.

| Family | Files | Ids |
|---|---:|---|
| `blockstates/` | 19 | 8 item + 5 fluid + 5 chemical storage blocks + `advanced_machine_casing` |
| `models/item/` | 58 | a block model, a disk model and a part model per variant, plus casing/housing/processors |
| `models/block/` | 22 | `cube_all` per block, plus `block/disk/{item,fluid,chemical}_disk` |
| `textures/` | 62 | `block/…`, `item/…`, three disk overlays, `gui/advancements.png` |
| `recipe/` | 112 | blocks, disks (shaped + shapeless), parts, storage upgrades, processors |
| `advancement/recipes/` | 76 | recipe unlock advancements (112 + 76 = the 188 "recipes" the brief quotes) |
| `loot_table/` | 19 | one per block, all using the mod's own `extradisks:storage_block` function |
| `lang/` | 13 | `en_us` plus 12 translations |
| `tags/` | 91 | `refinedstorage:*` item/block tags the mod adds its blocks and disks to |
| `META-INF/` | 2 | `neoforge.mods.toml`, `accesstransformer.cfg` |

The item ladder is `256k · 1024k · 4096k · 16384k · 65536k · 262144k · 1048576k ·
infinite`; the fluid ladder `16384b · 65536b · 262144b · 1048576b · infinite`; the
chemical ladder `65536b · 262144b · 1048576b · 8388608b · infinite`. Each rung has
a **storage block**, a **disk** and a **part**; the blocks and disks also have
storage-upgrade recipes (block→block, disk→disk) and shapeless recipes that move
a disk between the two forms' ingredients.

### The exact capacity of the 1024k rung

`src/main/java/de/melanx/extradisks/content/item/ExtraItemStorageVariant.java`
(upstream, tag-less `1.21.x` branch, commit `91da5f58`, 2026-05-03):

```java
TIER_5(256L), TIER_6(1024L), ... TIER_11(1048576L), TIER_12(null);
...
this.name = capacity + "k";
this.capacity = capacity * 1024;      // items
```

So `1024k` = **1024 × 1024 = 1,048,576 items**, and `infinite` has no capacity
(null → unlimited). The whole ladder in items: 262,144 · 1,048,576 · 4,194,304 ·
16,777,216 · 67,108,864 · 268,435,456 · 1,073,741,824 · unlimited.

### What a storage block and a disk are

- `ExtraItemStorageBlockProvider implements StorageBlockProvider` — the block's
  block entity holds a `SerializableStorage` made by
  `StorageTypes.ITEM.create(capacity, onChange)`, gets `Menus.getItemStorage()`
  as its menu, and uses the item resource factory. Energy use per tick from
  `ModConfig` (`storageblocks.item.1024k` = **14 FE/t**, range 0..MAX).
- `StorageBlock` (Refined Storage 2.0.9, `refinedstorage-common`) is an
  `AbstractBaseBlock` + `EntityBlock` with a block-entity ticker that calls
  `updateActiveness` / `doWork` — i.e. it is a **network node** whose storage
  joins the Refined Storage network. Its block properties are
  `BlockConstants.PROPERTIES` = `strength(2.0F, 6.0F).sound(SoundType.STONE)`,
  **no** `requiresCorrectToolForDrops`.
- `ExtraItemStorageDiskItem extends AbstractStorageContainerItem` — the disk is a
  portable container with the same `StorageTypes.ITEM.create(capacity)` storage,
  `stacksTo(1).fireResistant()`, contents in a data component through
  `StorageContainerItemHelper`.
- `ExtraItemStorageBlockItem extends AbstractStorageContainerBlockItem` — the
  block item is also a storage container: the block's storage is transferred into
  the dropped item when the block breaks (`ExtraStorageBlockLootFunction`, the
  lore table's `extradisks:storage_block` function) and back into the block when
  it is placed.

### Storage semantics (the numbers the runtime must reproduce)

Refined Storage 2.0.9, `LimitedStorageImpl.insert`:

```java
long spaceRemaining = capacity - delegate.getStored();
if (spaceRemaining == 0) return 0;
long maxInsert = Math.min(amount, spaceRemaining);
return super.insert(resource, maxInsert, action, actor);
```

- insert is **partial**: `min(amount, free)` is accepted, the rest is refused;
- at exactly full capacity an insert is **refused entirely** (0 accepted);
- extract is `min(available, amount)`;
- a resource's identity is the item **plus its components** (`ItemResource`), so
  two otherwise identical stacks with different components are two resources;
- `StorageImpl` itself keeps a resource list with no per-disk resource limit; this
  port keeps the parent port's bound of 256 resources per container
  (`ports/refinedstorage`, `MAX_RESOURCES`), so an item component cannot grow
  without limit.

### Where the numbers came from

| Number | Source |
|---|---|
| 1,048,576 items | `ExtraItemStorageVariant.TIER_6(1024L)` × 1024 |
| 256k…1048576k ladder | the same enum, in order |
| 14 FE/t | `ModConfig.storageblocks.item.1024k` |
| hardness 2.0 / resistance 6.0 / stone sound | Refined Storage `BlockConstants.PROPERTIES` |
| partial insert, refusal when full | Refined Storage `LimitedStorageImpl.insert` |
| `stacksTo(1)`, `fireResistant` | `ExtraItemStorageDiskItem` constructor |
| block loot carries the storage | `ExtraStorageBlockLootFunction` + the 1024k loot table |

## Repositories read (analysis only; no code copied)

| Repo | Ref | Files read |
|---|---|---|
| `ChaoticTrials/ExtraDisks` | branch `1.21.x`, commit `91da5f58` (2026-05-03) — the tag `1.21.1-4.0.15` does not exist in the repository, and the branch's `1.21.1-4.0.x` sources are the ones the jar was built from (variant enum, ids, loot function, config defaults all match the jar) | `content/item/{ExtraItemStorageVariant,ExtraItemStorageBlockProvider,ExtraItemStorageBlockItem,ExtraItemStorageDiskItem}.java`, `Registration.java`, `ModConfig.java`, `loottable/{ExtraStorageBlockLootFunction,ExtraLootFunctions}.java` |
| `refinedmods/refinedstorage2` | tag `v2.0.9` (the parent port's pin) | `common/content/BlockConstants.java`, `common/storage/{StorageTypes,SameTypeStorageType}.java`, `common/storage/storageblock/StorageBlock.java`, `api/storage/StorageImpl.java`, `api/storage/limited/LimitedStorageImpl.java` |

Both clones were scratch (`/tmp/opencode/extradisks-up/`), never committed.

## Slice chosen from the inventory

`extradisks:1024k_item_storage_block` (the block, holding the 1024k storage) and
`extradisks:1024k_item_storage_disk` (the same storage, portable in a component):
capacity **1,048,576 items**, insert/extract with upstream's partial-insert rule,
contents surviving a relaunch, and an over-capacity insert refused. Why the middle
rung: 256k is the smallest but the brief names 1024k; 1024k is the first rung the
jar's own creative tab icon does not use, and it is the rung whose item count
(1,048,576) is still comfortably inside a `u32` stack count and a single NBT list
entry, so the whole capacity is provable in game in one command.

Everything else in the jar — the other 17 block rungs, 17 disks, 23 parts, the
fluid and chemical ladders, the storage-upgrade recipes, the processors, the
housing and the casing — is listed as a gap in `notes.md`, not claimed.
