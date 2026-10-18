---
name: openapi-spec-generator
description: >
  Generate or update the OpenAPI 3.1 spec for the vote-visualizer Rust/Axum service by
  extracting routes, extractors, DTOs, and error mappings from services/rust/src/api.rs and
  the modules it uses. Use whenever the user mentions OpenAPI, Swagger, API spec, API
  contract, openapi.yaml, "document the API", or changes routes, DTOs, or error handling in
  services/rust and wants the spec brought in sync.
---

# OpenAPI spec generator (vote-visualizer)

Produce a complete, valid OpenAPI 3.1 spec for the Axum service in `services/rust` by reading
the code. Axum has no annotations — the contract lives in the router, the handler signatures,
the `serde` derives, and the `IntoResponse` impl for `ApiError`.

Defaults (do not ask; the user can override):

| Choice      | Default                                                |
|-------------|--------------------------------------------------------|
| Version     | OpenAPI `3.1.0`                                        |
| Format      | YAML                                                   |
| Output file | `services/rust/openapi.yaml` (update in place if present — merge, don't overwrite hand-written descriptions) |

Never leave placeholders like `# TODO: add schema`. Every value in the spec must be traceable
to code.

---

## Step 1 — Read the files in this order

| # | File                                        | What to pull out                                                                                     |
|---|---------------------------------------------|------------------------------------------------------------------------------------------------------|
| 1 | `services/rust/Cargo.toml`                  | `axum` version (0.8 → `/{id}` path syntax). Check whether `utoipa`/`aide` has been added (see Step 8) |
| 2 | `services/rust/src/main.rs`                 | `router().with_state(..)`, any `.nest()`/`.layer()`, `TcpListener::bind(..)` → `servers`             |
| 3 | `services/rust/src/api.rs`                  | **Source of truth for paths.** `router()`, every `.route()`, handler signatures                      |
| 4 | `services/rust/src/api_dto.rs`              | Response DTOs (`VoteDto`, `BallotDto`) and the `ApiResult` / `ApiPostResult` aliases                 |
| 5 | `services/rust/src/importer.rs`             | `ImportVote` (request, `Deserialize`) and `ImportReport` (response, `Serialize`)                     |
| 6 | `services/rust/src/error.rs`                | `ApiError::public()` → the complete 4xx/5xx set; `ErrorBody` → the error schema                      |
| 7 | `services/rust/src/normalize.rs`            | `VoteChoice::as_str()` → wire values for `choice`                                                    |
| 8 | `services/rust/src/stortinget_models.rs`    | `VoteProposal` / `Party` — the shape stored in `proposals` and returned by `GET /votes/{id}`         |
| 9 | `services/rust/src/storage.rs`              | How rows become DTOs (e.g. ballots ordered by `representative_id`); confirm nothing extra is serialized |
| 10 | `migrations/*.sql`                         | `NOT NULL` → `required`, `CHECK (x >= 0)` → `minimum: 0`, `CHECK (choice IN (..))` → `enum`          |
| 11 | `docs/architecture-design.md`              | Intent and wording for `info.description` and operation descriptions                                 |

Also grep the crate for every `Router::new()` and `.merge(` / `.nest(` to confirm no route is
mounted outside `api.rs`.

---

## Step 2 — Routes

Current shape of `router()` in `api.rs` (re-read it — this is a snapshot, not the contract):

```rust
Router::new()
    .route("/health", get(get_health))
    .route("/votes/{id}", get(get_vote))
    .route("/dev/imports/vote", post(post_dev_imports_vote))
```

Rules:

- `router()` returns `Router<AppState>`; `main.rs` applies `.with_state(state)` with no prefix,
  so paths are emitted exactly as written. If a `.nest(prefix, ..)` appears later, prepend it.
- Each chained method (`get(h).post(h2)`) is a separate operation on the same path item.
- Axum does not redirect trailing slashes — emit paths exactly as registered.
- `operationId` = the handler name converted to camelCase (`get_vote` → `getVote`).
- Tags: `health` for `/health`, `votes` for `/votes/*`, `dev` for `/dev/*`. Define every used tag
  at the root.
- `/dev/*` routes are local development tooling (see the architecture doc). Mark them with
  `x-internal: true` and say so in the description; keep them in the spec.

---

## Step 3 — Extractors → parameters and request bodies

| Extractor in this repo        | OpenAPI mapping                                                                                       |
|-------------------------------|-------------------------------------------------------------------------------------------------------|
| `State<AppState>`             | **Never emit.** Pool and Stortinget client, not API surface                                           |
| `Path(id): Path<i64>`         | Path param named from the route string (`{id}`), `integer`/`int64`, `required: true` + `400` on non-integer |
| `Json<ImportVote>`            | `requestBody`, `application/json`, `required: true`, schema `ImportVote`                              |
| `Query<T>` (if added)         | One query param per field; `Option<_>` / `#[serde(default)]` → not required                           |

Axum extractor rejections add statuses no handler mentions:

- `Path<T>` → `400` (plain-text body, not `ErrorBody`).
- `Json<T>` → `400` malformed JSON, `415` missing/wrong `Content-Type`, `422` valid JSON with the
  wrong shape. Bodies are plain text.

Document these as `text/plain` responses — they do **not** use the `ErrorBody` schema.

---

## Step 4 — Return types → success responses

| Handler returns          | Status                      | Body                                                        |
|--------------------------|-----------------------------|-------------------------------------------------------------|
| `&'static str`           | `200`                       | `text/plain` (`get_health` → `OK`)                          |
| `ApiResult<T>`           | `200`                       | `application/json`, schema of `T`; errors per Step 5        |
| `ApiPostResult<T>`       | the `StatusCode` returned   | `application/json`, schema of `T`; errors per Step 5        |

`post_dev_imports_vote` returns `ApiResult<ImportReport>`, so it responds **`200`**, not `201`.
Document what the code does; if `201` seems more correct, mention it in the summary instead of
writing it into the spec.

---

## Step 5 — `ApiError` → the 4xx/5xx set

`ApiError::public()` in `error.rs` is the only place handler errors become statuses. Read the
match arms and emit **only** what they produce. Current mapping:

| Variant(s)                                                               | Status | Public message (use as `example`) |
|--------------------------------------------------------------------------|--------|-----------------------------------|
| `ApiError::NotFound`, `ImportError::NotFound`                            | `404`  | the variant's `message`           |
| `ImportError::InvalidSourceData` / `Request` / `Decode`                  | `502`  | `upstream request failed`         |
| `ApiError::Database`, `ImportError::Database`                            | `500`  | `internal server error`           |

Body for all of them (`ErrorBody`):

```yaml
ErrorBody:
  type: object
  required: [ error ]
  properties:
    error:
      type: string
  example:
    error: internal server error
```

Rules:

- Use the message from `public()`, never the `#[error(..)]` Display string — those are for logs.
- For `404`, read the call site (`storage::find_vote`, `normalize::find_case_vote`) for the actual
  message to use as the example.
- Which operations get which status:
  - `getVote` → `404`, `500` (DB via `storage::find_vote`; `From<sqlx::Error>`), plus `400` from `Path`.
  - `postDevImportsVote` → `404`, `502`, `500`, plus `400`/`415`/`422` from `Json`.
  - `getHealth` → `200` only.
- Do not pad with `401`/`403`/`409`/`429` — nothing in this service produces them. There is no
  auth layer, so emit no `securitySchemes` and set root-level `security: []` (explicitly public;
  also satisfies Redocly's `security-defined` rule). If an auth layer is added later
  (`.layer()` / `.route_layer()` / custom `FromRequestParts`), read it and add the scheme and `401`.
- Put `NotFound`, `BadGateway`, `InternalError` in `components/responses` and `$ref` them.

---

## Step 6 — Rust types → schemas

No struct in this crate uses `#[serde(rename_all)]`, so **property names are snake_case exactly
as in the Rust field names.** Re-check for `rename`/`rename_all`/`skip`/`flatten` on every struct
you emit.

| Rust                                           | Schema                                                                          |
|------------------------------------------------|---------------------------------------------------------------------------------|
| `i64`                                          | `integer`, `format: int64`                                                      |
| `i32`                                          | `integer`, `format: int32`                                                      |
| `usize`                                        | `integer`, `format: int64`, `minimum: 0`                                        |
| `bool`                                         | `boolean`                                                                       |
| `String`                                       | `string`                                                                        |
| `Option<T>`                                    | `type: [<T>, "null"]`, **still in `required`** — serde serializes `None` as `null` unless `skip_serializing_if` is set |
| `OffsetDateTime` with `time::serde::rfc3339`   | `string`, `format: date-time`                                                   |
| `Vec<T>`                                       | `array`, `items: T`                                                             |
| `serde_json::Value`                            | see `proposals` below                                                           |

Repo-specific notes:

- **`choice`** (`BallotDto.choice: String`) — values come from `VoteChoice::as_str()` and the
  `ballots.choice` CHECK constraint: `enum: [for, against, absent]`.
- **Counts** — `for_count`, `against_count`, `absent_count` are `i32` with `CHECK (>= 0)` in the
  migration → `minimum: 0`.
- **`proposals`** — typed as `serde_json::Value` in `VoteDto`, but it is always the serialized
  `Vec<VoteProposal>` written by the importer. Emit `type: array, items: $ref VoteProposal`
  (with `Party` for `forslag_levert_av_parti_liste`), and add a description noting the DTO holds
  untyped JSON so the shape follows `stortinget_models::VoteProposal`. Field names stay in
  Norwegian, as serialized.
- **`ballots`** — ordered by `representative_id` (from the SQL in `storage::find_vote`); say so
  in the description.
- **Direction** — `ImportVote` is `Deserialize`-only (request); `ImportReport`, `VoteDto`,
  `BallotDto`, `ErrorBody` are `Serialize`-only (response). Never reuse a schema across directions
  when the derives differ.
- **Never emit** `VoteRow`, `BallotRow`, `SourceSnapshot`, `NormalizedVote`, or any upstream
  Stortinget response struct (`CaseVotes`, `VoteResults`, ...) — they are DB- or source-shaped and
  not served, except `VoteProposal`/`Party` as described above.
- Copy `///` doc comments on fields (e.g. `ImportReport.ballot_count`) into `description`.
- Give every schema an `example`. Use realistic Stortinget-shaped values (vote IDs are integers,
  representative IDs are short strings like `"ABC"`, party IDs like `"A"`, `"H"`, `"Sp"`).

---

## Step 7 — Assemble

```yaml
openapi: "3.1.0"
info:
  title: Vote Visualizer API
  version: <services/rust/Cargo.toml package.version>
  description: <from docs/architecture-design.md: read API for imported Storting votes plus local import tooling>
servers:
  - url: http://127.0.0.1:8080
    description: Local development (from TcpListener::bind in main.rs)
tags: [ health, votes, dev ]
paths: { /health, /votes/{id}, /dev/imports/vote }
components:
  schemas: { VoteDto, BallotDto, VoteProposal, Party, ImportVote, ImportReport, ErrorBody }
  responses: { NotFound, BadGateway, InternalError }
```

Only the local server exists in code. Do not invent production URLs — ask the user if they want
others added.

### Quality checklist

- [ ] Every `.route()` in `api.rs` has a path item, and nothing extra
- [ ] Descriptions/examples containing `: ` or backticks are quoted (YAML parse errors otherwise)
- [ ] Every operation has a unique camelCase `operationId` and a tag defined at the root
- [ ] Success status matches the handler's return type (no invented `201`/`204`)
- [ ] Error statuses match `ApiError::public()` plus extractor rejections — nothing more
- [ ] Every `$ref` resolves; no orphaned schemas
- [ ] Property names match serde output (snake_case); nullable fields are `[T, "null"]` and required
- [ ] Every schema and response body has an example
- [ ] Spec lints clean:
  ```sh
  pnpm --package=@redocly/cli dlx redocly lint services/rust/openapi.yaml
  ```
  Expected warnings to leave as-is: `info-license` (the repo has no license) and
  `operation-4xx-response` on `/health` (it cannot fail).

Optionally smoke-test against a running service (`just dev`):
```sh
curl -i http://127.0.0.1:8080/health
curl -i http://127.0.0.1:8080/votes/<id>
curl -i -X POST http://127.0.0.1:8080/dev/imports/vote \
  -H 'content-type: application/json' -d '{"case_id": <case>, "vote_id": <vote>}'
```
Only run the import call if the user agrees — it hits Stortinget's live API and writes to the
local database.

---

## Step 8 — Output

1. Write the spec to `services/rust/openapi.yaml` and report the lint result.
2. Give a short endpoint summary table: method, path, handler, params, request body, responses.
3. Flag code-level oddities found while extracting rather than papering over them in the spec,
   e.g. `proposals` being untyped `serde_json::Value`, the import endpoint returning `200`, or
   `Option` fields serialized as `null` rather than omitted.
4. Note that the spec is hand-extracted and can drift. Offer to add `utoipa` + `utoipa-axum`
   (`#[derive(ToSchema)]` on DTOs, `#[utoipa::path]` on handlers, `OpenApiRouter` in `router()`,
   features `time` for `OffsetDateTime`) so the spec is generated from the code. If `utoipa` or
   `aide` is already in `Cargo.toml`, don't hand-write the spec — produce it from the generated
   document instead.
