# Extra Disks — port notes (slice 1, migrated to the SDK format)

Ported from **Extra Disks `1.21.1-4.0.15`** (`ChaoticTrials/ExtraDisks`,
**Apache-2.0**), mod id `extradisks`. Jar kept at
`ports/_shared/licence-cache/ExtraDisks-1.21.1-4.0.15.jar`; its
`META-INF/neoforge.mods.toml` says `license = "The Apache License, Version 2.0"`,
so the licence is **rank 0 (permissive)** — the campaign source map's `MIT` is
the catalogue's claim, the jar wins (ARCHITECTURE.md §8). Local-only until the
maintainer approves publishing; publishable under Apache-2.0 with this source
and the upstream notice.

**Class + tier:** `content` / **machines** (`ports/ledger.csv`; provenance
`ports/_shared/source-map/atm10-sources.csv` — `ChaoticTrials/ExtraDisks`), and an
**add-on**: its required parent is `refinedstorage`, which this campaign has
itself ported (`ports/refinedstorage`, MIT).

**No cross-mod API** (ARCHITECTURE.md §3): Horizon injects each mod as its own
crate, so this port re-implements the storage semantics it needs —
`Storage`, the item resource identity and the saved encoding live in this crate's
`runtime.rs`; nothing is imported from `ports/refinedstorage`.

