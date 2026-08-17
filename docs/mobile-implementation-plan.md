# Mobile Implementation Plan — Lunar (Tauri v2)

> **Status:** Draft
> **Platform:** Android + iOS simultaneously
> **Last updated:** 2026-08-17

## Overview

This document outlines the plan to complete Tauri v2 mobile support for Lunar, covering Android and iOS. The project already has generated Android/iOS projects, a responsive UI, and the foundational Tauri mobile entry point. This plan addresses what remains: config, build pipeline, UI polish, feature gating, bidirectional sync, and release.

### Decisions

| Decision | Choice |
|---|---|
| Target platforms | Android + iOS simultaneously |
| Alarm system | Disabled on mobile for now |
| Ollama AI | Excluded from mobile |
| Sync behavior | Full bidirectional sync |

---

## Phase 1: Mobile Foundation (Config & Build)

### 1.1 — Mobile capabilities file

**Create** `src-tauri/capabilities/mobile.json`

Grant mobile-appropriate permissions. No window-state, no close/minimize/maximize (those are desktop concepts). Include core permissions plus any future mobile-specific features.

### 1.2 — Android/iOS config in tauri.conf.json

**Edit** `src-tauri/tauri.conf.json`

- Add `"android": { ... }` section with package config, minimum SDK, etc.
- Add `"iOS": { ... }` section with bundle config, minimum iOS version
- Sync version numbers — currently `tauri.conf.json` says `0.8.32` but `tauri.properties` and `Info.plist` say `0.8.31`

### 1.3 — Fix tauri-plugin-stronghold dead code

`tauri-plugin-stronghold` is declared in `Cargo.toml` and `package.json` but never initialized in `lib.rs`. Options:

- **Initialize it** if needed for sync server config storage on mobile (currently Stronghold is used in `stores/backup-settings.ts` for sync server config)
- **Remove it** if not needed

### 1.4 — Add mobile build commands to Justfile

Add to `scripts/console.just`:

```just
[working-directory: 'console']
build-android:
    npm run tauri android build

[working-directory: 'console']
build-ios:
    npm run tauri ios build

[working-directory: 'console']
build-android-dev:
    npm run tauri android build --debug
```

### 1.5 — Add proper mobile app icons

Current icons are desktop-focused (`.icns`, `.ico`). Need:

- **Android:** Adaptive icons (mdpi through xxxhdpi, anydpi-v26)
- **iOS:** All required icon sizes for App Store + device home screen

---

## Phase 2: Mobile-Adaptive UI Polish

### 2.1 — Auth layout on mobile

**File:** `layouts/auth.vue`

- `md:p-6` removes padding on mobile — verify edge-to-edge rendering works
- `rounded-xl` is applied only in Tauri (`!IS_WEB`) — on mobile Tauri, this should NOT apply since there's no window frame
- Decorative blur blobs are `hidden md:block` (good)

### 2.2 — Titlebar on mobile

**File:** `components/app/titlebar.vue`

- macOS traffic lights gated by `isMacOS && !IS_WEB` — good
- On mobile Tauri, the titlebar should be minimal: hamburger + app name only
- Desktop window controls (close/minimize/maximize) must be hidden on mobile
- Consider adding platform detection (Tauri v2 can detect OS via `@tauri-apps/plugin-os`)

### 2.3 — Workspace selector on mobile

**File:** `components/app/titlebar.vue`, `components/workspace/select.vue`

- `WorkspaceSelect` is `hidden md:block` — no way to switch workspaces on mobile
- **Solution:** Add workspace switcher to the mobile nav drawer (`USlideover` in `layouts/default.vue`)

### 2.4 — Side panel handling

**File:** `layouts/default.vue`

- Desktop aside is `hidden md:flex`, mobile uses right `USlideover`
- Verify all pages using `side_content` slot work in the mobile USlideover
- Pages to test: moodboard lightbox, note editor, settings panels

### 2.5 — Card action buttons

Current pattern: `opacity-100 md:opacity-0 md:group-hover:opacity-100`

- Always visible on mobile (no hover state) — correct
- Verify this pattern is consistent across ALL card components:
  - `components/todo/todo-card.vue`
  - `components/bookmark/card.vue`
  - `components/settings/worksaces/workspace-card.vue`

### 2.6 — Safe area handling

Current safe areas are only in `layouts/default.vue`:
- `env(safe-area-inset-top)` on mobile nav drawers
- `env(safe-area-inset-bottom)` on FAB container

**Gaps:**
- Pages with fixed bottom elements need `env(safe-area-inset-bottom)` padding
- The `p-6` padding on `<main>` may need `pb-[calc(1.5rem+env(safe-area-inset-bottom))]` on mobile
- `layouts/auth.vue` has no safe area handling

