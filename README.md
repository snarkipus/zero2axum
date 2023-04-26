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
