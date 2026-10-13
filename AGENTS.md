# AGENTS.md — Version Control with Jujutsu (`jj`) and DDD Guidelines

## Jujutsu (`jj`) Mental Model (read once)

- No staging area. The working copy **is** a commit, always `@`. Edits are auto-snapshotted on every `jj` command.
- Commits are mutable. Rewriting is normal and safe; descendants auto-rebase.
- Every commit has a stable **change ID** (use this to refer to commits) and a changing commit ID.
- **Bookmarks** ≈ Git branches, but they do **not** move automatically when you make new commits. You move them explicitly before pushing.
- Conflicts are stored in commits and don't block work. Resolve them when convenient.
- Every operation is logged. `jj undo` / `jj op restore` can recover from almost any mistake.

## Hard rules (JJ)

1. **Never open an interactive editor or TUI.** Always pass messages/paths inline:
   `jj describe -m "..."`, `jj commit -m "..."`, `jj squash -m "..."` or `jj squash -u`, `jj split -m "..." <paths>`.
   Do **not** use `jj split` without paths, `jj diffedit`, `jj rebase -i`, or `jj resolve` (merge tool) without a non-interactive plan.
2. **One logical change per commit.** Each commit must build and be reviewable on its own.
3. **Describe before you code.** Never leave a commit with an empty description at push time.
4. **Never push to the trunk bookmark (`main`) directly** unless explicitly told. Push feature bookmarks.
5. **Never rewrite immutable/pushed-and-shared commits.** Don't use `--ignore-immutable` without explicit permission.
6. **Run `jj st` before and after any rewriting operation** and read the output.
7. **No destructive shortcuts:** don't `jj abandon` commits you didn't create without asking.

## Standard routine (JJ)

### 0. Orient (start of every session)

```sh
jj st                 # working copy state, conflicts, parent
jj log -n 10          # recent history
jj git fetch          # sync with remote
```

### 1. Start a change (describe-first)

```sh
jj new 'trunk()'      # fresh commit on top of latest trunk (use `jj new` to stack on current work)
jj describe -m "feat(scope): short imperative summary"
```

If `@` is already an empty, undescribed commit, skip `jj new` and just `describe`.

### 2. Work

- Edit files normally; `jj` snapshots automatically.
- Inspect with `jj st`, `jj diff`, `jj diff --stat`.
- Run tests/linters **before** finalizing the commit.
- Refine the message any time: `jj describe -m "..."`.

### 3. Finish the change, start the next

```sh
jj new                # `@` becomes the finished commit's child; the finished commit stays as @-
```

Keep stacking small commits (`jj new` → `describe` → work). Don't batch unrelated work into one commit.

### 4. Reshape history (as needed)

