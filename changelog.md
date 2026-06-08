# Changelog

## [Unreleased]

---

## [0.3.0] - 2026-06-08

### Added

#### SQLx persistence layer (`lib/db/`)
- Migrated from bincode to SQLx for all player save data.
- `mark_dirty` tracking on every mutable subsystem; a background interval flushes only changed data so handlers no longer need explicit save calls on every mutation.
- New `lib/db/src/traits/pending.rs`: `Pending` trait shared across all dirty-tracking subsystems.
- New migration `lib/db/migrations/0002_wallet.sql` for the wallet table.
- New `lib/db/src/subsystems/wallet.rs` subsystem module.

#### Wallet system (`lib/logic/src/wallet.rs`, `lib/db/src/subsystems/wallet.rs`)
- `WalletManager` persisting gold and diamond balances across sessions.
- Enforced spending: `spend_gold` / `spend_diamond` return `Err` when the balance would go negative -  no silent underflow.
- Wired into `Player` and the login handler; weapon breakthrough now deducts the correct currency cost on success.

#### Server-side validation
- `NetContext::send_error(Code, details)` helper pushes `ScNtfErrorCode` to the client on any validation failure, replacing ad-hoc silent drops.
- `ErrCode` proto enum added to cover all server-generated error codes.
- **character/battle**: reject `battle-info` updates for object IDs not in the active team; HP clamped to `[0, max_hp]` from asset tables; SP clamped to `[0, 1.0]`.
- **character/team**: `set_team` requests containing object IDs not owned by the player rejected; control characters stripped from team names with a 20-char hard cap.
- **scene/revival**: `kill_char` rejected for IDs outside the current active team; `kill_monster` for non-enemy entity types (NPCs, interactives) silently ignored instead of corrupting state.
- **scene consistency**: additional guards across load, revival, and teleport paths to prevent desync.
- **skill ownership** validated before use; equip slot integrity and mission/guide input bounds enforced.
- **movement**: non-finite coordinates rejected; position deltas exceeding the impossible-speed threshold are capped before applying.
- **weapons**: weapon add-exp validation extracted into `perlica_logic` for reuse; redundant `ScItemBagSyncModify` push on weapon update removed.

#### Abstraction / trait layer (`lib/logic/src/traits/`)
- `Item`, `World`, and `Container` trait families introduced to unify access patterns across weapon, gem, equip, and stackable subsystems.
- Abstraction extended to gem/equip ID allocators, `MailManager`, and other non-item player facets.
- Inherent methods on concrete item types removed in favour of trait dispatch; call sites updated across the codebase.

#### Interest manager (`lib/logic/src/interest.rs`)
- Multi-tiered entity replication / interest manager. Four concentric zones (Immediate / Combat / Distant / Background) at 40 / 80 / 150 / 250 wu enter radii, each with its own check frequency (16 / 50 / 160 / 500 ms) and hysteresis-based leave radii (+60 wu, up from +25).
- `StreamBucket` enum (`Enemy` / `Interactive` / `Npc`) with per-bucket max-zone, per-tick spawn-budget, and concurrent-cap config.
- Adaptive query radius: `max_due_radius_for(max_zone)` returns the world-space radius of the outermost currently-due zone clamped by per-kind cap, so most ticks query only 40 wu instead of 275.
- O(1) `live_count: [usize; 3]` per bucket maintained alongside `entries`; `at_capacity(bucket)` is a single array index.
- Fast-mover predictive-radius bonus (+40 wu) when the player's EMA-tracked speed exceeds 20 wu/s.
- Inlined FxHash-style `BuildHasher` for `u64` keys.
- Height-band occlusion heuristic (`is_occluded`) for Zone 0 with a per-tick `OcclusionCache` (16 ms TTL).
- Always-resident entries via `ghost_in_resident()`: TPs, save points, dungeon entries, blockages, and doors exempt from leave-radius and orphan-sweep passes.
- Open-world combat-stickiness: `last_close_ms` per entry bumped automatically when an entity is observed at Combat range or closer; `should_retain` keeps recently-engaged enemies (within 5 s of last close sighting) inside `COMBAT_STICKY_MAX_RADIUS = 200 wu` even past the normal leave radius -  works without any client-side `in_battle` flag (dungeon-only iirc). Dungeon `in_battle = true` still triggers the same path as a fallback.

