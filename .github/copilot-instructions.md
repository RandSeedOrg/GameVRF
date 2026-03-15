# GitHub Copilot Custom Instructions for this repository

Purpose
- This is a global instruction baseline intended to apply across related repos (WL-Private, WL-Public, GameVRF, AssetsDAO). If a repo does not contain a certain layer (e.g., UI), skip that section.

- When generating or reviewing code, Copilot MUST:
  - identify which sections of this document apply (backend / frontend / cross-chain / UI / upgrades),
  - run the relevant checklists and explicitly report results as:
    - PASS / FAIL / NEEDS-INFO,
  - propose the smallest change that satisfies all MUST requirements.
- When a checklist contains blanks (e.g., `stale time: ____ ms`), Copilot MUST:
  - propose a safe default consistent with this document, and
  - clearly label it as a default that the team can override.

---

## CRITICAL: Bilingual PR Review Output Requirement (HIGHEST PRIORITY)

**This requirement applies to ALL Copilot PR review text (overview and comments).**

**MANDATORY FORMAT (MUST):**
- Every review item MUST use this exact two-line format:
  ```
  中文：<Simplified Chinese explanation>
  EN: <English explanation>
  ```

**SELF-CHECK + RETRY (MUST):**
- Before returning your review output, scan every item you wrote.
- If any item is missing either the `中文：` line or the `EN:` line, STOP and regenerate that item in the correct format.
- Repeat this check until all items contain both lines.

**FALLBACK (MUST):**
- If you cannot produce bilingual output after retry attempts, return this exact text instead of partial/broken output:

**EXAMPLES:**

Good (correct format):
```
中文：建议为列表端点添加分页，以避免大数据集导致的性能问题。
EN: Recommend adding pagination to the list endpoint to avoid performance issues with large datasets.
```


---

## 1) Application Overview
This is an iGaming and Lottery system built on the Internet Computer (IC) blockchain, providing decentralized gaming services.

It includes:
- User-facing games (e.g., Lucky Nickel, Quick Quid, Daily 4)
- Admin management console (WLMC)
- Community website
- Public repos (GameVRF and AssetsDAO)

Cross-chain note:
- The system uses the Internet Computer (IC) as a bridge to **Solana** and **EVM chains**. Cross-chain transaction security is a top priority (see Section 12.6).

---

## 2) Technology stack (authoritative “what we use”)

### 2.1 Backend (ICP canisters)
Languages:
- Rust (primary)
- Motoko (if/where used)

Rust canisters baseline (keep versions aligned unless explicitly upgraded):
- ic-cdk 0.17.1
- candid 0.10.11
- ic-stable-structures 0.6.9
Motoko: via `mops`

Key crates/patterns:
- ic-cdk / candid / stable structures / logging utilities already used in this codebase.
- Prefer existing repository patterns over introducing new frameworks.

Canister interaction patterns:
  - Use `query` for read-only deterministic calls.
  - Never mutate state in `query` methods.
- Inter-canister calls:
  - Avoid holding mutable state across `await`.
  - Handle reject codes explicitly; fail closed on unexpected errors.
  - Prefer simple, explicit request/response types to reduce coupling.

### 2.2 Frontend (apps)
Frameworks/tools:
- TypeScript/React (Node.js >= 20.19.0)
- Vite for `admin-app`
- Astro for `WLCommunity` (and any community site packages)


Sizing units by app (MUST):
- `ProjectL_frontend`:
  - Uses the convention **1rem = 1px** to support zooming/scaling across different devices.
  - Convert from design px to the same numeric `rem` value (e.g., 12px -> `12rem`).
  - Ensure the app's root font-size configuration enforces the 1:1 mapping (do not assume browser default).
- `admin-app` and `WLCommunity`:
  - Use standard responsive web conventions with **px** (and/or the existing Tailwind spacing scale where applicable).
  - Do not introduce the `1rem = 1px` assumption in these apps.

Styling system:
- Use the styling approach already adopted in the specific app (Tailwind/SCSS).
- Do not introduce a new styling system without clear justification and alignment with the existing codebase.

Canister agent/integration:
- Prefer a single, consistent approach for:
  - IDL generation
  - type binding strategy
  - canister method wrappers (hooks/services)
- Avoid manually duplicating Candid types in multiple places.

### 2.3 Deployment & environments
Environments:
- development
- testing
- production


### 2.4 Tools & quality gates
Formatting/linting:
- Rust: rustfmt, clippy
- Frontend: eslint, prettier

