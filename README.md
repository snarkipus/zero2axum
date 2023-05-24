# Build Log & Changes

## Rust Resources
- [Zero To Production In Rust](https://www.zero2prod.com/)
- [Rust Axum Full Course - Web Development](https://www.youtube.com/watch?v=XZtlD_m59sM)
- [Introduction to Axum](https://www.youtube.com/playlist?list=PLrmY5pVcnuE-_CP7XZ_44HN-mDrLQV4nS)
- [Building a Rust service with Nix](https://fasterthanli.me/series/building-a-rust-service-with-nix)

## SurrealDB Resources
- [SurrealDB - Rust Embedded Database - Quick Tutorial](https://www.youtube.com/watch?v=iOyvum0D3LM)
- [Beyond Surreal? A closer look at NewSQL Relational Data](https://www.youtube.com/watch?v=LCAIkx1p1k0)
- [Testing SurrealDB](https://dev.to/ndrean/testing-surrealdb-1kjl)
- [SurrealDB: Your Ultimate Guide to Smooth Installation and Configuration](https://travishorn.com/surrealdb-your-ultimate-guide-to-smooth-installation-and-configuration)
- [Awesome Surreal](https://github.com/surrealdb/awesome-surreal)
- [Concurrency Example](https://github.com/surrealdb/surrealdb/blob/main/lib/examples/concurrency/main.rs)
### DB Utilities
- [surrealdb-migrations](https://github.com/Odonno/surrealdb-migrations/)
- [Surrealist DB Explorer](https://github.com/StarlaneStudios/Surrealist)

### DB CLI / SurrealQL

<details>

<summary>Example SurrealQL Terminal Session</summary>

```
❯ target/release/surreal sql --conn ws://localhost:8000 --user surreal --pass password
> INFO FOR KV;
[{ ns: { default: 'DEFINE NAMESPACE default' } }]

> USE NS default;
[]

default> INFO FOR NS;
[{ db: { "`03358854-c64b-4218-ac5e-0a9f0ef6d9e0`": 'DEFINE DATABASE `03358854-c64b-4218-ac5e-0a9f0ef6d9e0`', "`0db7dbcd-36ab-44ab-bd4e-7e0a671fc257`": 'DEFINE DATABASE `0db7dbcd-36ab-44ab-bd4e-7e0a671fc257`', "`1d5d717c-e548-45a5-bca9-1f9d555df047`": 'DEFINE DATABASE `1d5d717c-e548-45a5-bca9-1f9d555df047`', "`1e88b4cd-7a1b-4dc9-a7a6-4b9113a0104b`": 'DEFINE DATABASE `1e88b4cd-7a1b-4dc9-a7a6-4b9113a0104b`', "`2ea5190c-bd20-43b6-b9f1-a72d61f08eb8`": 'DEFINE DATABASE `2ea5190c-bd20-43b6-b9f1-a72d61f08eb8`', "`30332091-8f2f-4276-a479-56be361c60a5`": 'DEFINE DATABASE `30332091-8f2f-4276-a479-56be361c60a5`', "`3226719f-41f7-4b0c-9b0b-44f81318838b`": 'DEFINE DATABASE `3226719f-41f7-4b0c-9b0b-44f81318838b`', "`32e61988-1490-4940-8da7-b24dc41f6125`": 'DEFINE DATABASE `32e61988-1490-4940-8da7-b24dc41f6125`', "`419938a5-f9e1-41ac-943d-3289dc1b98ad`": 'DEFINE DATABASE `419938a5-f9e1-41ac-943d-3289dc1b98ad`', "`53a87388-8163-447f-b512-914b153045fc`": 'DEFINE DATABASE `53a87388-8163-447f-b512-914b153045fc`', "`556f7657-2d7d-4dea-8f53-2b2337c62276`": 'DEFINE DATABASE `556f7657-2d7d-4dea-8f53-2b2337c62276`', "`5e8c1b46-ddd2-40f9-8cdf-3603665fb432`": 'DEFINE DATABASE `5e8c1b46-ddd2-40f9-8cdf-3603665fb432`', "`5ed41f41-8538-435a-9e73-1e3c455a2252`": 'DEFINE DATABASE `5ed41f41-8538-435a-9e73-1e3c455a2252`', "`6416fdf9-a335-4ee2-9c5e-1b7048c14ce2`": 'DEFINE DATABASE `6416fdf9-a335-4ee2-9c5e-1b7048c14ce2`', "`736d865f-0fcf-4834-a506-a117a84384f3`": 'DEFINE DATABASE `736d865f-0fcf-4834-a506-a117a84384f3`', "`7b5c1533-165f-499d-a126-2596fe2ff5fd`": 'DEFINE DATABASE `7b5c1533-165f-499d-a126-2596fe2ff5fd`', "`8513744a-4148-422d-8029-a83555cdce30`": 'DEFINE DATABASE `8513744a-4148-422d-8029-a83555cdce30`', "`9744022e-ef87-41c1-9bf2-7c01b6ec01ab`": 'DEFINE DATABASE `9744022e-ef87-41c1-9bf2-7c01b6ec01ab`', "`a2bb8f4e-1b60-4aab-958d-3cadfcd4f682`": 'DEFINE DATABASE `a2bb8f4e-1b60-4aab-958d-3cadfcd4f682`', "`a4dbdf37-008e-4052-813c-db7df07f9f79`": 'DEFINE DATABASE `a4dbdf37-008e-4052-813c-db7df07f9f79`', "`b9182537-ab0f-4d78-b821-1d7338ecfa71`": 'DEFINE DATABASE `b9182537-ab0f-4d78-b821-1d7338ecfa71`', "`c51c93e5-118c-4111-901c-4be338729174`": 'DEFINE DATABASE `c51c93e5-118c-4111-901c-4be338729174`', "`d231d5f7-55b8-4d45-9789-8043624390fd`": 'DEFINE DATABASE `d231d5f7-55b8-4d45-9789-8043624390fd`', "`df0ced2c-945d-4e73-83cc-bde46385077f`": 'DEFINE DATABASE `df0ced2c-945d-4e73-83cc-bde46385077f`', "`e097fb8e-c07e-4676-b7f9-99073fac7398`": 'DEFINE DATABASE `e097fb8e-c07e-4676-b7f9-99073fac7398`', "`e485535d-a90f-4d74-a163-82b4890bf8a8`": 'DEFINE DATABASE `e485535d-a90f-4d74-a163-82b4890bf8a8`', "`f2d0c249-a5ed-42a8-9d01-536bb6c69ef3`": 'DEFINE DATABASE `f2d0c249-a5ed-42a8-9d01-536bb6c69ef3`', "`f7deb705-6e6c-4b61-be16-54e2e6ce2a4f`": 'DEFINE DATABASE `f7deb705-6e6c-4b61-be16-54e2e6ce2a4f`', "`fa49ab02-c35a-4d93-b210-f0756031a008`": 'DEFINE DATABASE `fa49ab02-c35a-4d93-b210-f0756031a008`', "`fe8abdf9-00c7-48c6-862e-101a9c5cceec`": 'DEFINE DATABASE `fe8abdf9-00c7-48c6-862e-101a9c5cceec`', newsletter: 'DEFINE DATABASE newsletter' }, nl: {  }, nt: {  } }]

default> USE DB `03358854-c64b-4218-ac5e-0a9f0ef6d9e0`;
There was a problem with the database: There was a problem with the database: Parse error on line 1 at character 15 when parsing '-c64b-4218-ac5e-0a9f0ef6d9e0;'

default/03358854-c64b-4218-ac5e-0a9f0ef6d9e0> INFO FOR DB;
[{ dl: {  }, dt: {  }, fc: {  }, pa: {  }, sc: {  }, tb: { subscriptions: 'DEFINE TABLE subscriptions SCHEMAFULL' } }]

default/03358854-c64b-4218-ac5e-0a9f0ef6d9e0> INFO FOR TABLE subscriptions;
[{ ev: {  }, fd: { email: 'DEFINE FIELD email ON subscriptions TYPE string ASSERT $value != NONE AND is::email($value)', id: 'DEFINE FIELD id ON subscriptions TYPE string ASSERT $value != NONE', name: 'DEFINE FIELD name ON subscriptions TYPE string ASSERT $value != NONE', subscribed_at: 'DEFINE FIELD subscribed_at ON subscriptions TYPE datetime ASSERT $value != NONE' }, ft: {  }, ix: { email: 'DEFINE INDEX email ON subscriptions FIELDS email UNIQUE', idIndex: 'DEFINE INDEX idIndex ON subscriptions FIELDS id' } }]

default/03358854-c64b-4218-ac5e-0a9f0ef6d9e0> SELECT * FROM subscriptions;
[{ email: 'ursula_le_guin@gmail.com', id: subscriptions:xf8xb288jdyx8ay12r1k, name: 'le guin', subscribed_at: '2023-05-21T03:24:38.086917396Z' }]
```

</details>

## Solutions
- [Zero To Production (with axum)](https://github.com/mattiapenati/zero2prod)
- [An implementation of Zero To Production In Rust using Axum instead of Actix (partial)](https://github.com/SaadiSave/zero2prod)


## Docs

| Crate | Docs |
| --- | --- |
| Axum | [0.6.16](https://docs.rs/axum/0.6.16/axum/) |
| Tokio | [1.27.0](https://docs.rs/tokio/1.27.0/tokio/) |
| tracing | [0.1.37](https://docs.rs/tracing/0.1.37/tracing/) |
| color_eyre | [0.6.2](https://docs.rs/color-eyre/0.6.2/color_eyre/) |
| rstest | [0.17.0](https://docs.rs/rstest/0.17.0/rstest/) |
| surrealdb | [1.0.0-beta.9](https://docs.rs/surrealdb/1.0.0-beta.9+20230402/surrealdb/) |

## Watch Commands
Backend (Server):

`cargo watch -q -c -w src/ -x run`

Frontend (Client):

`cargo watch -q -c -w tests/ -x "test -q quick_test -- --nocapture"`

## Chapter 1
- Toolchain: 1.69.0
- Linker: [mold](https://github.com/rui314/mold) (v1.11.0)
- Code Coverage: [LLVM source-based coverage](https://github.com/taiki-e/cargo-llvm-cov) w/[Codecov](https://about.codecov.io/) integration

## Chapter 2
- no change

## Chapter 3
- Framework: Axum (0.6.16)
- Aync Runtime: Tokio (1.27.0)
- Environment: [git-crypt](https://dev.to/heroku/how-to-manage-your-secrets-with-git-crypt-56ih), [direnv](https://direnv.net/)
- Error Handling:
  - [Sentry](https://www.sentry.io)
  - color-eyre
- Logs:
  - tracing/tracing-subscriber
  - serde, serde_json
- Testing:
  - httpc_test, rstest
- Database:
  - SurrealDB, [surrealdb-migrations](https://github.com/Odonno/surrealdb-migrations)

## Chapter 4
- Telemetry:
  - [ ] TODO: OpenTelemetry w/Honeycomb: [Honeycomb.io](https://ui.honeycomb.io)
  - [ ] TODO: Verify Sentry (will address with error handling - it's a mess right now)

## Chapter 5
- A bit different w/SurrealDB
  - No offline compile-time verification
  - No 'lazy' connection to SurrealDB (would require refactoring initialization code to endpoint handler - doable, but dumb)

- ok, so I'm going to have to refactor this stupid thing - it totally breaks the flow of the book if I don't
  - [x] instead of connection pool, pass configurations
  - [x] initialize database connection (post init.sh migration) at the handler
  - [x] major refactor of the tests to create configs/migrations within the tests

### Deployment Configuration
- zero2axum:
  - ~~Dockerfile deployment via `spec.yaml` to Digital Ocean Apps~~
    - turns out that managing `git-crypt` secrets with Dockerfile deployment and DO Apps Framework sucks (or at least I couldn't figure it out)
  - switched to Fly.io:
    - [x] Local deployment via `flyctl deploy` after using `fly launch` to generate the `fly.toml` config file worked painslessly (and without wrestling with git-crypt)
    - [x] [Fly.io CD Setup](https://fly.io/docs/app-guides/continuous-deployment-with-github-actions/)
- SurrealDB (5.4.4 Connecting To Digital Ocean’s Postgres Instance): 
  - [x] ~~VPS w/Docker deployment to Digital Ocean Droplet~~
  - this got needlessly complicated and non-automated (domain names, etc.) - might as well just host my own
  - [x] Host SurrealDB on personal VPS (in my case, k3s cluster running on Hetzner)
  - [x] SSL does horrible, terrible things and doesn't work [Bug: 1929](https://github.com/surrealdb/surrealdb/issues/1929) ([Fix: PR#1960](https://github.com/surrealdb/surrealdb/pull/1960))
  - [x] Refactor 'production' environment to reflect `Wss` vs. `Ws` connection and new database endpoint
  - [ ] noticed that the initial schemaful migration run in the init script for local dev isn't being run for prod, so there are no unique constraints ... this would have broken with Postgres ... need to figure this out
  - for now, just doing a manual migration using the surreal cli via:
    `surreal import --conn https://my.db.here -u surreal -p password --ns default --db newsletter schemas/script_migration.surql` (also requires nightly, source built surreal since ssl is broken in beta-9)

## Chapter 6
- Type Safety: just a note, lack of `sqlx` kinda sucks ... 6.5 clearly shows the issue where a query is binding a field to a struct (instead of &str), and I know that will explode ... but it's silent without anything that guarantees type-safe queries.

- pretty straight forward - only some minor errata around the `fake` crate and using `Arbitrary` ... no longer using rng as a trait, now it uses a struct - there's a link to the issue in the source
    
## Chapter 7
### 7.2.2.2 Connection Pooling
> ... most HTTP clients offer connection pooling: after the first request to a remote server
has been completed, they will keep the connection open (for a certain amount of time) and re-use it if we
need to fire off another request to the same server, therefore avoiding the need to re-establish a connection
from scratch.

>`reqwest` is no different - every time a Client instance is created reqwest initialises a connection pool under
the hood.

>To leverage this connection pool we need to reuse the same Client across multiple requests.
It is also worth pointing out that Client::clone does not create a new connection pool - we just clone a
pointer to the underlying pool.

### 7.2.2.3 How to Reuse the same `reqwest::Client` in ~~`actix-web`~~ `Axum`
> To re-use the same HTTP client across multiple requests in actix-web we need to store a copy of it in the
application context - we will then be able to retrieve a reference to Client in our request handlers using an
extractor (e.g. actix_web::web::Data).

How do?

#### Option 1:
> Derive the Clone trait for EmailClient, build an instance of it once and then pass a clone to app_data
every time we need to build an App

#### Option 2:
> Wrap EmailClient in actix_web::web::Data (an Arc pointer) and pass a pointer to app_data every
time we need to build an App - like we are doing with PgPool:

Which one?

> If EmailClient were just a wrapper around a Client instance, the first option would be preferable - we avoid
wrapping the connection pool twice with Arc. This is not the case though: EmailClient has two data fields attached (base_url and sender). The first implementation allocates new memory to hold a copy of that data every time an App instance is created, while the second shares it among all App instances. That’s why we will be using the second strategy.

#### NOTE:
- axum-macros crate with `#[debug_handler]` on the route handler makes [debugging](https://docs.rs/axum-macros/latest/axum_macros/attr.debug_handler.html) A LOT easier

Ok, so this might be one of the bigger differences I've seen between Actix and Axum ... the sharing of State information. Where Actix allows the `app_data` to be passed atomically, Axum requires state to be packaged into a struct of some kind.

Regardless, the naive approach of just cloning the entire structure would be "Option 1" ... so we'll have to wrap the EmailClient in an `Arc` by hand

So, for us ... we can do something like (no idea if this is write - had to derive `clone` on the whole thing which seems like it's not quite right):

```rust 
#[derive(Clone)]
pub struct AppState {
    pub configuration: Settings,
    pub email_client: Arc<EmailClient>,
}
```
Then, we can setup the `AppState` like this in the async `run()` function:
```rust
let state = AppState {
    configuration,
    email_client: Arc::new(email_client),
};
```
Then, on the handler side, we can do the following to extract `AppState`:
```rust
pub async fn handler_subscribe(
    Extension(state): Extension<AppState>,
    Form(data): Form<FormData>,
) -> Result<impl IntoResponse> {...}
```
Oddly, that works ... tests are all green.

### 7.2.3 HTTP Mocking with ~~`wiremock`~~ `mockito`
Long story short - just use `wiremock`.
#### Issues
- no match trait to implement, only an `Into<Matcher>` which can return one of the `Matcher` enum types ... probably a way to do that, but beyond my skill level
- no exposed `.with_delay()` method ... you have to use `.with_chunked_body()` and pass a closure ... and the sleep thread seems to be block in the main test thread