| Goal | Command |
| --- | --- |
| Fold current edits into parent | `jj squash` (add `-u` to keep the parent's message) |
| Auto-distribute edits into the right ancestor commits | `jj absorb` |
| Split by file (non-interactive) | `jj split -m "msg for first part" path/a path/b` |
| Move a commit elsewhere | `jj rebase -r <change-id> -d <dest>` |
| Move a commit and its descendants | `jj rebase -s <change-id> -d <dest>` |
| Drop an unwanted commit you made | `jj abandon <change-id>` |
| Edit an older commit in place | `jj edit <change-id>` → change files → `jj new` (or `jj edit @+`) to return |

Prefer `jj absorb` or `jj squash` for review fixes over adding "fix typo" commits.

### 5. Name and publish

```sh
jj bookmark create feat/short-name -r @-    # first time: point at the last finished commit
# later updates to an existing bookmark:
jj bookmark advance --to @-                 # moves the nearest bookmark forward
#   (or: jj bookmark set feat/short-name -r @-)
jj git push -b feat/short-name              # new bookmarks are tracked automatically
```

- Bookmark naming: `feat/…`, `fix/…`, `chore/…`, `docs/…`, kebab-case.
- Push only after: tests pass, `jj log` shows the intended stack, no conflicts (`jj st` clean), no empty/undescribed commits.
- After pushing, keep working on a fresh `@` (`jj new`), not on the pushed commit.
- Quick throwaway push (no naming): `jj git push --change @-` creates an auto-named bookmark.

### 6. Stay in sync with trunk

```sh
jj git fetch
jj rebase -s <stack-root-change-id> -d 'trunk()'   # rebase the whole stack onto new trunk
jj st                                               # check for conflicts
jj git push -b feat/short-name                      # force-with-lease semantics are built in
```

### 7. Conflicts

- `jj st` / `jj log` mark conflicted commits. Conflicts do not block you.
- Resolve in the conflicted commit: `jj edit <change-id>`, fix the conflict markers in files, then confirm `jj st` shows no conflicts.
- Resolve the **oldest** conflicted commit first; descendants often clear automatically.
- Never push while any commit in the stack is conflicted.

### 8. Recover from mistakes

```sh
jj op log             # list every operation
jj undo               # undo the last operation
jj op restore <op-id> # return the whole repo to a prior state
jj evolog             # history of how the current commit evolved
```

When unsure, run `jj op log` first. Don't "fix" a bad state by hand-editing `.jj/` or `.git/`.

## Commit message format (Conventional Commits)

```
type(scope): imperative summary, ≤ 72 chars

Optional body: what and why, wrapped at ~72 cols.
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `build`, `ci`.

## Daily cheat sheet (JJ)

```sh
jj st                      # status
jj log -n 10               # recent history
jj diff                    # current changes
jj new 'trunk()'           # start work from latest trunk
jj describe -m "..."       # set message
jj new                     # finish this change, start the next
jj squash / jj absorb      # fold fixes into earlier commits
jj bookmark advance --to @-  # move bookmark up to the finished commit
jj git push -b <bookmark>  # publish
jj git fetch               # sync
jj undo                    # oops
```

## DDD.md — Domain-Driven Design for a Rust Discord Bot

### 1. Layers and the dependency rule

Dependencies point **inward only**:

```
bot (binary, composition root)
 ├─► discord (interface)  ──► application ──► domain
 └─► infrastructure       ──► application ──► domain
```

| Layer | Crate | Contains | May depend on | Must NOT depend on |
| --- | --- | --- | --- | --- |
| Domain | `domain` | Entities, value objects, aggregates, domain events, domain services, repository **traits**, domain errors | `std`, `thiserror`, `serde` (only if needed), `uuid`/`time` types | `serenity`, `poise`, `sqlx`, `tokio`, `reqwest`, any I/O |
| Application | `application` | Use cases (commands/queries), DTOs, ports for external services (`Clock`, `EventPublisher`, `Notifier`…), orchestration | `domain`, `async-trait`, `tracing` | `serenity`, `poise`, `sqlx` |
| Infrastructure | `infrastructure` | Adapters: `sqlx` repositories, HTTP clients, clock/RNG impls, config loading | `domain`, `application`, `sqlx`, `reqwest` | `discord` crate |
| Interface | `discord` | `poise` commands, event handlers, embeds/presenters, Discord ↔ domain ID mapping | `application`, `domain` (types only), `poise` | `infrastructure`, `sqlx` |
| Composition root | `bot` | `main.rs`: config, wiring, startup, graceful shutdown | all crates | — |

Workspace layout:

```
Cargo.toml                (workspace)
crates/
  domain/src/<context>/{mod.rs, entity.rs, value_objects.rs, events.rs, repository.rs, error.rs}
  application/src/<context>/{mod.rs, commands.rs, queries.rs, service.rs, ports.rs}
  infrastructure/src/{persistence/<context>/, clock.rs, config.rs}
  discord/src/<context>/{commands.rs, handlers.rs, presenter.rs}
  bot/src/main.rs
migrations/
```

One crate per layer is the default **because the compiler then enforces the dependency rule**. Do not collapse layers into one crate without asking.

Verify before finishing any task:

```sh
cargo tree -p domain -e normal | grep -Ei "serenity|poise|sqlx|tokio|reqwest"   # must print nothing
cargo tree -p application -e normal | grep -Ei "serenity|poise|sqlx"            # must print nothing
```

### 2. Bounded contexts

- Split by business capability, not by technical type (e.g. `moderation`, `economy`, `tickets`, `scheduling`).
- Each context is a module in every layer, with the same name across layers.
- Contexts never share aggregates or reach into each other's repositories. They communicate via **domain events** or a small, explicit application-level interface.
- Shared primitives (`GuildId`, `UserId`, `ChannelId`, `Timestamp`) live in `domain::shared`.

### 3. Ubiquitous language

- Name types after the business concept, not the Discord concept. `Warning`, `Ticket`, `Reminder`, not `DiscordMessageRow`.
- Keep a glossary in `docs/glossary.md`. Add a term **before** using it in code; reuse it in commands, tests, docs and user-facing text.
- No `Manager`, `Handler`, `Helper`, `Util` in the domain. If a name needs one, the concept isn't understood yet.

### 4. Domain layer rules

- **Domain is synchronous and pure.** No `async`, no I/O, no `Utc::now()`, no random. Time and randomness arrive as parameters.
- **Discord IDs are domain newtypes**, not `serenity` types:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct GuildId(u64);
  ```
  Conversion to/from `serenity` IDs happens only in the `discord` crate.
- **Value objects**: immutable, validated in the constructor, compared by value. Use `TryFrom`/`new() -> Result<_, DomainError>`; invalid states must be unrepresentable. Use newtypes instead of raw `String`/`u64` for anything with rules (`ReasonText`, `Amount`, `Duration`).
- **Entities** have identity (`Id` newtype), **aggregates** have one root and enforce all invariants of what they own. Fields are private; mutate only through methods that return `Result`.
- **One aggregate per transaction.** Reference other aggregates by ID only.
- **Domain events**: past-tense structs (`WarningIssued`, `TicketClosed`). Aggregate methods **return** events (`Vec<DomainEvent>`); they don't publish them.
- **Domain services** only for logic that spans aggregates and doesn't belong to one. Stateless, pure.
- **Repository traits live in `domain`**, one per aggregate root, named by the aggregate (`WarningRepository`). Methods use domain language (`find_active_for_member`), not SQL language.
- **Errors**: one `thiserror` enum per context (`ModerationError`). Variants describe business rule violations, not technical failures.
- `#![forbid(unsafe_code)]` in every crate. No `unwrap()`/`expect()` outside tests and `main`.

### 5. Application layer rules

- A **use case = one struct method**, one business intent: `IssueWarning::execute(cmd) -> Result<WarningDto, AppError>`. Group related use cases in a `<Context>Service` only if they share dependencies.
- Inputs are plain **command/query structs** (`IssueWarningCommand { guild, target, moderator, reason }`) using domain types or primitives. **Never accept `serenity` or `poise` types.**
- Outputs are **DTOs** (plain structs), never aggregates, never Discord types.
- Flow of every use case: **load aggregate → call domain method → save → publish events → return DTO.** No business rules here; if you're writing an `if` about the business, move it into the domain.
- **Ports** (traits) for everything non-domain the use case needs: `Clock`, `EventPublisher`, `Notifier` (e.g. "send DM"/"post to channel" as an abstraction), `IdGenerator`. Define them in `application::<context>::ports`.
- Dependency style (pick one, keep it): `Arc<dyn Trait>` with `#[async_trait]`, injected via constructor. Don't mix with generics.
- Authorization is an application concern expressed with a domain concept (`Actor { id, role }`), checked in the use case. Discord permission checks in `poise` (`required_permissions`) are only a UX pre-filter, never the sole enforcement.
- `AppError` wraps domain errors plus technical categories (`NotFound`, `Conflict`, `Unavailable`, `Unauthorized`). Use `thiserror`.
- Use `tracing` with structured fields (`guild_id`, `use_case`). Never log tokens or message content unless required.

### 6. Infrastructure layer rules

- Implements domain repository traits and application ports. Nothing else calls `sqlx`.
- **Separate persistence models** (`WarningRow`) from domain types. Map explicitly: `TryFrom<WarningRow> for Warning` and `From<&Warning> for WarningRow`. Never derive `sqlx::FromRow` on a domain type.
- Migrations in `/migrations`, forward-only, one concern each. Use `sqlx::migrate!`.
- Transactions: keep them inside the repository method for single-aggregate saves. If a use case truly needs multiple writes atomically, introduce a `UnitOfWork` port, and ask first.
- Map `sqlx::Error` to `AppError`/repository error variants at this boundary. Don't leak `sqlx` types upward.
- Config (`DISCORD_TOKEN`, `DATABASE_URL`, etc.) loaded from environment in `infrastructure::config`, validated at startup. Secrets never in code, logs, or `Debug` output (wrap in a `Secret` type).

### 7. Discord (interface) layer rules

- Use `poise`; import serenity through `poise::serenity_prelude` to avoid version mismatches.
- `poise::Data` holds **only application services** (`Arc<ModerationService>`, …), never a DB pool or HTTP client.
- **Every command has the same shape:**
  1. Extract and validate input (map `serenity` IDs → domain IDs).
  2. Build a command/query struct.
  3. Call exactly **one** use case.
  4. Map the DTO/`AppError` to a response via the context's `presenter`.
  No business logic, no persistence, no `if` on business rules in command functions.
- **Presenters** turn DTOs into embeds/messages, and `AppError` into user-friendly text. All user-facing strings live in one place per context so wording stays consistent.
- **Event handlers** (`FullEvent` in poise) do the same: translate a Discord event into an application command, call a use case, return. Handlers must be idempotent: Discord may redeliver events.
- Respect Discord limits: respond to interactions within 3 seconds (`ctx.defer()` for anything slower); mind embed and message length limits in the presenter, not the domain.
- One file of commands per context; command names and descriptions use the ubiquitous language. Prefer slash commands; add prefix commands only if requested.
- The `error` handler in the poise `FrameworkOptions` is the single place where unexpected errors are logged and shown generically to the user.

### 8. Composition root (`bot` crate)

- `main.rs` is the **only** place that constructs concrete infrastructure types and wires them into services.
- Order: load config → init `tracing` → connect DB → run migrations → build adapters → build use cases/services → build poise `Framework` → start client → handle shutdown signal.
- No logic beyond wiring. If `main.rs` grows beyond ~150 lines, extract `bootstrap` modules.

### 9. Testing rules

| Layer | Kind | How |
| --- | --- | --- |
| Domain | Unit tests, no mocks | Plain `#[cfg(test)]`; cover every invariant and every error variant |
| Application | Unit tests with **in-memory fakes** of repositories/ports (in `application/tests/support`) | Assert on returned DTOs, saved state, and published events |
| Infrastructure | Integration tests against a real DB (`#[sqlx::test]`) | Round-trip: save → load equals original |
| Discord | Minimal: unit-test ID mapping and presenters | No live Discord in CI |

- Test names state behavior: `issuing_warning_to_bot_is_rejected`.
- A bug fix starts with a failing test in the lowest layer where the bug lives.
- Required before finishing: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

### 10. "Where does this go?"

| Question | Layer |
| --- | --- |
| Is it a business rule or invariant? | `domain` |
| Does it coordinate loading/saving/notifying for one user intent? | `application` |
| Does it touch SQL, HTTP, files, env, the clock? | `infrastructure` |
| Does it touch `poise`/`serenity` types, embeds, slash-command options? | `discord` |
| Does it only connect things together at startup? | `bot` |

### 11. Routine for adding a feature (follow in order)

1. **Language**: add/confirm terms in `docs/glossary.md`; identify the bounded context.
2. **Domain**: model value objects/aggregate methods/events; write tests first; add or extend the repository trait.
3. **Application**: add the command/query struct, DTO, use case, and any new port; test with in-memory fakes.
4. **Infrastructure**: add migration, row model + mapping, repository impl; add integration test.
5. **Discord**: add the command/handler and presenter; wire into the poise command list.
6. **Composition**: wire new dependencies in `bot`.
7. **Verify**: run the `cargo tree` dependency checks, then fmt/clippy/test.
8. Commit in small steps, one per layer when practical (see `AGENTS.md` for the jj workflow).

### 12. Do / Don't summary

**Do**: keep the domain pure; validate at construction; return events; map at every boundary; keep commands thin; inject dependencies via constructors; ask before changing layer boundaries.

**Don't**: import `serenity`/`sqlx` into `domain` or `application`; put `Utc::now()` in the domain; derive `FromRow` on aggregates; let a command call a repository; return aggregates from use cases; share aggregates across contexts; use `unwrap()` in non-test code; add a "utils" crate. **add to AGENTS.md** only