Testing tools:
- Rust: cargo test
- Frontend: repo/app-specific test runner
- E2E: only if already configured; avoid introducing flaky tests

Security tooling:
- Secret scanning, dependency scanning, SAST (when enabled in GitHub/CI)

### 2.5 Blockchain / ICP specifics (non-negotiable constraints)
Determinism, time, randomness:
- Use `ic_cdk::api::time()` (Rust) / `Time.now()` (Motoko); do not assume wall-clock guarantees for security.
- For randomness, use `raw_rand` (update) and derive via DRBG for crypto needs.
- Prefer integer math over floating point for critical logic (especially money/odds/payouts).

Cycles and limits:
- Be explicit about cycles; estimate costs when adding inter-canister calls or HTTP outcalls.
- Respect IC message size limits (rule of thumb: ~2 MiB per message, depending on context).

Pagination and payload requirements (MUST):
  - Any endpoint returning a list/array MUST implement pagination if the list can exceed:
    - 100 items, OR
    - ~200 KB estimated serialized payload, OR
    - could grow unbounded over time (e.g., bets, transactions, logs, histories, leaderboards).
  - Default pagination parameters (MUST): 20 items
  - Paginated response shape (MUST):
    - `items: Vec<T>` (or equivalent)
    - `next_cursor: Option<String>` (or stable encoded cursor)
    - `has_more: bool` (optional but recommended)
    - `total: Option<u64>` only if cheap to compute (avoid expensive full scans/counts)
  - Pagination approach (MUST/SHOULD):
    - MUST prefer cursor-based pagination for mutable datasets to avoid duplicates/missing rows.
    - Offset-based pagination is allowed only for static datasets or when stable ordering is guaranteed.
  - Stable ordering (MUST):
    - Every paginated endpoint MUST document and enforce a stable sort order (e.g., `(created_at desc, id desc)`).
    - Cursor MUST be derived from ordering key(s).
- Large binary transfer rules (MUST):
  - Do not return large binary blobs in a single response.
  - Binary transfers MUST be chunked (e.g., `get_chunk(asset_id, chunk_index)`) or served via an asset canister / dedicated asset pipeline.

Upgrades and stable memory:
- Persist state across upgrades using stable memory.
  - Rust: `#[pre_upgrade]` / `#[post_upgrade]`; consider `ic-stable-structures` for large/stable data.
  - Motoko: stable variables; implement `preupgrade` / `postupgrade`.
- Version state and test upgrade/downgrade paths locally when state layout changes.

Candid compatibility:
- Maintain backward compatibility for Candid (`.did`).
- Avoid breaking changes; if unavoidable, provide migration notes and upgrade steps.

---

## 3) Project structure & ownership (monorepo map)
### 3.1 Workspace layout overview
- Cargo workspace (backend)
- npm workspace(s) (frontend)

### 3.2 Backend structure
- `backend/` contains canisters.
- `backend/libraries/` contains shared libraries.

Guidance:
- Put shared logic (validation, normalization, DTO mapping, auth helpers) into shared libraries when used in multiple places.
- Avoid duplicating business rules across canisters.

### 3.3 Frontend structure
- `src/` apps include: `ProjectL_frontend`, `admin-app`, `WLCommunity` (scope/ownership depends on repo).
- Shared UI/components/hooks:
  - if similar logic exists across apps, prefer extracting shared modules instead of copy–paste.

### 3.4 Cross-cutting conventions
- Match existing naming and folder conventions.
- Respect import boundaries (no backdoor dependencies that blur layers).
- API boundaries must stay clear: frontend ↔ canister ↔ shared libs.

---

## 4) Dependencies policy (adding/upgrading)
### 4.1 General principles
- Minimize dependencies; justify new ones.
- Prefer existing utilities and patterns in the repo.
- Avoid introducing overlapping dependencies that solve the same problem.

### 4.2 Rust dependency rules
- Allowed categories typically include: serialization, stable storage, testing, utility.
- Prefer minimal feature flags.
- Keep versions aligned with workspace constraints; document reasons for upgrades.

### 4.3 Node/npm dependency rules
- Respect lockfile policy and workspace placement.
- Avoid heavy dependencies that increase bundle size without clear value.
- Consider security and licensing impact for any new frontend dependency.

### 4.4 Upgrade process
- Use changelogs and migration notes.
- For upgrades that affect state/Candid/types/build tooling, include:
  - risk summary
  - required code changes
  - test plan
  - rollout/rollback notes when needed

