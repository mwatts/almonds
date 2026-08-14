# Lunar

A Rust data-access layer (built on SeaORM) compiled to **WebAssembly** so it can be driven from JavaScript, and equally usable **natively** from Rust.

It ships three layers:

- **entities** — SeaORM models for every table (`src/entities/`). They also export TypeScript types via `ts-rs` into `bindings/*.ts`.
- **adapters** — typed create/update payload DTOs, all `camelCase` (e.g. `CreateNote`, `UpdateWorkspace`).
- **repositories** — workspace-scoped CRUD (`NotesRepository`, `WorkspaceRepository`, …). The same structs power native Rust callers and are exported to JS through `#[wasm_bindgen]` wrapper methods.

```
┌─────────────────────────── WASM (browser/JS) ───────────────────────────┐
│  new NotesRepository()  →  async methods → Promise<Model | Model[]>     │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │
             #[wasm_bindgen] wrapper (repositories/*.rs)
                                │
                     async-trait repository (sea-orm)
                                │
                          DatabaseConnection
                                │
              MockDatabase (wasm)     Postgres via DataEngine (native)
```

> **Important (read before wiring a real DB):** the current browser build backs every repository with a SeaORM `MockDatabase` (see `utils::mock_connection`). The JS API is fully usable and type-checked, but **no real SQL executes in the browser**. PGlite cannot back SeaORM on `wasm32` today — see [PGlite (browser Postgres)](#pglite-browser-postgres) for the practical patterns.

---

## Project layout

```
lunar/
├── src/
│   ├── entities/            # SeaORM models (also generate bindings/*.ts)
│   ├── adapters/            # camelCase create/update DTOs + RequestMeta
│   ├── repositories/        # async-trait repos + #[wasm_bindgen] JS wrappers
│   ├── data_engine.rs       # native connect + migrations
│   ├── utils.rs             # mock_connection / wasm helpers
│   └── error.rs             # LunarError (serialized to JS on rejection)
├── bindings/                # generated TS types (ts-rs)
├── index.ts                 # re-exports the bindings (auto-generated)
├── pkg/                     # wasm-pack output (lunar.js, lunar_bg.wasm, lunar.d.ts)
└── migration/               # native-only migration crate
```

---

## Building

### Prerequisites

- Rust toolchain
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)

### Native (Rust)

```bash
cargo check
cargo build
```

### WebAssembly

```bash
# dev profile (matches pkg/ artifacts checked into the repo)
wasm-pack build --target web --dev

# release
wasm-pack build --target web --release
```

Output goes to `pkg/`:

```
pkg/
├── lunar.js        # ESM entry
├── lunar_bg.wasm
├── lunar.d.ts      # TypeScript types for the JS API
└── package.json    # {"name": "lunar", "type": "module", ...}
```

### Consuming `pkg/` in a JS app

Any of:

- copy `pkg/` into your app (e.g. `libs/lunar/`),
- add it as a workspace dependency (`"lunar": "file:../lunar/pkg"` or a pnpm workspace member),
- publish it to a registry.

---

## Repository JS API

### Initialize the WASM module

```ts
import init, { NotesRepository } from "lunar/pkg/lunar.js";

await init(); // resolves lunar_bg.wasm relative to lunar.js
```

If the `.wasm` is served from a known path instead (e.g. under `public/`):

```ts
await init({ module_or_path: "/lunar_bg.wasm" });
```

### Conventions