#### Spatial grid (`lib/logic/src/spatial.rs`)
- 2-D XZ spatial grid with bucketed `query_radius_indices`. One grid per streamed bucket inside `SceneCache`; rebuilt once per scene transition.

#### Equip system (`lib/config/src/equip.rs`, `lib/config/src/tables/equip.rs`, `servers/game-server/src/handlers/equip.rs`)
- `EquipBasicTable` and `EquipAttrTable` config structs backed by `assets/tables/Equip.json` (~5.4k entries).
- Slot-aware puton/putoff flow with `partType -> CraftShowingType` mapping corrected.
- Already-equipped-on-same-character is now a no-op instead of an error.
- Previous-owner tracking on equip swaps.
- `EquipDepot::compute_suitinfo()` for set-bonus computation (`lib/logic/src/item.rs`).

#### Mission & level-script improvements
- Corrected mission config parsing; field names in `lib/config/src/mission.rs` aligned with the data files.
- `lib/logic/src/level_script.rs` expanded (+342 lines): quest-drive logic wired end-to-end; level-script event dispatch extended across teleport, entity lifecycle, and trigger events.
- `servers/game-server/src/handlers/scene/level_script.rs` expanded (+230 lines): handler-side quest triggers hooked up to the logic layer.
- `sconfig.rs` extended to expose the new level-script config surface.

#### Unit test suite (`lib/logic/src/` -  3,669 lines added)
- Full test coverage for every module in `perlica-logic`: `bitset`, `char_bag`, `progression`, `entity`, `enums`, `error`, `level_script`, `mail`, `mission`, `movement`, `player`, `scene`, `spatial`, `wallet`, and all `traits` extensions.
- Complements the 30+ interest-manager tests already in `lib/logic/src/interest.rs`.

#### Docker
- `Dockerfile`, `docker-compose.yml`, and `docker-entrypoint.sh` added for containerised deployment.

### Changed

#### Handler module splits
- `servers/game-server/src/handlers/scene.rs` (600+ lines) split into `scene/{mod,dialog,entity,level_script,load,revival,teleport}.rs`.
- `servers/game-server/src/handlers/character.rs` split into `character/{mod,battle,progression,skill,team}.rs`.
- `servers/game-server/src/handlers/weapon.rs` split into `weapon/{mod,exp,equip,breakthrough,gem}.rs`.
- No behaviour changes; purely structural to reduce cognitive load for contributors.

#### Scene streaming (`lib/logic/src/scene.rs`) 
- Replaced bulk-spawn at scene load with proximity-based streaming for enemies, interactives, and NPCs. `finish_scene_load` and `handle_revival` no longer dump every interactive on the map (often 200+ on map01_lv001).
- `SceneCache` now builds three spatial grids (enemies / interactives / NPCs) at 50 wu cell size.
- `update_visible_entities` rewritten as three streaming helpers (`stream_enemies` / `stream_interactives` / `stream_npcs`) sharing the same skeleton: clamped query radius -> 3-D distance check -> capped zone classify -> single-probe `touch_or_classify` -> spawn-budget gate -> capacity gate -> ghost-in.
- Unified ghost-out pass walks `interest.entries` directly instead of `entities.monsters()`, saving one HashMap probe per ghosted-in entity.
- Reusable scratch buffers (`candidates_buf`, `leave_ids_buf`) owned by `SceneManager` -  zero allocation in steady state on the per-tick hot path.
- `pack_resident_interactives` / `install_resident_interactives` introduced to send the always-resident subset (TPs, blockages, doors, save points, dungeon entries) at scene load while streaming the rest.
- `SceneManager::on_entity_killed` and `on_entity_despawned` keep the interest counter atomically in sync after a kill or despawn event.
- `interactives()` and `npcs()` iterator helpers added to `lib/logic/src/entity.rs`, symmetric with `monsters()` and `characters()`.

