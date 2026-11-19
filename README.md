# Extra Disks for Horizon

A clean-room Rust port of [Extra Disks](https://github.com/ChaoticTrials/ExtraDisks)
(`1.21.1-4.0.15`, **Apache-2.0**) to [Horizon](https://github.com/horizon-rust/horizon),
Minecraft Java Edition 26.3.

**Slice 1: the 1024k item storage.** One storage block and one storage disk, each
holding **1,048,576 items** in up to 256 resources, with insert, extract, contents
that survive a relaunch and the upstream refusal of an insert past capacity.

```text
/extradisks info <x> <y> <z>                      what the block holds
/extradisks insert <x> <y> <z> <item> <count>     put items in (admin: no inventory)
/extradisks extract <x> <y> <z> <item> <count>    take items out (into the inventory)
/extradisks disk info                             the held 1024k disk
/extradisks disk insert|extract <item> <count>    put in / take out of it
```

Right-click the block with a stack to store it, right-click it with an empty hand
to take a stack out. Breaking it gives its contents back to the breaker.

## What is not here yet

The other seven item rungs, the fluid and chemical ladders, the parts, the
casing and processors, the storage upgrades, **recipes** (they need Refined
Storage's own items), the **network** (the block is standalone storage, not a
Refined Storage node), the GUI and the energy use. The full list is in
[`notes.md`](notes.md#gaps--deviations-slice-1).

## Install

`extradisks.hmod` is one file and carries both halves: the content (`assets/`,
`data/` — the block, the disk item, models, textures, lang, loot, creative tab)
and the code (`src/`, compiled by the launcher against the pinned game build).
Put it in the launcher's mod list and play. It is built for Horizon
`0b14f8acfd97fa2c750e42795249dcc981178ca4` with horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0` (SDK/API `0.2`).

## Build and test

Everything runs in this folder; the SDK is fetched into `.horizon/sdk/` by the
first `cargo horizon-mod` command and is never packed.

```sh
CARGO_INCREMENTAL=0 cargo test                        # the pure modules' 26 tests
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
cargo fmt --check
rm -f Cargo.lock                                      # see notes.md, trap 5
CARGO_INCREMENTAL=0 cargo horizon-mod build           # -> target/horizon-mod/extradisks.hmod
cp target/horizon-mod/extradisks.hmod ./extradisks.hmod
cargo horizon-mod check extradisks.hmod               # the registry's own check
cargo horizon-mod fits extradisks.hmod --build nightly
```

`tools/e2e.sh` is the in-game scenario; it needs a launcher-built game carrying
this package and a `horizon-ctl`, which this machine does not have.

## Licence

Apache-2.0 (the original mod's licence, read from its `neoforge.mods.toml`); see
[`LICENSE`](LICENSE) and [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md), both inside the
package. The port is a rewrite from the mod's observed behaviour, its data files
and its public documentation — no Java code is translated.