---

## 5) Environment setup & build
### 5.1 Prerequisites
- Rust toolchain (follow repo toolchain config)
- Node.js >= 20.19.0
- dfx: 0.27.0 and 0.22.0 must be installed (project requirement)

### 5.2 Bootstrapping steps
- Install, build, run backend.
- Install, build, run each frontend app.

### 5.3 Environment variables & config files
- Environment-specific configuration files are tracked:
  - `.env.dev`, `.env.prod`, `.env.test`
  - They may contain canister IDs (public identifiers, not secrets).
- Use `.env.local` (gitignored) for local secrets/overrides.

### 5.4 Build commands (canonical)
- Backend: cargo workspace commands
- Frontend: npm workspace commands

### 5.5 Common pitfalls / known issues
- dfx network state issues
- canister ID mismatch across environments
- generated types drift (Candid ↔ TS/Rust mismatch)
- payload growth causing message-size problems (see ~2 MiB rule of thumb)

---

## 6) Coding standards & patterns (how we write code here)

### 6.1 General
- Prefer clarity, minimal diffs, and incremental improvements.
- Use early returns to reduce nesting.
- Avoid “clever” abstractions unless they simplify reuse.
- File size policy:
  - Prefer keeping source files under 300 LOC when practical.
  - For existing large files (>300 LOC), avoid making them larger.
  - When refactoring, split into focused modules.

### 6.2 Backend patterns (ICP-safe)
- Follow “check → effect → interact”.
- Do not hold mutable state across `await`.
- Access control must be centralized.
- Error handling:
  - prefer typed errors where possible
  - handle inter-canister call rejects explicitly
  - avoid leaking sensitive details in error messages/logs

### 6.3 Frontend patterns

- Use consistent data fetching patterns (hooks/services) within each app.
- Avoid duplicating wrappers for the same backend method across pages.
- Error/loading states should be consistent and user-friendly.
- Type safety:
  - ensure alignment with Candid (Option/variant/time/number units)
  - never assume optionals are always present
  - handle variants exhaustively


### 6.4 Logging & observability
- Redact sensitive data; avoid logging secrets or private URLs.
- Avoid leaking principals or sensitive metadata in logs.
- Prefer repo logging utilities; fallback to `ic_cdk::println`.

### 6.5 Performance & limits
- Large lists must use backend pagination + frontend incremental loading when growth is possible.
- Response size budgets for list endpoints used on initial page load
- UI responsiveness targets (MUST):
  - meaningful “above the fold” content SHOULD render within < 1.5s on a normal dev machine (local) and < 3s under typical user conditions
  - list views MUST render page 1 first and load subsequent pages incrementally (infinite scroll or “Load more”)
- Avoid N+1 patterns:
  - Do not fetch “details per item” after fetching a list unless unavoidable.
  - Prefer summary DTOs for list endpoints and a separate details endpoint when needed.

Efficiency checklist (MUST):
- [ ] No O(N²) (or worse) algorithm is introduced in a path that can grow; bounds and guards exist if unavoidable.
- [ ] No per-item loops that perform inter-canister calls (must batch or mirror locally).
- [ ] Cross-canister calls are minimized, batched, and have explicit error handling.

### 6.6 Frontend UX, Loading States, and Data Strategy (MUST)

#### 6.6.1 Loading states (MUST)
Loading approach must be explicit for every user-facing query.

Checklist (MUST):
- [ ] Loading UI type is chosen and consistent:
  - [ ] Skeleton loading (preferred for lists/cards/pages)
  - [ ] Global loading overlay (only for full-page blocking transitions)
- [ ] Button loading states exist for actions:
  - disable the button while in-flight
  - show spinner or loading label
  - prevent double-click / repeated submissions
- [ ] Silent loading / prefetch is considered:
  - [ ] Can data be prefetched without user-visible loading?
  - [ ] Prefetch must not spam network or cause unnecessary update calls.

#### 6.6.2 Real-time needs and request strategy (MUST)
Every data fetch must declare its update requirement:

Choose exactly one strategy per resource (MUST):
- [ ] One-time fetch (no refresh after initial load)
- [ ] Manual refresh (user pulls to refresh / clicks refresh)
  - Must show “refreshing” state and preserve previous content.
- [ ] Auto polling (only when truly needed)
  - Polling interval (REQUIRED):
    - non-critical data: > 10 seconds
    - critical data: 2–5 seconds
  - Stop/suspend rules (MUST):
    - stop polling when leaving the page / component unmount
    - pause polling when the browser tab is hidden (`document.hidden === true`)
    - resume when visible again
  - Cost rule (MUST):
    - polling must use query calls only
    - do not poll update calls