#### Performance
- Hot-path scene data cached; `BTreeMap` replaced with `HashMap` across entity and scene structures; per-tick movement allocation eliminated.
- Domain math utilities (`wu` conversions, distance helpers) extracted into `perlica_logic` for reuse across modules.

#### Error handling
- `LogicError::InvalidOperation` replaced with purpose-specific error variants.
- All non-fatal `.notify` / `.send` call sites have `.unwrap()` removed; these methods now log failures internally. Fatal code paths that must panic retain `.unwrap()`.

### Fixed

- **fix(scene)**: monster disappearance mid-fight. Two compounding causes addressed:
 - The kill-monster handler removed the entity from `EntityManager` but never cleared the matching interest entry, leaving `live_count[Enemy]` inflated for up to 500 ms. Once the cap was reached, the streamer refused to refresh the active fight. Now atomically cleaned via `on_entity_killed` from both kill/destroy paths.
 - The previous +25 wu hysteresis was too tight -  a single laggy movement packet past the Combat leave radius evicted an actively-fought enemy. Hysteresis bumped to +60 wu; time-based stickiness retains recently-engaged enemies inside 200 wu for 5 s after last sighting.
- **fix(scene)**: scene-load FPS hitch. Interactives are no longer bulk-spawned on scene entry or revival -  only navigation-critical residents are sent up-front.
- **fix(equip)**: slot mapping, `suitinfo` computation, and attr loading on login.
- **fix(weapon)**: exp table field name, breakthrough level-cap off-by-one, and failed-breakthrough semantics corrected.
- **fix(muip)**: GM listener was reading from the wrong config section (`[muip]` instead of `[muip_gm]`).
- **fix(weapons)**: redundant `ScItemBagSyncModify` push on weapon exp update removed.
- Removed leftover `to_xxx` helpers in `item.rs` superseded by the `From`/`Into` conversions introduced in 0.2.0.

---

## [0.2.0] - 2026-04-13

### Added

#### Mail system (`lib/logic/src/mail.rs`, `servers/game-server/src/handlers/mail.rs`)
- `StoredMail` and `MailManager` with expiry, attachment state, and CRUD ops.
- Canned welcome and login-greeting mail factories.
- `push_mail_sync`, `deliver_login_mails`, plus handlers for get/read/delete/claim mail and attachments.
- `LoginPhase` now has a `Mail` stage between `Bitsets` and `EnterScene` that pushes `ScSyncAllMail` and delivers welcome mail for new players or greeting mail for returning ones.
- `Player` stores a `MailManager` and a transient `is_new_player` flag set during login from whether the DB returned an existing record.
- `PlayerRecord` and `PlayerRecordRef` persist `MailManager` with `serde(default)` so older saves still load.

#### GM console / MUIP (`lib/muip/`, `servers/game-server/src/{gm.rs,handlers/gm.rs}`)
- New `lib/muip` crate and a 365-line `handlers/gm.rs` providing live testing commands for the various game systems.
- `assets/tables/Index.json` (~1.6k entries) added to support GM lookups.
- New `perlica-muip-server` binary mentioned in the README install steps.

#### Mission and Guide systems
- `MissionManager` and `GuideManager` added to `PlayerRecord` and `Player`.
- New `lib/config/src/mission.rs` config schema.
- Mission/guide command handlers wired into the game-server router.
- Locale tables landed: `assets/tables/I18nTextTable_EN.json` and `assets/tables/TextTable.json`.

#### Item system rewrite (`lib/logic/src/item.rs`, `lib/config/src/item.rs`)
- `WeaponDepot` generalised into `ItemManager` covering weapons, gems, equip, and stackables.
- `WeaponInstance`, `GemInstance`, `EquipInstance`, `WeaponDepot`, `GemDepot`, `EquipDepot`, and `StackableDepot` now use idiomatic `From`/`Into` conversions instead of bespoke `to_xxx` helpers.
- `WeaponAttachGemArgs`, `WeaponDetachGemArgs`, `WeaponPutonArgs` convert to their matching `Sc*` proto messages via `From`.
- `WeaponInstId` converts to and from `u64`.
- `assets/tables/Item.json` (~19.6k entries) added.