**Format: the SDK package (2026-11-18).** This port was migrated from the retired
`.hnmod` container to one flat `.hmod` that carries **both halves** — see
[`## Migration to the SDK format`](#migration-to-the-sdk-format). `src/runtime.rs`
is **byte-identical** to the pre-migration `mixins/extradisks/runtime.rs`
(sha256 `19614096ed66aea0fc3eacf06e7ef543eb664659be4298c7dab236d59e34aa93`),
including the copy inside the built package.

Status: **slice 1 implemented** — the 1024k item storage (1,048,576 items) as a
block and as a portable disk. The behaviour was verified in-game as 0.1.0; the
0.2.0 interactions (the right-click, the break and the command family over the
SDK's typed hooks) are **compiled and packaged against the exact pin but not
re-run in-game here** (no `horizon-launcher` on this machine) — gap 12 below.

## The slice

| Id | What it is | Holds |
|---|---|---|
| `extradisks:1024k_item_storage_block` | the storage block (a block entity) | 1,048,576 items in up to 256 resources |
| `extradisks:1024k_item_storage_disk` | the storage disk (an item) | the same, in the `extradisks:storage` item component |

Both use one engine (`runtime.rs`), so the capacity rule, the resource identity
and the encoding cannot drift apart:

- `1024k` = `TIER_6(1024L) × 1024` = **1,048,576 items**
  (`ExtraItemStorageVariant`; see [`FINDINGS.md`](FINDINGS.md)).
- insert is upstream's `min(amount, free)` (`LimitedStorageImpl.insert`): a
  partial insert takes what fits, and a full container refuses everything.
- extract is `min(available, amount)`, and an emptied resource disappears.
- a resource is the item **plus its components**, so two named diamonds are two
  resources.
- at most 256 resources (the parent port's bound, kept so one component and one
  block-entity compound stay small; upstream's list is unbounded).
- saved form: `"<amount>|<id>|<component key>"` per entry, one list of strings
  in the block entity's compound (`ExtradisksStorage`) and in the disk's item
  component (`extradisks:storage`). A hostile or cut-off save is trimmed to
  capacity, not trusted. The component key is the item's component patch as
  network NBT in hex (`src/codec.rs`), so it is equal exactly when two stacks
  stack together.

### The interface (a recorded deviation)

Upstream the block is a Refined Storage **network node** with a GUI: it joins the
network, consumes 14 FE/t (`ModConfig.storageblocks.item.1024k`) and opens
`Menus.getItemStorage()`. This port has no network (that is the parent port's
crate) and no mod menu, so:

- **right-click** the block: a held stack goes in (what fits of it), an empty
  hand takes one stack of the first resource out;
- **`/extradisks`**: `info <pos>`, `insert <pos> <item> <count>`,
  `extract <pos> <item> <count>`, and the `disk` family (`disk info|insert|extract`)
  on the held disk;
- **breaking** the block gives its contents to the breaker.

The admin commands move items without the inventory (they are how the exact
1,048,576 capacity is proved); the right-click path conserves items.

## Upstream files read

`ChaoticTrials/ExtraDisks`, branch `1.21.x`, commit `91da5f58` (2026-05-03) —
the repository has no `1.21.1-4.0.15` tag, and this branch's sources match the
jar (variants, ids, loot function, config defaults). Files:
`content/item/ExtraItemStorageVariant.java`,
`content/item/ExtraItemStorageBlockProvider.java`,
`content/item/ExtraItemStorageBlockItem.java`,
`content/item/ExtraItemStorageDiskItem.java`, `Registration.java`,
`ModConfig.java`, `loottable/ExtraStorageBlockLootFunction.java`,
`loottable/ExtraLootFunctions.java`. Parent semantics from
`refinedmods/refinedstorage2` tag `v2.0.9`:
`common/content/BlockConstants.java`, `common/storage/StorageTypes.java`,
`common/storage/storageblock/StorageBlock.java`,
`api/storage/StorageImpl.java`, `api/storage/limited/LimitedStorageImpl.java`.
Behaviours extracted, with the numbers and where they came from, are in
[`FINDINGS.md`](FINDINGS.md). Both clones were scratch (`/tmp/opencode/`), never
committed; the jar stays as the licence evidence.

## Upstream defects fixed

None found in this slice's scope: the variant ladder, the capacity arithmetic,
the insert rule, the ids and the lang strings agree with each other and with the
jar's data files. Two things that *look* like defects were checked and are not:

- `1024k` meaning 1,048,576 items (not 1,024,000) is the enum's own
  `capacity * 1024`, consistent with the parent's 1k = 1,024;
- the `1048576k` rung (1,073,741,824 items) is displayed as "1048m" in `en_us`
  and `262144k` as "262m" — a display abbreviation, not a capacity.

## Migration to the SDK format

*Written 2026-11-18 for this port, the second migration after `imfast`. Follow
it with `ports/imfast/notes.md` beside it: that recipe is the measured one, and
this section records where a **content + code** bundle differs.*

**Why.** Horizon's mod-API merge (#16941, launcher ≥ 0.2.70) retired `.hnmod` on
2026-11-18; an `.hnmod` no longer loads, and the registry closed every `.hnmod`
submission — this mod's was **#874**. The new unit is **one flat `.hmod`**, and it
may carry content *and* code, which is exactly what this mod needed: the AE2
"embedded content" pattern (a content `.hmod` carried inside an `.hnmod` and
written to `.horizon-cache` at start) is **gone**, because `assets/` and `data/`
now sit beside `src/` in the same package. Pins: SDK nightly
**`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, API `0.2`, package format 6.

### 1. The file map (0.2.0)

| Path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and the only identity: `id`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `[native]` pinned to the SDK's `game`/`horizon`/`mixins` |
| `Cargo.toml` | one crate; the only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks`: `register`, `component_types`, `right_click_block`, `break_block`, `register_commands`, `run_mod_command` |
| `src/runtime.rs` | the port's pure logic, **copied byte-for-byte** from the retired `mixins/extradisks/runtime.rs` (17 tests) |
| `src/codec.rs` | pure: a stack's component key (network NBT in hex) and the saved-line list, 6 tests |
| `src/format.rs` | pure: the grouped counts and the one-line storage summary, 3 tests |
| `src/storage_block.rs` | the block entity's saved compound, the right-click and the break |
| `src/disk.rs` | the `extradisks:storage` component (`Component` descriptor) and the disk's read/write |
| `src/commands.rs` | the six usages, their argument accessors and their bodies |
| `assets/extradisks/`, `data/extradisks/` | the carried content: blockstate, models, textures, lang, the block and item declarations, the loot table, the creative tab — **unchanged** from 0.1.0 |
| `LICENSE`, `LICENSE-NOTICE.md`, `README.md` | inside the package (the publisher's tree is not the package) |
| `.gitignore` | `/target`, `/.horizon`, `/Cargo.lock` (trap 8) |
| `tools/e2e.sh` | the in-game scenario, updated for one package and the break's `/give` semantics |
| `extradisks.hmod` | the delivered package |

### 2. Helper → hook map (0.1.0 injections → 0.2.0 `Hooks` methods)

| 0.1.0 (retired) | 0.2.0 | What moved |
|---|---|---|
| `[[injection]] component_types` + `component_types.rs` | `Hooks::component_types(&self, cx, &mut ComponentTypesEvent)` | `event.types.push(...)`; same `RegistrationContext`-phase collect hook |
| `[[injection]] block_entity` + `block_entity.rs` (returned `(String, Compound)`) | `Hooks::register` → `r.block_entity_type("item_storage").blocks(&["1024k_item_storage_block"]).build()` | the declaration owns the type now, and a declared type **starts with an empty compound**, so no `BlockEntityPart` is needed; the hook itself is a per-object Part, not a global hook |
| `[[injection]] right_click_block` + `right_click.rs` (`canceled: &mut bool`) | `Hooks::right_click_block(&self, HostSide, &mut RightClickBlockEvent) -> Flow` | fires on both sides: match `HostSide::Server` and `event.click.side`, then `*event.cancellation_result = InteractionResult::Success` + `Flow::Cancel` |
| `[[injection]] destroy_block` + `destroy.rs` | `Hooks::break_block(&self, &mut dyn ServerHost, &mut BreakBlockEvent) -> Flow` | **renamed** (`destroy_block` no longer exists); `event.pos`; `Flow::Continue` |
| `[[injection]] register_commands` + `register.rs` (`NativeCommand`) | `Hooks::register_commands(&self, host, &mut RegisterCommandsEvent)` | `mod_api::Command::new(usage)` with typed argument kinds (`block_pos`, `item`, `integer`) |
| `[[injection]] perform_command` + `perform.rs` (raw `&str`) | `Hooks::run_mod_command(&self, host, &mut RunModCommandEvent) -> Flow` | **renamed and parsed**: `event.command.literals` + `event.command.argument("pos")`; `Flow::Cancel` = handled |
| `[[injection]] content_root` + `embedded_content.rs` + `data/extradisks-content.hmod` | **deleted** | the content half is in the same package; nothing is written to `.horizon-cache` any more |
| `[[module]] state` + `support.rs` (`extradisks_state`) | `src/storage_block.rs`, `src/disk.rs`, `src/commands.rs` | one responsibility per file; the same functions, retyped against the host API |
| `[[module]] runtime` + `runtime.rs` (`crate::game::extradisks`) | `src/runtime.rs` + `pub mod runtime;` | byte-identical; `pub mod` keeps its `pub` items from warning `dead_code` (imfast trap 8) |
| `horizon-mixin.toml`, `compatibility.json`, `rust-toolchain.toml`, `horizon-native.json` | deleted | `horizon-mod.toml` is the only manifest; the pinned commit decides the toolchain |
| `tools/horizon.py`, `tools/package.py`, `tools/ladder-embed.sh`, `tools/test_*.py`, `tools/hmod-load.sh`, `tools/add-notices.py` | deleted | staging and `.hnmod` packaging are gone |

**Kept:** `tools/e2e.sh` (the scenario and its assertions are format-independent),
`notes.md`, `FINDINGS.md`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, the whole
`assets/` + `data/` content, and the jar in `licence-cache/`.

**Retired, in evidence:** `ports/_shared/evidence/extradisks/retired-hnmod-0.1.0/`
holds `mixins/` (with the old `runtime.rs` and `support.rs`), `extradisks.hnmod`
(28,735 B), the old content package `extradisks.hmod`, the old content manifest,
`compatibility.json`, `rust-toolchain.toml`, `Cargo.lock`, the retired tools,
`PLAN.md`, `VALIDATION.md` and the port's old `AGENTS.md`.

### 3. How the glue is written now (what cost me the most time)

The SDK's `Host` is **not** the old `Game`. Facts learned by compiling against
`0b14f8a` (each one an error I hit):

1. `Host` carries **services** (`host.commands()`, `host.players()`,
   `host.items()`, `host.level(&id)`) and the area traits, but a *thing's* reads
   are **handle methods**: `level.is_loaded(host, pos)`, `level.block(host, pos)`,
   `entity.data(host)`, `entity.replace_data(host, data)`,
   `player.selected_slot(host)`, `player.inventory_slots(host)`,
   `player.set_inventory_slot(host, slot, stack)`, `player.give_stack(host, stack)`,
   `player.tell(host, text)`, `stack.item().id(host)`, `block_key.id(host)`,
   `dimension_key.id(host)`. There is no `host.item_key_id(...)`,
   `host.player_inventory_slots(...)` or `host.acting_player(...)`.
2. `host.players()` is **server-only** (`ServicesServerHost`) — a helper that
   needs the acting player must take `&mut dyn ServerHost`.
3. `host.level(...)` takes a **dimension id string**: go through
   `DimensionKey::id(host)` (a parsed command's `dimension`) or
   `LevelDimension::key().id()` (a click's `dimension`).
4. `break_block` and the command hooks carry **no dimension**: the acting
   player's `player.level(host)` is the level to write.
5. **`JavaString` is not exported**, but `Tag::String("x".into())` compiles —
   the enum variant's field type drives the inference — and
   `Tag::compound([("key", Tag::Int(1))])` builds a compound from `&str` keys.
6. **There is no exported SNBT parser or formatter.** A component identity must
   therefore be built from values, not text: `Vec::<u8>::try_from(&Tag)` is the
   network NBT encoding (canonical because `Compound` is a `BTreeMap`), hex it
   for the save, and `Tag::try_from(bytes)` to read it back (`src/codec.rs`).
7. The old `Text`/`game.held()`/`native_*` helpers are gone: feedback in a
   command is `host.commands().feedback(text)` (it honours silence and
   track-output); feedback from a click is `player.tell(host, text)`.
8. `right_click_block` returns `Flow`, and **cancelling with the default
   `InteractionResult::Pass` still runs the item's own use**; a storage block
   sets `Success` so the click ends there.
9. `Command` usages carry their own permission level: `Command::new` is 0 (every
   player), which is what the 0.1.0 `perform_command` gave the admin commands.

### 4. Ladder for this package

`build-slot.sh` speaks the retired staged tool, so — like `imfast` — every step
runs **directly in the port** with `CARGO_INCREMENTAL=0`, no staging, no
`--target-dir`:

```sh
cd ports/extradisks
CARGO_INCREMENTAL=0 cargo test                        # 26 tests
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
cargo fmt --check                                     # NOT --all (imfast trap 4)
rm -f Cargo.lock                                      # trap 8
CARGO_INCREMENTAL=0 cargo horizon-mod build            # -> target/horizon-mod/extradisks.hmod
cp target/horizon-mod/extradisks.hmod ./extradisks.hmod
cargo horizon-mod check extradisks.hmod
cargo horizon-mod check extradisks.hmod --json
cargo horizon-mod fits extradisks.hmod --build nightly
unzip -p extradisks.hmod uses.json                     # what it touches, as the registry sees it
```

Measured: **26 passed** (17 runtime + 6 codec + 3 format); clippy 0 warnings;
`fmt --check` clean; package **37,801 B**, sha256
`e47071a3c5ff8f0643330079ffb85810504d41a6a40e0adac8eff7b622b8334f`, **26
entries**; `check` and `fits` exit 0; a second build is byte-identical.

`uses.json` is 252 entries — the six hooks, the `Component` impl, the handle
methods above, and the traits they come through. The registry's check does not
execute native source; the compile against the exact pin is the proof that the
surface exists.

### 5. Traps the next migration must know

1. **A retired path blocks the build.** `error: unexpected package path:
   mixins/extradisks/runtime.rs` — delete/move `mixins/`, the `.hnmod`,
   `compatibility.json`, `rust-toolchain.toml` and the old `tools/` *before* the
   first build.
2. **`cargo horizon-mod build` packs `README.md`.** It is part of the package
   (as are `LICENSE` and any root `NOTICE`), so a stale README installs a stale
   description; this port's was rewritten in the same touch.
3. **A retired hook name is an unimplemented method.** `destroy_block` is
   `break_block` and `perform_command` is `run_mod_command`; read
   `.horizon/sdk/api/crates/horizon-mod-vocab/src/hooks/<name>.rs` (declaration)
   and `generated/mod_api_generated.rs` (the trait method, the event struct and
   every field) — never the pinned `docs/native-mods.md`, which predates the
   merge and still describes `native_*` functions.
4. **Handle methods, not host methods** (trap list above, §3.1): a
   `host.<thing>_<verb>(...)` call that "should exist" usually lives on the
   handle, e.g. `level.is_loaded(host, pos)` rather than `host.level_is_loaded(...)`.
5. **`host.level` takes a name.** `DimensionKey::id(host)` is the bridge from a
   parsed command to a `LevelRef`.
6. **No exported `JavaString`, no exported SNBT.** Build tags by value
   (`Tag::String("x".into())`, `Tag::compound([...])`); for a canonical text form
   of a compound use the network-NBT bytes in hex, not a hand-written SNBT.
7. **A content + code bundle needs no embedded-content hook at all** — that whole
   AE2 pattern (`content_root`, a `data/*-content.hmod`, `.horizon-cache`) is
   obsolete; `assets/` and `data/` travel in the `.hmod`.
8. **`Cargo.lock` is packed and changes the artifact** (imfast trap 5): any
   `cargo test`/`clippy` writes one, and the resolved versions are newer than the
   SDK's own `api/Cargo.lock`. `rm -f Cargo.lock` right before the build, and
   `/Cargo.lock` in `.gitignore`.
9. **`cargo fmt --all --check` reports the fetched SDK** (imfast trap 4); use
   `cargo fmt --check`.
10. **A block entity is declared, not hooked.** `Hooks::register` +
    `Registrar::block_entity_type(...).build()`; the `block_entity` method is a
    `BlockEntityPart` hook, and a declared type starts with an empty compound,
    so the port needs no Part.
11. **`side = "both"`, `capabilities = ["registries"]`, `api = "0.2"`** are
    required values, not choices (imfast traps 6–7); this mod also *uses* the
    registry capability, since it adds a block entity type beside its content.
12. **The SDK is per-clone**: `.horizon/sdk/` is fetched by the first
    `cargo horizon-mod` command and must never be packed; `.gitignore` carries it.

## Review repairs (PR #681, 0.1.0) and what 0.2.0 keeps

The reviewer's items and their state:

1. *Absolute upstream URL and author credit* — the manifest's `authors` lists
   `MelanX` first and `README.md` links
   `https://github.com/ChaoticTrials/ExtraDisks`; carried into 0.2.0.
2. *Include the Apache-2.0 licence and the upstream notice in the distributed
   artifact* — `LICENSE` and `LICENSE-NOTICE.md` are **roots of the `.hmod`**
   (see `unzip -l`), so an artifact-only download carries them.
3. *Preserve a broken block's contents, or disclose the limitation* — the
   `break_block` hook gives every entry to the breaker (`player.give_stack`), so
   a filled block cannot lose 1,048,576 items; what the inventory cannot hold is
   thrown at their feet by `/give`'s own rule (see deviations, below).
4. *The staging hash* was the orchestrator's (horizon-mods#686); 0.2.0 is a new
   artifact with a new hash and a new version, as the registry requires.

## Deviations (deliberate, 0.2.0)

1. **A broken block returns its contents instead of refusing the break.** 0.1.0
   cancelled a break whose contents did not fit. The SDK's give has `/give`
   semantics — overflow is thrown at the breaker's feet and stays theirs — so
   nothing can be lost either way, and refusing would be a guess about their
   inventory. A user notices items landing around them instead of the block
   staying put. There is still no loot function that copies a mod block entity
   into the dropped item (gap 6).
2. **Commands take parsed arguments.** `block_pos` resolves `~`/`^`, and `item`
   arrives as a stack *with its components*, which **closes** 0.1.0's gap 8: a
   component-carrying resource can now be deposited and withdrawn by command. A
   count below 0 is refused by the game's `integer` parser before the hook.
3. **`/extradisks extract` refuses without a player.** There is no acting player
   for a console or command-block source, so nothing can be handed over; the
   command says so instead of silently dropping items.
4. **The disk commands read the selected hotbar slot only.** The block's
   right-click handles both hands (main and off, slot 40); a disk in the off
   hand is not a disk command target — `Hold a 1024k storage disk first.`
5. **The state lives in the block entity and the item, not in the mod.** The mod
   struct is a unit type; every storage is its own compound or component, so a
   reload, a relaunch and a chunk unload are the engine's problem.
6. **The block is a declared mod block entity, not a template's.** The content
   file still uses `template: minecraft:barrel` for its properties and shape; the
   entity type is `extradisks:item_storage` from `Hooks::register`.

## Gaps / deviations (slice 1)

1. **No network.** The storage block neither joins a Refined Storage network nor
   answers the parent's grid: upstream it is a network node (`StorageBlock` →
   `updateActiveness`/`doWork`). A player sees standalone storage, not storage a
   Grid can browse.
2. **No energy.** Upstream the 1024k block consumes 14 FE/t
   (`ModConfig.storageblocks.item.1024k`) and the disk has no buffer. No FE hook
   is used, and no energy is modelled.
3. **No GUI.** Upstream opens `Menus.getItemStorage()` with insert slots, a
   search box and a sorting/priority panel. This port has the right-click and
   the commands and no screen.
4. **The rest of the ladder and the other families do not exist**: the other 7
   item rungs (256k, 4096k, 16384k, 65536k, 262144k, 1048576k, infinite), the 17
   other disks, the 23 parts, the 5 fluid blocks/disks/parts, the 5 chemical
   ones, `advanced_machine_casing`, `advanced_storage_housing`, the withering
   processors, and the 36 storage-upgrade recipes.
5. **No recipes at all.** Every recipe in the jar needs `refinedstorage:`
   ingredients (`quartz_enriched_iron`, `advanced_processor`,
   `improved_processor`) and the `refinedstorage:parts/items/1024k` tags, which
   this content mod cannot declare in another namespace and the parent port does
   not ship yet. The two items come from the creative tab.
6. **Breaking the block** returns the contents to the breaker (deviation 1); the
   loot table itself still drops a plain block item, because the pin has no loot
   function that copies a mod block entity's saved compound into the dropped
   item's components. Recorded as an API gap.
7. **The disk's contents only leave or enter through the commands.** Upstream the
   disk is inert without a Disk Drive and the drive reads its storage component;
   this port has no drive, so `/extradisks disk …` is the disk's whole interface.
8. **256 resources**, not upstream's unbounded list (see the slice table).
9. **No comparator output, no hopper faces, no redstone**: upstream's storage
   node answers comparators through its storage configuration.
10. **Cosmetic**: the block model is the jar's `cube_all` with the jar's texture
    and the disk is the jar's `handheld` model; upstream draws nothing extra (no
    animated disk list), so nothing is lost here.
11. **The off-hand disk** is not a command target (deviation 4).
12. **The 0.2.0 interactions are not re-run in-game here.** This machine has no
    `horizon-launcher`, so no game can be built carrying a *native* mod; the
    package is compiled and checked against the exact pin, and the 0.1.0 in-game
    run remains the behaviour evidence for the parts that did not change
    (capacity, insert/extract, the saved encoding, persistence). The new
    interaction paths (right-click over the typed hook, `break_block` giving,
    the parsed commands) are typechecked only. `tools/e2e.sh` is updated for the
    one-package layout and the new break semantics and must be run with
    `GAME=<launcher-built horizon-client>` before the port claims `ported`.
13. **No glue unit tests.** The hook bodies need a host; the SDK's fake-host
    machinery is not part of the public mod surface this port uses, so the pure
    decisions live in `runtime.rs`/`codec.rs`/`format.rs` (26 tests) and the glue
    is proven in-game, not in `cargo test`.

## Verification

Pin: SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, API `0.2`, package format 6,
launcher 0.2.78. Every step ran **directly in the port** with
`CARGO_INCREMENTAL=0`, no staging and no `--target-dir` (see §4).

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

| Step | Command | Result |
|---|---|---|
| runtime + pure tests | `cargo test` | **26 passed**, 0 failed (`evidence/extradisks/sdk-test.log`) |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings (`evidence/extradisks/sdk-clippy.log`) |
| format | `cargo fmt --check` | clean (`evidence/extradisks/sdk-fmt.log`) |
| package | `cargo horizon-mod build` | `target/horizon-mod/extradisks.hmod`, EXIT 0 (`evidence/extradisks/sdk-build.log`) |
| registry check | `cargo horizon-mod check extradisks.hmod` | `extradisks 0.2.0 (native-mod, format native-1/0b14f8a/e59da90, for 26.3)`, **37,801 B**, sha256 `e47071a3…334f`, **EXIT 0** (`sdk-check.log`, `sdk-check-json.log`) |
| what it uses | `unzip -p extradisks.hmod uses.json` | 252 surface entries (`sdk-uses.json`) |
| fits the pinned build | `cargo horizon-mod fits extradisks.hmod --build nightly` | `0b14f8a  made for`, EXIT 0 (`sdk-fits.log`) |
| `runtime.rs` untouched | `cmp` + `unzip -p … src/runtime.rs \| sha256sum` | `19614096…aa93` on disk, in the package and in the retired tree (`sdk-runtime-cmp.txt`) |
| repeat build | `rm -f Cargo.lock; cargo horizon-mod build` again | same 37,801 B, same sha256, `cmp` clean |
| symbols line | `python3 ports/tools/check-skills.py --notes ports/extradisks` | see the report; the `Skills:` line is above |
| in-game e2e | `GAME=<launcher-built> tools/e2e.sh` | **not run**: no launcher here (gap 12) |

### What the 0.1.0 e2e shows (a headless run, `--headless`, creative)

The End arena (stone floor at y=63, the block at 40,64,0). Quoted from the
0.1.0 evidence, which is what the unchanged parts still rest on:

```
block(40,64,0): extradisks:1024k_item_storage_block[facing=north,open=false]
info: 0 / 1,048,576 items in 0 resources, 1,048,576 free            (empty)
info: 5 / 1,048,576 items in 1 resources, 1,048,571 free            (5 diamonds right-clicked in)
info: 0 / 1,048,576 items in 0 resources, 1,048,576 free            (empty hand took them back)
Inserted 1,048,576 x minecraft:stone (refused 0).; 0 of 1,048,576 free
info: 1,048,576 / 1,048,576 items in 1 resources, 0 free            (exactly full)
Refused: minecraft:stone does not fit (the storage takes no more).  (the control)
Refused: minecraft:dirt does not fit (the storage takes no more).
Extracted 576 x minecraft:stone (the storage had 576, 0 stayed).
Inserted 576 x minecraft:dirt (refused 424).                        (partial insert)
info: 1,048,576 / 1,048,576 items in 2 resources, 0 free
disk: 0 / 1,048,576 items in 0 resources, 1,048,576 free            (the held disk)
disk: 1,048,576 / 1,048,576 items in 1 resources, 0 free            (insert 1,048,576 diamonds)
Refused: minecraft:diamond does not fit (the disk takes no more).
Extracted 64 x minecraft:diamond. The disk: 1,048,512 / 1,048,576 items in 1 resources, 64 free
info 56,64,0: No Extra Disks storage block there.                   (a vanilla barrel)
… relaunch: info 1,048,576 / 1,048,576 items in 2 resources, 0 free (contents survived)
… extract 576 dirt: 1,048,000 / 1,048,576 items in 1 resources, 576 free
PASS: … EXIT:0
```

The controls are inside the scenario: **over capacity** (two refusals at exactly
1,048,576, on the block and on the disk), **partial insert** (576 of 1,000), and
**not a storage block** (a vanilla barrel). `SELFTEST=1` flips the first refusal
into "the storage grew" and the run fails, so the scenario can fail.

### Harness facts learned there (still true)

- A command's chat answer races unrelated chat: read `ctl events --since SEQ` and
  filter `event == "chat"` instead of `tail -1` of `ctl messages`.
- `SELFTEST`/`KEEP` are **not forwarded** to the second machine (only
  `HORIZON_RES` is), so a sentinel leg must run with `BUILD_LOCAL=1`.
- The `.hmod` bytes depend on the box's zlib: this machine's zlib-ng and the
  second machine's stock zlib give different archive sizes for identical
  sources. The delivered artifact was built here, twice, byte-identically.

## Licensing

Apache-2.0, rank 0: a derivative port may be published **under the same licence
with the original notice**; the campaign's own rule still requires the
maintainer's approval for publishing (ARCHITECTURE.md §8), so this port is
local-only until then. `LICENSE` carries the Apache-2.0 text; `LICENSE-NOTICE.md`
records the jar-read licence and the attribution Apache section 4(c) asks for;
both are roots of the `.hmod`. The jar is the evidence of the licence and stays in
`ports/_shared/licence-cache/`.

## Changelog text (what the resubmission says)

> **0.2.0 — native `.hmod` with the SDK (API 0.2).** Republished for the launcher
> ≥ 0.2.70 mod API, superseding **0.1.0** and the closed **#874**: the retired
> `.hnmod` container no longer loads (Horizon's mod-API merge #16941), so this
> version ships content and code as one flat `extradisks.hmod`. Same 1024k
> storage block and disk, same 1,048,576-item capacity, same behaviours as the
> 0.1.0 review (#681) resolved: absolute upstream credit, the Apache-2.0 licence
> and notice inside the artifact, and a broken block that returns its contents.
> The storage engine (capacity, resource identity, saved encoding) is unchanged
> and unit-tested. New: commands take the game's parsed `block_pos` and `item`
> arguments, so a component-carrying resource can be stored by command, and the
> hooks are the API's typed ones (`right_click_block`, `break_block`,
> `run_mod_command`). Builds against Horizon `0b14f8a`, mixins `e59da90`.

## Files

- `horizon-mod.toml`, `Cargo.toml`, `.gitignore` — the manifest, the crate, the ignore list.
- `src/lib.rs` — the one `impl Hooks`; `src/runtime.rs` — the pure engine (byte-identical, 17 tests).
- `src/codec.rs`, `src/format.rs` — pure helpers (9 tests); `src/storage_block.rs`, `src/disk.rs`, `src/commands.rs` — the glue.
- `assets/extradisks/`, `data/extradisks/` — the carried content.
- `LICENSE`, `LICENSE-NOTICE.md`, `README.md` — inside the package.
- `tools/e2e.sh` — the in-game scenario; `FINDINGS.md`, this file.
- `extradisks.hmod` — the delivered package, 37,801 B, sha256
  `e47071a3c5ff8f0643330079ffb85810504d41a6a40e0adac8eff7b622b8334f`.
- Evidence: `../_shared/evidence/extradisks/` (`sdk-*.log`, `sdk-uses.json`,
  `sdk-sha256.txt`, `sdk-entries.txt`, `sdk-runtime-cmp.txt`, and the retired
  0.1.0 tree under `retired-hnmod-0.1.0/`). Jar:
  `../_shared/licence-cache/ExtraDisks-1.21.1-4.0.15.jar`.
