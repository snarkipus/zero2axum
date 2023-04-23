# Build Log & Changes

## Resources
- [Zero To Production In Rust](https://www.zero2prod.com/)
- [Rust Axum Full Course - Web Development](https://www.youtube.com/watch?v=XZtlD_m59sM)
- [Building a Rust service with Nix](https://fasterthanli.me/series/building-a-rust-service-with-nix)

## Notes
Backend (Server): `cargo watch -q -c -w src/ -x run`
Frontend (Client): `cargo watch -q -c -w tests/ -x "test -q quick_test -- --nocapture"

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