#### Equip handler (`servers/game-server/src/handlers/equip.rs`)
- New equipment puton/putoff handler set and wallet handler.

#### Character const (`lib/config/src/character.rs`)
- `CharacterConst` for global leveling data.

#### Scene: dynamic visibility and respawn (`lib/logic/src/scene.rs`)
- Replaced full-scene monster spawning with radius-based dynamic visibility driven by `EnterView`/`LeaveView` events.
- 60s respawn cooldown for killed monsters keyed by `level_logic_id`.
- 80/100 unit hysteresis to prevent entity flickering at vision edges.
- Visibility checks integrated into authoritative movement and scene-load handlers.
- Per-level spawn data in `assets/level_data/map01_lv001_lv_data_sub01.json` with a new `lib/config/src/{level_data,tables/level_data}.rs` schema.

#### Level scripts (`lib/logic/src/level_script.rs`)
- New ~415 line module covering teleportation, entity lifecycle, and level script events.
- Packet validation relaxed to allow empty bodies for certain command types.

#### Save layer
- `PlayerRecordRef` introduced so `PlayerDb::save` doesn't have to clone the whole `PlayerRecord` to serialise.

#### Project meta
- `README.md` and `CONTRIBUTING.md` added.
- `LICENSE`: GNU AGPL v3.
- CI workflow `.github/workflows/rust.yml`: fmt + clippy + build + test on `ubuntu-latest` and `windows-latest`; release-mode prebuilts as artifacts; rolling `dev-<sha>` pre-release on every master push; tagged release with linux/windows binaries on `v*` tags.
- Discord link replaced with a permanent invite.
- `assets/img/sleep.png` added for README.

### Changed

#### Errors: `anyhow` -> typed `thiserror`
- New error enums in `lib/config/src/error.rs`, `lib/db/src/error.rs`, `lib/logic/src/error.rs`.
- Call sites across `config/{character,id_to_str,level_data,skill,str_to_id,weapon}`, `db/saves`, `logic/{character/char_bag,item}`, and the game-server handlers updated.
- Added `InvalidStructure` (config) and `Insufficient` (logic) variants so callers can branch on missing items or bad JSON without string matching.

#### Entity IDs
- Arbitrary IDs for NPCs, enemies, and interactives removed; logic IDs are used directly so level scripts and inter-entity interactions trigger correctly.
- `EntityDestroyReason` changed from `Immediately` to `Dead`.

#### Scene: spawn data replaced
- `assets/tables/EnemySpawns.json` removed in favour of per-level `assets/level_data/` files (see Scene entry above).

#### Factory / scene handler
- Factory now sends needed dummy data plus interacts, NPCs, etc. so the map loads correctly client-side.
- `DynamicParam` parsing fixed.

#### Workspace
- `Cargo.toml` `[workspace.package].version` bumped to `0.2.0`.

### Fixed

- **Revival flow**: scene handler revival path corrected; factory route registered.
- **`charBag` team UI crash**: client crashed when `max_indexes` tail entries were omitted. Side effect: editing a whole team no longer reloads the entire scene.
- **`set_team` leader**: when `CsCharBagSetTeam` removed the current leader, `leader_index` was left pointing at a missing character, causing `move_leader_to_front` to silently no-op and the client to receive a `ScSelfSceneInfo` with a `leader_id` matching no character - triggering "Can not find main character in SC_SELF_SCENE_INFO". The first occupied slot is now promoted to leader whenever the existing one is removed.
- Infinite loading on The Hub; missing enemies added; enemy logic fixes (contributed by inkursion).
- All clippy warnings cleared.

---

## [0.1.0] - 2026-03-18

### Added

- Weapon depot (`WeaponDepot`, `WeaponInstance`) with experience, breakthrough, and gem attach/detach handlers.
- Scene system: entity manager, monster spawning, NPC and interactive entity support, authoritative movement.
- Bitset persistence: player progress flags saved to and restored from the DB on login/logout.
- Initial game-server handler set: scene load, revival, teleport, dialog, and entity interactions.
- Core `PlayerRecord` / `PlayerDb` save layer backed by an embedded database.