#### 6.6.3 Caching and invalidation strategy (MUST)
Every query must define cache keys and invalidation rules.

Query key design (MUST):
- [ ] List the query key structure (must support precise invalidation), e.g.:
  - `["game", productId, "summary"]`
  - `["orders", userId, filter, cursor]`
- [ ] Keys must include all inputs that change the response:
  - user identity
  - environment/network
  - filters/sorts
  - cursor/page
  - locale if it affects response

Cache lifecycle (MUST):
- [ ] Stale time: 300,000ms (during this window, repeated requests should hit cache, not the network)
- [ ] Cache time (GC): 60 minutes (how long to keep data after unmount)

Read-your-writes consistency after mutations (MUST):
- After any update/mutation that changes server state, choose one:
  - [ ] Invalidate: mark related query keys stale and refetch
  - [ ] SetData: update local cache directly (only if rules are clear and correctness is guaranteed)
- Do not leave the UI in a stale state after a successful mutation.

#### 6.6.4 Concurrency and race-condition handling (MUST)
Checklist (MUST):
- [ ] Request deduplication / cancellation:
  - when users rapidly switch tabs/routes, cancel previous in-flight requests for the same resource key
  - prevent out-of-order responses from overwriting newer data
- [ ] Dependent requests:
  - if request B depends on request A, enforce sequencing (e.g., `enabled` gating)
  - do not start B until A is ready

#### 6.6.5 State management architecture (MUST)
Use a “State Matrix” to decide where state lives. Avoid pushing everything into a single global store.

State Matrix (MUST to document for new features that introduce state):
| State example | Storage location | Reason / lifecycle |
|---|---|---|
| `filter_status` | URL query params | survives refresh; shareable links |
| `user_profile` | server cache (e.g., React Query) | server-derived data with caching rules |
| `is_sidebar_open` | UI store (e.g., Zustand/Context) | UI-only global toggle across components |

Persistence (MUST):
- [ ] Does any state need to persist to `localStorage`?
  - [ ] No
  - [ ] Yes: list keys and data schema, e.g. “dark mode”, “last wallet type”
- Persisted keys MUST be namespaced and versioned (see Section 13.2 Browser storage policy).

#### 6.6.6 Component design and reuse (MUST)
Checklist (MUST):
- [ ] Is there reusable UI in this change (e.g., filter bar, card layout, table columns)?
- [ ] Confirm whether a similar component already exists under common/shared components.
- [ ] If not reusing existing components, document why (API mismatch, design mismatch, performance, etc.).

#### 6.6.7 Asset loading strategy (IC / Cloudflare R2) (MUST)
Checklist (MUST):
- [ ] Where are images/videos stored?
  - [ ] IC Canister
  - [ ] Cloudflare R2
- [ ] Are thumbnails used for large images/video posters?
- [ ] Fallback handling:
  - [ ] placeholder/fallback image exists for load failure
  - [ ] avoid layout shift when assets load

---

## 7) Testing strategy & expectations

### 7.1 What must be tested
- Critical paths, bug fixes, and boundary cases.
- Keep tests deterministic (no real network/time unless mocked).

Before committing, run:
- Rust backend: `cargo test` (or `cargo test -p <package_name>`)
- Frontend: `npm test` (workspace-specific or root-level)

### 7.2 Backend testing
- Unit tests + canister-level tests (pocket-ic / ic-repl where used)
- Upgrade tests for state layout changes

### 7.3 Frontend testing
- Unit/component tests with stable mocks
- Type alignment tests when Candid/types change

### 7.4 Integration/E2E (if applicable)
- Test upgrade hooks and inter-canister flows locally (`dfx start`) when relevant.
- Prefer stable, non-flaky test patterns.

---

## 8) Code review requirements (what reviewers must check)

### 8.1 Functional correctness
- Requirements met; edge cases handled.

### 8.2 Security & privacy
- No secrets, tokens, keys, or private URLs.
- Access control is correct; data exposure is minimized.
- Logs/errors are redacted and do not leak sensitive metadata.

### 8.3 ICP-specific checks
Query/update correctness:
- Read-only endpoints are `query`.
- No state mutation in `query` methods.