---

## Phase 3: Mobile Feature Gating

### 3.1 — Exclude Ollama/AI from mobile

- `/ollama` route is already commented out in `routes.ts` — verify no navigation entry points to it
- Add a route guard or platform check to prevent direct URL navigation to AI pages on mobile
- Remove or hide Ollama entry from any navigation menus on mobile

### 3.2 — Disable alarms on mobile

- Alarm commands are already commented out in `lib.rs` and `commands/mod.rs` — no change needed
- `alarm-settings.vue` imports `invoke` from `@tauri-apps/api/core` directly — should gracefully degrade
- Hide alarm settings on mobile or show "coming soon" state
- Reminder creation should still work, but alarm scheduling should be disabled on mobile

### 3.3 — Moodboard on mobile

- `save_moodboard_image` / `delete_moodboard_image` use filesystem commands — verify paths resolve in mobile sandboxed storage
- Upload UI needs camera/gallery integration for mobile (currently just file picker)

---

## Phase 4: Bidirectional Sync (Major Feature)

> This is the largest effort. The current sync is push-only (client → server). Bidirectional sync requires pull (server → client) and conflict resolution.

### 4.1 — Server-side change tracking

**Add DB triggers on server PostgreSQL tables** (same pattern as client):

- Create a `server_sync_queue` table on the server (or reuse `sync_queue` with a `source` column)
- Add triggers on all 8 entity tables to track INSERT/UPDATE/DELETE
- Each entry: `identifier`, `table_name`, `record_identifier`, `operation`, `created_at`, `source_client_id`

**Alternative (simpler):** Poll-based approach — query `WHERE updated_at > last_sync_timestamp`. Less real-time but much simpler to implement.

### 4.2 — Add client_id to sync_queue

- Add a `client_id` column (UUID) to the `sync_queue` table on both client and server
- Prevents echo loops (client A pushes → server stores → client B pulls → client B pushes back → infinite loop)
- Each device generates a unique `client_id` on first launch, stored in app data

### 4.3 — Server-side pull queries (GraphQL)

Add new GraphQL queries for each entity:

```graphql
type Query {
  pull_notes(since: String!, client_id: String!): [SyncNoteInput!]!
  pull_bookmarks(since: String!, client_id: String!): [SyncBookmarkInput!]!
  pull_snippets(since: String!, client_id: String!): [SyncSnippetInput!]!
  pull_todos(since: String!, client_id: String!): [SyncTodoInput!]!
  pull_reminders(since: String!, client_id: String!): [SyncReminderInput!]!
  pull_workspaces(since: String!, client_id: String!): [SyncWorkspaceInput!]!
  pull_recycle_bin(since: String!, client_id: String!): [SyncRecycleBinInput!]!
  pull_workspace_preferences(since: String!, client_id: String!): [SyncWorkspacePreferenceInput!]!
}
```

Each query reads from the server's sync_queue, joins with the entity table, excludes entries from the requesting `client_id`, and returns the records.

### 4.4 — Conflict resolution strategy

- Use **last-write-wins (LWW)** based on `updated_at` timestamp
- The `filter_stale_items` function in `server/src/mutations/sync_queue.rs` already has timestamp comparison — extend it
- **DELETE conflicts:** if client has a record the server doesn't, it was deleted server-side → apply delete locally
- **Concurrent edits:** the one with the latest `updated_at` wins

### 4.5 — Client-side pull implementation

Add a `pull()` method to each frontend store (notes, bookmarks, etc.). The `sync-queue.ts` orchestrator calls `pull()` after all pushes complete.

**Pull flow:**

1. Read `last_sync_timestamp` from localStorage (per-entity or global)
2. Call `pull_*` GraphQL query with `since` timestamp and `client_id`
3. For each returned record: upsert locally
4. Update `last_sync_timestamp`
5. Handle DELETE operations: remove records locally if server signals deletion

### 4.6 — Client-side sync state persistence

Add a `sync_state` table or localStorage entry tracking:

- `client_id` (UUID, generated once)
- `last_push_timestamp` (when we last pushed)
- `last_pull_timestamp` (when we last pulled)

### 4.7 — DELETE handling fix

Current `extract_unsynced()` tries to fetch records by identifier from the entity table — fails for deleted records since they no longer exist.

**Fix:** For DELETE operations in sync_queue, send the `record_identifier` + `operation` without trying to fetch the (now-deleted) record. Server should handle DELETE by removing the record if it exists.

### 4.8 — Fix bookmark sync bug

**File:** `lunar/src/repositories/bookmarks.rs` line ~270

The `TableName` filter is commented out:

```rust
// .filter(sync_queue::Column::TableName.eq("bookmarks"))
```

Uncomment it. Currently bookmarks sync mixes with other table entries.

