# Save system audit (2026-11-25)

Code: `crates/victory-hle/src/modules/xgamesave.rs` (XGameSave) and
`crates/victory-hle/src/modules/storage.rs` (persistent local storage).
Fixtures: synthetic only (containers named `slot` holding the bytes
`synthetic save`); no game saves were read, copied or committed.

## Where saves live

| Item | Value on this laptop |
|---|---|
| Runtime data directory | `%LOCALAPPDATA%\victory\data\<title id>` (`VICTORY_DATA_DIR` overrides; `Config::data_dir`) |
| Saves | `data\<title>\saves\<user>\<service configuration id>\<container folder>\v<N>\` |
| Persistent local storage | `data\<title>\storage\` (the title's `settings\settings.xml` lives here) |
| Crash reports and temp | `data\<title>\temp\`, `data\<title>\crashes.log` |

Containers here: a profile (`Player`, version 25), a story save
(`SARC0015`, version 1) and the settings profile (version 1). Their folders
held exactly one committed version each, and no leftovers.

## How it works

| Property | Implementation | Verified by |
|---|---|---|
| Serialization | a version folder holds blob `i` as file `i` plus a `manifest` with the exact container, display and blob names (so case, forbidden characters, device names and length never reach a file name) | `names_are_exact` |
| Profile isolation | a folder per user and service configuration; container folder = safe name + hash of the exact name | `saves_persist_across_providers`, `an_empty_configuration_id_is_the_default_one` |
| Transactions | each update writes a whole `v<N>.new/` (unchanged blobs hard-linked), `sync_all` per file, then one directory rename commits | `a_kill_at_any_step_leaves_old_or_new` (kills a child process at every step) |
| Crash recovery | `.new` leftovers are removed on sight; the newest readable version is the container | same, plus the tests below |
| Quota | 256 MB per user, 16 MB per update; a full disk and a full quota fail with their own errors | `a_full_disk_is_not_the_quota` |
| Older runtime layouts | moved over on first use | `the_old_layout_moves_over` |

## Defect found and fixed: a stray version folder deleted the save

- **Defect:** recovery took any `v<digits>` name as a version. A leftover
  `v01` next to the committed `v1` parsed as version 1, and the cleanup of
  "older" versions rebuilt the name as `v1` and deleted the committed save.
  An empty or unreadable newer folder (`v2`) also pruned `v1`, and the
  read then failed. Upstream reports: #870 (bug), #1729 (fix).
- **Reproduced here:** new test `stray_version_folders_never_cost_the_save`
  failed on this branch (the save was gone).
- **Fix:** upstream `b8dbcba2` (moe-miau, PR #1729, GPL-3.0-or-later),
  applied cleanly. Only `v<N>` exactly as `commit` writes it is a version;
  `current` takes the newest version whose manifest reads, removes only
  versions below it, and leaves unreadable newer entries alone; updates go
  above the highest name present. With no readable version, reads and
  updates fail and nothing is removed.
- **After:** all 11 save tests pass: the new one, upstream's three
  (`a_noncanonical_folder_does_not_remove_a_committed_save`,
  `an_unreadable_newer_version_does_not_cost_the_save`,
  `no_readable_version_is_refused_and_kept`) and the existing ones.

## Other runtimes

- DexNom (`8cca4d22`): "save handling flushes copied blobs before
  publication and can recover a complete checkpoint". This branch already
  syncs every file it writes before the rename; hard-linked blobs are files
  already on disk. Not ported: no defect reproduced here.
- Compatibility between runtime versions: the format (folders, manifest)
  is the same as upstream's after #1729, so saves move between this branch
  and upstream in both directions.