Frontend ↔ backend usage correctness:
- Verify frontend calls use appropriate methods (e.g., `get_*_summary` vs `get_*_details`) to reduce exposure and payload.
- For similarly named methods, confirm each call site matches intent (query vs update).
- Prefer frontend query reads over canister-to-canister reads when feasible.
- Avoid using backend HTTP outcalls to fetch third-party data unless explicitly required and reviewed.

Types and payload:
- Check that frontend types align with Candid/Rust definitions (optionals, error variants, time/number units).
- Large list endpoints must implement pagination and incremental loading.
- List endpoint returns summary DTO and stays under payload budgets.

Interfaces and upgrades:
- Maintain backward compatibility for Candid (.did).
- Persist state across upgrades using stable memory; version state when layout changes.
- Test upgrade/downgrade paths locally for upgrade-relevant changes.

Candid/data compatibility checklist (MUST):
- [ ] No existing `record` field type was changed.
- [ ] No existing `variant` case payload type was changed.
- [ ] Newly added `record` fields use `opt` when backward compatibility is required.
- [ ] If any breaking change is unavoidable, migration/rollout steps are documented and tested.
- [ ] Deprecated methods are not removed abruptly: the old method is kept and returns an explicit "deprecated" error (instead of trapping) until all clients are migrated.

Storage & upgrade safety checklist (MUST):
- [ ] Data storage location: Where is the new data stored? (Heap / StableBTreeMap / StableLog / other stable structure)
- [ ] Upgrade protection (Pre/Post Upgrade):
  - If the data is stored in Heap variables, it is serialized and written into stable memory in `pre_upgrade`, and restored in `post_upgrade`.
  - If using `ic-stable-structures`, schema/version compatibility has been verified and documented (including migration plan if needed).
- [ ] 4GB Heap limit risk: If data volume grows significantly, confirm it will not exhaust the ~4GB Wasm heap.
  - If growth is unbounded, move data to stable structures and/or enforce explicit limits (pagination, retention, TTL, pruning).

Concurrency and await safety:
- Do not hold mutable state across `await`; follow “check → effect → interact”.
- Handle inter-canister call errors explicitly and inspect reject codes.

Access control and identity (MUST):
 - **Validate Caller:** Always check `ic_cdk::caller()` at the start of every `update` method.
 - **Caller Classification & Logic:**
 - **User vs. Canister:** Distinguish if the caller is a User Principal (frontend) or another Canister (inter-canister automation).
 - **Anonymous:** You MUST explicitly identify if an interface allows anonymous calls.
  - **Authorization Policy:**
    - Administrative methods (not provided for frontend_app) for both `query` and `update` MUST require WLMC-managed permission (configured via WLMC System/User).
    - `query`: prefer `#[ic_cdk::query]` with `#[has_permission_option("<permission_code>")]`.
    - `update`: prefer `#[ic_cdk::update]` with `#[has_permission_option("<permission_code>")]` (default: `"admin"` if unknown; adjust per WLMC).
- Do not introduce “Add Admin” style hardcoding.
 - **Security Disclosure Checklist (MUST):** 
   - [ ] **Audit for Authority:** Check every `update` method for an access control guard.
   - [ ] **Disclose Anonymous Interfaces:** If an `update` method *does not* check authority (is anonymous), you MUST explicitly list it in the review. 
   - [ ] **Verify Justification:** If anonymous access is found, verify it is intentional (e.g., public onboarding/login); otherwise, flag as a security vulnerability

Atomicity / cross-canister transaction safety checklist (MUST):
- [ ] Atomicity: This operation involves cross-canister calls. If step 2 fails, what happens to step 1?
  - The IC does not provide ACID transactions across canisters.
  - A compensating action (rollback), resumable workflow, or explicit state machine must be implemented and documented.
  - The operation must be idempotent and safe to retry (no double credit/debit).
  - Partial-failure states must be observable and recoverable (e.g., `Pending -> Completed` / `Pending -> Failed -> Refunded`).

Timestamp unit convention and sanity checks (MUST)
- IC backend time source: `ic_cdk::api::time()` returns nanoseconds (ns).
- Frontend time source: `Date.now()` returns milliseconds (ms).
- Quick sanity checks (MUST):
  - ms timestamps are typically ~1e12–1e13
  - ns timestamps are typically ~1e18–1e19
  - Add explicit validation/conversion at API boundaries to prevent unit mix-ups (1,000,000x errors).

Logging:
- Avoid leaking principals or sensitive metadata in logs.

