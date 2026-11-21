# Extra Disks — validation ladder (0.2.1, slice 2)

Pin: SDK nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (2026-11-20), horizon-mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, API `0.2`, package format 6, launcher 0.2.92.
Delivered artifact: **38,022 B**, sha256 `100a271e7879a3e3ab54e4090bb21c8f0e689d7f1ab6e62255dfc29eeed07a1a`,
26 entries, no packed `Cargo.lock`.

Every compile/test step ran through `ports/tools/build-slot.sh` (`BUILD_LOCAL=1`,
`CARGO_INCREMENTAL=0`, no staging, no `--target-dir`). The game legs ran on `launcher-game.sh`'s
export of the Nightly at this pin. Evidence lives in `../_shared/evidence/extradisks/` and
`../_shared/evidence/extradisks-e2e/`; each of the four required logs prints the sha256 and size of
the artifact it ran against.

| # | Step | Command | Result |
|---|---|---|---|
| 1 | unit tests | `cargo test --offline` | **26 passed**, 0 failed — `sdk-test.log` |
| 2 | lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** — `sdk-clippy.log` |
| 3 | format | `cargo fmt --check` | exit 0 — `sdk-fmt.log` |
| 4 | package (x2) | `rm -f Cargo.lock && cargo horizon-mod build` twice | 38,022 B both, `cmp`-identical, 26 entries |
| 5 | registry check | `cargo horizon-mod check extradisks.hmod` | `extradisks 0.2.1 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, EXIT 0 — `sdk-check.log`, `sdk-check-json.log` |
| 6 | fits the pin | `cargo horizon-mod fits extradisks.hmod --build 0a875da` | `0a875da  made for`, EXIT 0 — `sdk-fits.log` |
| 7 | fits another build | `… --build 0b14f8a` | **EXIT 1**, scanner false positive — `sdk-fits-other.log` + `fits-other-FALSE-POSITIVE.md` |
| 8 | package gate | `python3 ports/tools/verify-sdk-package.py extradisks` | **1/1** — `verify-sdk-package.log` |
| 9 | in-game leg | `tools/e2e.sh` on the launcher build | **PASS + EXIT:0** — `extradisks-e2e/e2e-launcher.log` |
| 10 | counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL:` + **EXIT:1** — `extradisks-e2e/e2e-selftest.log` |
| 11 | control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL:` + **EXIT:1** — `extradisks-e2e/e2e-control.log` |
| 12 | red proof | the loot assertion before the fix | `FAIL: the storage block dropped 0 of itself` (vanilla barrel 1) — `extradisks-e2e/e2e-loot-red.log`; static red in `data-shape-loot-red.log` |
| 13 | log gate | `python3 ports/tools/check-e2e-logs.py extradisks-e2e --strict` | `logs=4 ok=4`, exit 0 |
| 14 | skills gate | `python3 ports/tools/check-skills.py --notes ports/extradisks` | PASS |

Step 7 is red for a reason outside the package: the build's usage scanner reads the
`CommandArgumentValue::Item(stack)` match arm at `src/commands.rs:80` as a literal construction of the
unrelated non-constructible type `Item`, and `fits` refuses it. `type:Item` is byte-identical in both
builds' surfaces; the pin run, `check`, and the in-game `item` argument are green. Full proof and the
upstream-report wording are in `fits-other-FALSE-POSITIVE.md`. Silencing it would mean dropping the
command's item argument (and the component-carrying storage it enables), so it is reported, not
worked around.