### 4.9 — Server-side echo prevention

- When server receives a push mutation, record the `client_id` in the server's sync_queue
- When serving pull queries, exclude entries originating from the same `client_id`
- This prevents device A's changes from being pulled back by device A

---

## Phase 5: Mobile-Specific Tauri Commands

### 5.1 — Platform detection command

Add a `get_platform` Tauri command that returns `"android"`, `"ios"`, or `"desktop"`. Frontend uses this to conditionally show/hide features.

### 5.2 — File system paths

Verify `app_data_dir()` resolves correctly on Android and iOS. The `moodboard` and `export_notes_as_pdf` commands use filesystem paths — ensure they work in mobile sandboxed storage.

### 5.3 — Notification permissions

On mobile, Tauri notification plugin needs runtime permission request. Add permission request on first launch for Android 13+ and iOS.

---

## Phase 6: Testing & Release

### 6.1 — Android release build

- Configure signing (keystore)
- Set up `tauri.conf.json` bundle targets for `.aab` (Google Play)
- Test on physical device

### 6.2 — iOS release build

- Configure provisioning profiles and signing
- Set up `ExportOptions.plist` for App Store distribution
- Test on physical device

### 6.3 — CI/CD for mobile

- Add GitHub Actions jobs for Android and iOS builds
- Automate version bumping across `tauri.conf.json`, `tauri.properties`, `Info.plist`

---

## Implementation Order

```
Phase 1 (Foundation)      ████░░░░░░░░░░░░░░░░  1-2 days
Phase 2 (UI Polish)       ██████░░░░░░░░░░░░░░  2-3 days
Phase 3 (Feature Gating)  ██░░░░░░░░░░░░░░░░░░  0.5 day
Phase 4 (Sync)            ████████████████████  5-7 days  ← biggest effort
Phase 5 (Mobile Cmds)     ██░░░░░░░░░░░░░░░░░░  1 day
Phase 6 (Testing/Release) ████░░░░░░░░░░░░░░░░  2-3 days
                          Total: ~12-17 days
```

**Recommended approach:** Complete Phases 1-3 and 5 first to get a working mobile build, then tackle Phase 4 (bidirectional sync) as a separate focused effort.

---

## Key Files Reference

| File | Relevance |
|---|---|
| `src-tauri/tauri.conf.json` | Tauri config — needs mobile sections |
| `src-tauri/capabilities/default.json` | Universal permissions |
| `src-tauri/src/lib.rs` | Mobile entry point, plugin init |
| `src-tauri/Cargo.toml` | Rust dependencies, mobile gates |
| `console/nuxt.config.ts` | Viewport meta, modules |
| `layouts/default.vue` | Mobile nav drawers, safe areas, FAB |
| `layouts/auth.vue` | Auth layout mobile adaptation |
| `components/app/titlebar.vue` | Hamburger menu, mobile controls |
| `components/navigation/app.vue` | Desktop sidebar (`hidden md:flex`) |
| `components/app/primary-cta.vue` | Desktop CTA + mobile FAB |
| `composables/useMobileNav.ts` | Mobile nav toggle state |
| `app/plugins/lunar.client.ts` | IS_WEB / IS_TAURI detection |
| `app/utils/invoke.ts` | UNSUPPORTED set, command routing |
| `app/stores/sync-queue.ts` | Sync orchestrator — needs pull logic |
| `lunar/src/sync_engine.rs` | Sync engine types |
| `lunar/src/entities/sync_queue.rs` | Sync queue entity |
| `lunar/src/repositories/sync_queue.rs` | Sync queue repository |
| `lunar/src/repositories/notes.rs` | extract_unsynced / clear_synced pattern |
| `lunar/src/repositories/bookmarks.rs` | Has table_name filter bug |
| `server/src/mutations/sync_queue.rs` | Stale filtering (incomplete) |
| `server/src/query_root.rs` | GraphQL schema registration |
| `scripts/console.just` | Build commands — needs mobile targets |
| `console/src-tauri/gen/android/` | Generated Android project |
| `console/src-tauri/gen/apple/` | Generated iOS project |

---

## Known Issues to Fix

1. **Version drift:** `tauri.conf.json` = 0.8.32, `tauri.properties` = 0.8.31, `Info.plist` = 0.8.31
2. **Bookmark sync bug:** Table name filter commented out in `lunar/src/repositories/bookmarks.rs:270`
3. **Stronghold dead code:** Declared but never initialized
4. **DELETE handling:** `extract_unsynced()` fails for deleted records (tries to fetch non-existent rows)
5. **Server sync_queue.rs:** `filter_stale_items` compares timestamps but doesn't apply the results
6. **Note categories:** `note_category.graphql` defines mutation but no server handler exists