### 8.4 Consistency & redundancy checks
- Reuse existing utilities/hooks/services; avoid duplicate wrappers.
- If similar hooks/services/components appear across apps (`ProjectL_frontend`, `admin-app`, `WLCommunity`), extract shared modules instead of copy–paste.
- In the same file or nearby modules, watch for highly similar logic (validation, mapping, error handling, request wrappers) and extract shared helpers.
- For logic duplicated across canisters/libraries (e.g., `validate_*`, `normalize_*`, DTO conversions), move it into `backend/libraries/` or an existing shared library.
- For repeated branch logic in large functions/components, use parameterization/strategy functions/splitting.
- Encourage early returns and splitting long functions without changing behavior.
- When the same backend method is wrapped by multiple services/hooks in different pages, consolidate into a single shared wrapper.
- Currency precision consistency (MUST):
  - Maintain consistent precision rules for SOL/ICP/USDC/Gocin.
  - Use a unified conversion/arithmetic approach to ensure exactness.
  - Add boundary-case tests for rounding and conversion.

### 8.5 Maintainability
- Prefer smaller functions and cohesive types; extract helpers.
- Avoid making >300 LOC files larger unless necessary.
- Improve modularity, naming, and docs where practical.

### 8.6 Quality gates
- Format/lint/tests pass; no new warnings; CI green.
### 8.7 WLMC Dictionary Enforcement — PR Review (MUST)
- During PR review, Copilot MUST detect when WLMC Dictionary should be used, and suggest using an existing Dict Code or creating a new one (avoid duplicates).

Dictionary contract:
- Dict Item fields:
  - Label: dict key
  - Value: dict value
  - Description: item usage description
- Public method (preferred):
  - `admin.get_dict_with_code : (text) -> (opt DictVo) query;`

---

## 9) Git/PR workflow & release hygiene
- Small, focused commits with clear messages (present tense).
- Separate refactors/moves from logic changes where possible.
- Include migration notes (state layout, Candid changes, upgrade steps) in PR descriptions for any breaking or upgrade-relevant changes.
- PR titles/descriptions must be English and action-oriented.

---

## 10) Language policy (explicit & enforceable)

### 10.0 Where bilingual content is allowed (scope)
- Simplified Chinese + English are in code notes and review comments
- Everything else (PR titles/descriptions, code identifiers, logs/errors, UI copy, documentation unless explicitly stated) must follow the rules below.

### 10.1 Copilot PR Review comments
**Required language:** Simplified Chinese (简体中文) + English in both Copilot Pull request overview and comments.

**MANDATORY FORMAT (MUST):**
- Every review item MUST use this exact two-line format:
  ```
  中文：<Simplified Chinese explanation>
  EN: <English explanation>
  ```
- Both lines are REQUIRED for each item.
- The Chinese line MUST start with `中文：` (including the colon).
- The English line MUST start with `EN:` (including the colon and space).
- Do not include secrets, private URLs, or sensitive identifiers in review comments.

**SELF-CHECK + RETRY (MUST):**
- Before returning your review output, scan every item you wrote.
- If any item is missing either the `中文：` line or the `EN:` line, STOP and regenerate that item in the correct format.
- Repeat this check until all items contain both lines.

**FALLBACK (MUST):**
- If you cannot produce bilingual output after retry attempts, return this exact text instead of partial/broken output:
  ```
  中文：无法生成双语评审内容，请手动检查代码变更。
  EN: Unable to generate bilingual review content. Please manually review the code changes.
  ```

**Example format:**
```
中文：请为该列表接口增加分页（默认 20，最大 100），并确保响应体保持在 200KB 以内。
EN: Please add pagination to this list endpoint (default 20, max 100) and keep the response under 200KB.
```

### 10.2 PR titles/descriptions
- English only (concise, action-oriented).
- Include testing evidence (commands run) and migration notes when relevant.

### 10.3 Code identifiers/logs/errors/notes
- English only (replace any Simplified Chinese with English).
- This includes:
  - function/variable/type names
  - log messages
  - error strings
  - inline TODO/NOTE comments in code

### 10.4 User-facing browser console information / UI copy
- English only (replace any Simplified Chinese with English).

### 10.5 Public repos
- Everything must be English only (issues/PRs/docs/code/comments).

---

## 11) Security baseline (project-wide)
If this repository is private; treat all code/config/data as confidential.
Follow ICP-safe patterns (stable state, upgrade hooks, await safety, caller checks).

