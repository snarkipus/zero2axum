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

## Solutions
- [Zero To Production (with axum)](https://github.com/mattiapenati/zero2prod)
- [An implementation of Zero To Production In Rust using Axum instead of Actix](https://github.com/SaadiSave/zero2prod)


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
  - [x] Refactor 'production' environment to reflect `Wss` vs `Ws` connection and new database endpoint

## Chapter 6
- Type Safety: just a note, lack of `sqlx` kinda sucks ... 6.5 clearly shows the issue where a query is binding a field to a struct (instead of &str), and I know that will explode ... but it's silent without anything that guarantees type-safe queries.

- pretty straight forward - only some minor errata around the `fake` crate and using `Arbitrary` ... no longer using rng as a trait, now it uses a struct - there's a link to the issue in the source
    