- Every repository is constructed with `new RepositoryName()` — it creates a `MockDatabase`-backed connection. There are **no constructor arguments**.
- Every method is **async** and returns a `Promise`.
- Payloads and `meta` are plain **camelCase objects** (see [Payloads](#payloads) and [RequestMeta](#requestmeta)).
- **UUIDs are strings.** `meta = { workspaceIdentifier: string } | null`.
- Success resolves to a serialized `Model` (camelCase) or `Model[]`, or `undefined` for void operations.
- Failures reject with a serialized `LunarError` object, e.g. `{ DbOperationError: "note not found" }`. Variants: `DbConnectError`, `DbOperationError`, `EnvError`, `WorkspaceNotFound`, `BookmarkNotFound`, `NotesNotFound`, `TodoNotFound`, `SnippetNotFound`, `ReminderNotFound`, `NotificationNotFound`.
- **Enums serialize as PascalCase strings**, e.g. `"Todo"`, `"High"`, `"Development"` — matching the `bindings/sea_orm_active_enums.ts` types. There is no camelCase renaming on enums.

### WorkspaceRepository

| Method | Returns |
| --- | --- |
| `create_workspace(payload)` | `Workspaces` |
| `get_workspace_by_id(identifier)` | `Workspaces` |
| `list_workspaces()` | `Workspaces[]` |
| `update_workspace(identifier, payload)` | `Workspaces` |
| `delete_workspace(identifier, meta)` | `void` |
| `verify_workspace_password(identifier, password)` | `boolean` |
| `exists(identifier)` | `boolean` |

### NotesRepository / BookmarkRepository / SnippetRepository / ReminderRepository

Common methods (payload type varies by repo):

| Method | Returns |
| --- | --- |
| `create(payload, meta)` | `Model` |
| `find_by_id(identifier, meta)` | `Model \| null` |
| `find_all(meta)` | `Model[]` |
| `recently_added(meta)` | `Model[]` (notes/bookmarks/snippets) |
| `update(identifier, payload, meta)` | `Model` |
| `delete(identifier, meta)` | `void` |

BookmarkRepository additionally has `find_by_tag(tag, meta)` (`Model[]`) and `exists(identifier)` (`boolean`).

### TodoRepository

| Method | Returns |
| --- | --- |
| `create_todo(payload, meta)` | `Todo` |
| `find_by_id(identifier, meta)` | `Todo \| null` |
| `find_all(meta)` | `Todo[]` |
| `update(identifier, payload, meta)` | `Todo` |
| `delete(identifier, meta)` | `void` |
| `change_priority(identifier, priority, meta)` | `Todo` |
| `update_due_date(identifier, dueDate, meta)` | `Todo` |
| `mark_done(identifier, done, meta)` | `Todo` |

`dueDate` is a `YYYY-MM-DD` string or `null`; `priority` is `"High" | "Medium" | "Low"`.

### NotificationRepository

`create(payload, meta)` → `Notifications`, `find_by_id`, `find_all`, `find_by_type(notificationType, meta)` → `Notifications[]`, `mark_as_read(identifier, meta)` → `Notifications`, `delete(identifier, meta)`.

### RecycleBinRepository

`store(payload, meta)` → `RecycleBin`, `find_all(meta)`, `find_by_id(identifier, meta)`, `find_by_item_type(itemType, meta)`, `purge(identifier, meta)`, `purge_all(meta)` — all void except `find_*`.

### UserPreferencesRepository / WorkspacePreferenceRepository

`create(payload)`, `get_by_identifier(identifier)` / `get(meta)`, `update(identifier, payload[, meta])`.

### Workspace transfer / duplicate

Record repositories (notes, bookmarks, snippets, todo, reminder, workspace preferences) expose:

| Method | Returns |
| --- | --- |
| `transfer_record(recordIdentifier, previousWorkspaceIdentifier, targetWorkspaceIdentifier)` | `void` |
| `duplicate_record(recordIdentifier, previousWorkspaceIdentifier, targetWorkspaceIdentifier)` | `void` |
| `record_exists_in_workspace(recordIdentifier, workspaceIdentifier)` | `boolean` |

> `DataEngine` is exported but has a **private constructor** — it is native-only. Use the repositories as your JS entrypoint.

---

## Payloads

All adapter DTOs are camelCase when crossing the JS boundary.

**RequestMeta** — required by most workspace-scoped methods:

```ts
const meta = { workspaceIdentifier: "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d" };
```

| DTO | Fields |
| --- | --- |
| `CreateWorkspace` | `name`, `description` |
| `UpdateWorkspace` | `name?`, `description?`, `isDefault?`, `isHidden?`, `isSecured?`, `password?` (empty string clears) |
| `CreateNote` | `title`, `content`, `categories?: string[]`, `workspaceIdentifier?` |
| `UpdateNote` | `title?`, `content?`, `categories?` |
| `CreateBookmark` | `title`, `url`, `tag` |
| `UpdateBookmark` | `title?`, `url?`, `tag?` |
| `CreateSnippet` | `title?`, `language?`, `code`, `description?`, `isPinned`, `createdAt`, `updatedAt` |
| `UpdateSnippet` | `title?`, `language?`, `code?`, `description?`, `isPinned?` |
| `CreateTodo` | `title`, `description?`, `dueDate?`, `priority` |
| `UpdateTodo` | `title?`, `description?` |
| `CreateReminder` | `title`, `description?`, `recurring`, `recurrenceRule?`, `alarmSound?`, `remindAt`, `workspaceIdentifier?` |
| `UpdateReminder` | `title?`, `description?`, `recurring?`, `recurrenceRule?`, `alarmSound?`, `remindAt?` |
| `CreateNotification` | `title`, `body`, `notificationType`, `workspaceIdentifier?`, `isRead` |
| `CreateRecycleBinEntry` | `itemId`, `itemType`, `payload` (JSON string), `workspaceIdentifier?` |
| `CreateUserPreferences` | `masterFirstName`, `masterLastName`, `masterEmail` |
| `UpdateUserPreferences` | `masterFirstName?`, `masterLastName?`, `masterEmail?` |
| `CreateUserPreference` | `firstName`, `lastName`, `email` |
| `UpdateUserPreference` | `firstName?`, `lastName?`, `email?` |

Timestamps (`createdAt`, `updatedAt`, `remindAt`, `dueDate`) are RFC3339 / date strings. Note `CreateSnippet` currently requires `createdAt`/`updatedAt`.

```ts
import { NotesRepository } from "lunar/pkg/lunar.js";

const notes = new NotesRepository();
const meta = { workspaceIdentifier: "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d" };

const note = await notes.create(
  { title: "Hello", content: "from lunar" },
  meta,
);

const all = await notes.find_all(meta);
```

---

## TypeScript usage

The generated types in `bindings/` (re-exported by `index.ts`) describe the models, enums and `JsonValue`:

```ts
import type { Notes, ItemType, Priority, Tag } from "lunar/index";
import { NotesRepository } from "lunar/pkg/lunar.js";

async function createNote(): Promise<Notes> {
  const repo = new NotesRepository();
  return repo.create(
    { title: "t", content: "c", categories: ["ideas"] },
    { workspaceIdentifier: "…" },
  );
}
```

Two packages can be consumed from this repo:

- **`pkg/`** (`wasm-pack` output) — the runtime JS classes only. Its `lunar.d.ts` declares the repository classes but **not** the `Model` types.
- **crate root** (`index.ts` + `bindings/`) — pure TypeScript types. Point a `file:`/workspace dependency at the `lunar/` root (or copy `bindings/` + `index.ts`) to get `Model`, `Create*`, `RequestMeta` and enum types.

`bindings/` contains: `bookmark.ts`, `notes.ts`, `notifications.ts`, `recycle_bin.ts`, `reminder.ts`, `snippets.ts`, `todo.ts`, `user_preferences.ts`, `workspace_preferences.ts`, `workspaces.ts`, `sea_orm_active_enums.ts`, `serde_json/JsonValue.ts`.

---

## Vue / Nuxt integration

WASM is browser-only, so:

- load it **client-side** only (Nuxt: a client plugin),
- keep the `.wasm` out of SSR.

### 1. Install the package

Place `pkg/` inside your app (or use a `file:`/workspace dependency):

```
<app>/
├── libs/lunar/          # copy of pkg/
└── app/
```

### 2. Nuxt client plugin

`app/plugins/lunar.client.ts`:

```ts
import init, {
  NotesRepository,
  WorkspaceRepository,
} from "~/libs/lunar/lunar.js";

let ready: Promise<void> | null = null;

export function useLunar() {
  return {
    notes: () => new NotesRepository(),
    workspaces: () => new WorkspaceRepository(),
  };
}

export default defineNuxtPlugin({
  name: "lunar",
  async setup() {
    if (import.meta.server) return;
    ready ??= init();
    await ready;
  },
});
```

If the `.wasm` is served from `public/` instead of bundled, use:

```ts
await init({ module_or_path: "/lunar_bg.wasm" });
```

### 3. Nuxt / Vite config

`nuxt.config.ts`:

```ts
export default defineNuxtConfig({
  ssr: false, // optional but typical for a lunar-backed app
  vite: {
    optimizeDeps: {
      exclude: ["lunar"], // keep the ESM module out of dep pre-bundling
    },
  },
});
```

For Vite (non-Nuxt), the same `optimizeDeps.exclude` plus a wasm-friendly setup applies.

### 4. Composable

`app/composables/useNotes.ts`:

```ts
import { NotesRepository } from "~/libs/lunar/lunar.js";

export function useNotes() {
  const repo = new NotesRepository();
  const meta = { workspaceIdentifier: "9b1deb4d-…" };

  return {
    list: () => repo.find_all(meta),
    byId: (id: string) => repo.find_by_id(id, meta),
    create: (payload) => repo.create(payload, meta),
    update: (id: string, payload) => repo.update(id, payload, meta),
    remove: (id: string) => repo.delete(id, meta),
  };
}
```

### 5. Pinia store

`app/stores/notes.ts`:

```ts
import { defineStore } from "pinia";
import { NotesRepository } from "~/libs/lunar/lunar.js";
import type { Notes } from "lunar"; // crate root types (index.ts re-exports bindings)

export const useNotesStore = defineStore("notes", {
  state: () => ({ notes: [] as Notes[], loading: false }),
  actions: {
    async load(workspaceIdentifier: string) {
      this.loading = true;
      const repo = new NotesRepository();
      try {
        this.notes = await repo.find_all({ workspaceIdentifier });
      } finally {
        this.loading = false;
      }
    },
  },
});
```

---

## PGlite (browser Postgres)

[PGlite](https://pglite.dev) is a full WASM build of PostgreSQL that runs in the browser (in-memory or IndexedDB), Node, Bun and Deno.

### Why the repositories can't talk to it (today)

SeaORM 2.0 ships only sqlx drivers (`sqlx-postgres`, `sqlx-mysql`, `sqlx-sqlite`) plus `rusqlite`. The `sqlx-postgres` driver requires TCP + the tokio runtime, which do not exist on `wasm32-unknown-unknown`. There is **no SeaORM → PGlite connector**, so the browser build of this crate backs repositories with a `MockDatabase` — the API is real, the persistence is not.

JS-side ORMs that **do** support PGlite: [Drizzle](https://orm.drizzle.team), [Kysely](https://kysely.dev), [MikroORM](https://mikro-orm.io/docs/usage-with-pglite) (or raw SQL via `@electric-sql/pglite`).

### Pattern A — JS data layer + lunar types (recommended for browser apps)

Do data access in JS against PGlite and reuse lunar's generated TypeScript types for shape parity:

```ts
// app/plugins/pglite.client.ts
import { PGlite } from "@electric-sql/pglite";

const db = new PGlite(); // or new PGlite("idb://almonds") to persist in IndexedDB
await db.exec(`CREATE TABLE IF NOT EXISTS workspaces (…)`);
export default defineNuxtPlugin(() => {
  return { provide: { db } };
});
```

```ts
// app/composables/useWorkspaces.ts
import type { Workspaces } from "lunar/index";

export function useWorkspaces() {
  const { $db } = useNuxtApp();
  const list = async (): Promise<Workspaces[]> =>
    (await $db.query("SELECT * FROM workspaces")).rows;
  return { list };
}
```

Because both lunar's bindings and your PGlite rows are camelCase, the app components stay decoupled from the backend.

### Pattern B — TS facade matching the repository API

If you want app code to be *backend-agnostic* (swap PGlite ↔ lunar-wasm ↔ native later), implement a thin facade that mirrors the repository signatures:

```ts
// app/services/notes.ts
import type { Notes, CreateNote, UpdateNote } from "lunar/index";
import type { PGlite } from "@electric-sql/pglite";

export class PgNotesRepository {
  constructor(private db: PGlite) {}

  async create(payload: CreateNote, meta: { workspaceIdentifier: string }): Promise<Notes> {
    const { rows } = await this.db.query(
      `INSERT INTO notes (title, content, workspace_identifier) VALUES ($1, $2, $3)
       RETURNING *`,
      [payload.title, payload.content, meta.workspaceIdentifier],
    );
    return rows[0];
  }

  async find_all(meta: { workspaceIdentifier: string }): Promise<Notes[]> {
    const { rows } = await this.db.query(
      `SELECT * FROM notes WHERE workspace_identifier = $1`,
      [meta.workspaceIdentifier],
    );
    return rows;
  }
}
```

The Nuxt plugin then decides which implementation to provide:

```ts
export default defineNuxtPlugin((nuxtApp) => {
  const repo =
    import.meta.env.VITE_DATA_LAYER === "pglite"
      ? new PgNotesRepository(nuxtApp.$db)
      : new NotesRepository();
  return { provide: { notes: repo } };
});
```

### Pattern C — native path for real persistence

Run the repositories **natively** (Tauri command handler or a small Rust server) against a real Postgres, and keep PGlite for offline/fallback UI. See [Native (non-wasm) usage](#native-non-wasm-usage).

### PGlite caveats

- Single connection model (no connection pool).
- Storage: in-memory, `idb://…` (IndexedDB) in the browser, filesystem in Node/Bun/Deno.
- Alpha status — fine for prototypes, offline-first and dev; vet it for production workloads.

---

## Native (non-wasm) usage

In Rust (Tauri commands, a server, or CLI) the repositories run against a real Postgres:

```rust
use std::sync::Arc;
use lunar::data_engine::DataEngine;
use lunar::repositories::{prelude::*, workspace::WorkspaceRepository};

#[tokio::main]
async fn main() -> Result<(), lunar::error::LunarError> {
    let engine = DataEngine::new("postgres://user:pass@localhost:6543/orchard").await?;

    engine.run_migrations().await?;

    let conn = engine.connection().clone();
    let workspaces = WorkspaceRepository::new(Arc::new(conn));

    let workspace = workspaces
        .create_workspace(lunar::adapters::workspace::CreateWorkspace {
            name: "Work".into(),
            description: String::new(),
        })
        .await?;

    Ok(())
}
```

Notes:

- `run_migrations` is `cfg(not(wasm32))`. `DataEngine::new` compiles on wasm but needs a reachable Postgres (no network on wasm32), so it is practically native-only.
- The `migration` crate is a native-only dependency.
- The native constructor lives on the `*RepositoryExt` traits (`WorkspaceRepositoryExt::new(Arc<DatabaseConnection>)`); the prelude re-exports the traits, and the structs are under `lunar::repositories::<name>`. The wasm constructors (`new_wasm()`) and JS wrappers are irrelevant here.

---

## Regenerating types & entities

- `bindings/*.ts` are produced by `ts-rs` (entities carry `#[ts(export, export_to = "…")]`). Regenerate by building/exporting the crate; check `ts-rs` docs for the export trigger.
- `index.ts` is generated by `scripts/fix-ts-exports.py` — do not edit by hand.
- `server/` entities are generated separately with `just generate-server-entities`.

---

## Troubleshooting

**`DataEngine` has a private constructor / no methods in JS.** Expected — it is native-only. Use the repositories.

**`Failed to fetch` / wrong MIME type for `lunar_bg.wasm`.** Serve the `.wasm` with `application/wasm`, or pass an explicit path via `init({ module_or_path })`.

**`Error: Cannot find module` when the wasm isn't copied.** The ESM entry resolves `lunar_bg.wasm` relative to `lunar.js` (`new URL('lunar_bg.wasm', import.meta.url)`), so keep both files together.

**Method names look like `create_js` in Rust.** The Rust wrappers are suffixed `_js` to avoid shadowing the async-trait methods; `#[wasm_bindgen(js_name = "…")]` exposes them under clean names (e.g. `create`, `find_all`).

**`ReferenceError: …` on SSR.** WASM is client-only — load it inside a Nuxt `*.client.ts` plugin or guard with `import.meta.client`.