### 11.1 Secret handling & redaction
- Never include secrets, tokens, keys, or private URLs in code, comments, or tests.
- Use `.env.local` (gitignored) for any local development secrets or overrides.
- Redact sensitive info in logs and errors; prefer structured logging if available.

### 11.2 Input validation & safe parsing
- Validate all inputs from users, other canisters, and any external sources.
- Fail closed on validation or parsing errors.

### 11.3 AuthN/AuthZ expectations
- Always validate caller identity and permissions.
- Centralize authorization; avoid scattered permission checks.

### 11.4 Dependency and supply-chain hygiene
- Minimize dependencies and keep them audited/updated deliberately.
- Avoid introducing unmaintained or high-risk packages.

### 11.5 Responsible logging & incident response pointers (if any)
- Log minimally, redact aggressively.
- Never log secrets or private URLs.
- Avoid logging full payloads for large/PII-sensitive structures.

### 11.6 Cross-chain security (IC bridging to Solana & EVM) (MUST CHECK)
Threat model focus:
- Top threats: tamper-proofing failures and double spending.
- Treat any cross-chain event as untrusted until verified.
- Require a unique, replay-resistant identifier for each cross-chain settlement

---

## 12) Product-level invariants & “hard-to-change later” rules

### 12.1 Identifier strategy (MUST)
- Game identifiers MUST use stable, non-display identifiers:
  - Use `product_id` as the canonical game identifier (not name/slug).
  - `product_id` must be stable across environments and versions.
- Frontend routing, caching keys, and backend storage MUST key by `product_id`.

### 12.2 Browser storage policy (LocalStorage/IndexedDB) (MUST)
- Storage entries MUST be namespaced and versioned:
  - Key format MUST include app + environment + schema version (e.g., `wl:<app>:<env>:v1:<key>`).
- A migration strategy MUST exist for any stored schema change:
- Stored data MUST have TTL where appropriate:


### 12.3 Data polling / long-polling efficiency (MUST/SHOULD)
- Prefer query endpoints for read paths and keep payloads small.
- Polling interval rules (SHOULD):
  - avoid sub-second polling by default
  - use backoff when no changes are detected or on errors
- Long-polling (if used) MUST:
  - support cancellation/cleanup on page unmount/navigation
  - avoid overlapping in-flight requests (one active request per resource key)
  - include server-side pagination and narrow DTOs
- Prefer event-driven/subscription approaches if available; otherwise use incremental polling with cursors.

### 12.4 Add new “invariants” here before they spread
- If you introduce any new global concept (IDs, currency precision rules, settlement states, role names), define it here first so it remains consistent across repos.

### 12.5 Wallet/Top-up and Balance State Management (MUST)
- All features that initiate deposits, manage wallet balance changes, display deposit state, or process game play debits MUST use a unified recharge/top-up & balance state manager and shared operations.
- No component/page/game may reimplement its own deposit flow, balance query, or debit/concurrency control; reuse the shared module to ensure consistent UX and prevent duplicated logic.




---

## 14) UI guidelines (design system & consistency)

**Note:** These are standard requirements for UI-affecting changes. When changes do not involve the UI layer, these checks can be skipped. Placement at the end is for readability and does not reduce the importance of these requirements.

Use tokens and existing styling conventions (Tailwind/SCSS/etc.) in the specific app.
Do not introduce a new styling system.

### 14.1 Typography (MUST, in px)
Global font family:
- "SF Pro", -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif

Typography roles (use tokens; do not hardcode one-off values):
- First Level Title:
  - weight: 600 (Semibold)
  - size: 32 (px-equivalent)
  - color: #000000
- Secondary Title:
  - weight: 600 (Semibold)
  - size: 24
  - color: #000000
- Emphasize Text:
  - weight: 600 (Semibold)
  - size: 16
  - color: #000000
- Body / Regular:
  - weight: 400 (Regular)
  - size: 14
  - color: #000000
- Tips / Explain:
  - weight: 400 (Regular)
  - size: 12
  - color: #000000a6 (65% opacity)
- Micro Text:
  - weight: 400 (Regular)
  - size: 10
  - color: #00000073 (45% opacity)

### 14.2 Color Palette (MUST)
Brand colors:
- primary: #5e40a0
- primary_hover: #834aa3
- primary_active_line: #46258d

Neutral colors:
- text_black: #000000
- text_secondary_65: #000000a6
- text_disabled_25: #00000040
- bg_disabled_10: #0000001a

### 14.3 Spacing Scale (MUST)
Standard spacing tokens are multiples of 4:
- xxs: 4
- xs: 8
- sm: 12
- md: 16
- lg: 24
- xl: 32
- 2xl: 48
- 3xl: 64+

Use spacing tokens; avoid arbitrary spacing values unless matching existing app conventions.

### 14.4 Buttons (MUST)
General rules:
- Provide disabled + loading states for all button variants.
- Use consistent radius/padding/height per variant.
- Text is English-only.

Variants:

A) Solid Button (Primary) (core actions: submit/save)
- height: 40
- padding-x: 8
- radius: >= 20
- states:
  - normal: bg #5e40a0, text #ffffff
  - hover: bg #834aa3, text #ffffff
  - active: bg #46258d, text #ffffff
  - disabled: bg #0000001a, text #00000040

B) Solid Button (Black) (secondary but important: cancel/back)
- height: 20
- padding-x: 8
- radius: >= 10
- states:
  - normal: bg #000000, text #ffffff
  - hover: bg #000000d9, text #ffffff
  - disabled: bg #0000001a, text #00000040

C) Outline Button (secondary with border emphasis)
- height: 20
- padding-x: 8
- radius: >= 20
- states:
  - normal: border #000000, text #000000, bg transparent
  - hover: border #46258d, text #46258d, bg transparent
  - active: border #46258d, text #46258d, bg transparent
  - disabled: border #00000040, text #00000040, bg transparent

D) Dashed Button ("add" actions)
- height: 40
- padding-x: 8
- radius: >= 20
- states:
  - normal: dashed border #000000, text #000000
  - hover: dashed border #46258d, text #46258d
  - disabled: dashed border #00000040, text #00000040

E) Text / Link Button (weakest interaction or links)
- states:
  - normal: text #000000, no decoration
  - hover: text #46258d, underline only for link semantics
  - disabled: text #00000040, no decoration

### 14.5 Input Fields (MUST)
Dimensions:
- single-line height: 36
- textarea height: 114
- radius: single-line 20, textarea 10
- text size: 14

States:
- default:
  - bg: #0000000d
  - border: #0000001a
  - placeholder: #00000073 (45% opacity)
- filled:
  - bg: #0000000d
  - border: #0000001a
  - text: #000000
- error:
  - bg: #0000000d
  - border: #ca1212
  - error message: #c11616 (shown below field)
- disabled:
  - bg: #0000000d
  - border: #0000001a
  - entire field opacity: 30%

Character count (if present):
- bottom-right: #00000033
- on error: #ca1212

### 14.6 Checkbox (MUST)
- size: 12 x 12
- corners: square (no rounding)
States:
- default: fill white, border #000000a6 (65%), text #000000
- checked: fill #000000a6 (65%), border none, icon white check (vector-124)
- disabled: fill white, border #0000004d (30%), text #0000004d (30%)

### 14.7 Select / Dropdown (MUST)
Trigger:
- height: 32
- radius: 20
- default: bg white, border black, text black
- disabled: bg white, border #0000004d (30%), text #0000004d (30%)

Dropdown panel:
- radius: 10
- bg: white
- shadow: 0px 4px 12px #000000 (use closest existing shadow token if present)
Items:
- height: 26
- text size: 12
- hover bg: #0000000d (5%)

### 14.8 Switch (MUST)
- size: 50 x 30
- knob: white circle
States:
- off: track #000000 at 10%
- on: track #5e40a0 at 100%
- disabled on: track #5e40a04c (30%)
- disabled off: track #0000000d (5%)

### 14.9 Tabs (MUST)
- selected typography: Semibold
- unselected typography: Regular or Semibold (match existing app style, but keep contrast clear)
Indicator: bottom line
States:
- active: text #000000, line 2px black
- inactive: text #000000a6 (65%), line 1px #000000a6 (65%)

### 14.10 Popup / Bottom Sheet (MUST)
Container:
- bottom anchored
- radius: 20 (top corners only)
- background: #ffffff
- mask: black overlay at 40–60% opacity
- gesture: swipe down to dismiss (if platform supports it)

Typography:
- header title: 16, Semibold, #000000
- subtitle/body: 12, Regular, #00000073 (≈45% opacity)
- button text: 14 or 16, Medium, black or brand color

Spacing:
- global padding: 16 (left/right/top)
- title -> subtitle gap: 8–12
- content -> button: comfortable spacing using spacing tokens
- bottom area: fixed button bar, white background, pinned

Toast
Maximum width 220px, maximum height 60px, 8px side padding (or spacing on both sides), color #000000,opacity 65%
