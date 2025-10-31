garagex-backend/                # git root
├─ Cargo.toml                   # workspace manifest
├─ Cargo.lock
├─ .env                         # local secrets (gitignored)
├─ .env.example
├─ migrations/                  # sqlx migrations
│  ├─ 001_*.sql
│  └─ ...
├─ docker/                      # docker-compose / Dockerfiles for dev
│  └─ docker-compose.yml
├─ server/                      # main actix-web application crate
│  ├─ Cargo.toml
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ lib.rs                 # optional, expose App factory for tests
│  │  ├─ config.rs
│  │  ├─ state.rs               # AppState
│  │  ├─ error.rs               # common error types / conversions
│  │  ├─ routes.rs              # route registration
│  │  ├─ health.rs
│  │  ├─ auth/
│  │  │  ├─ mod.rs
│  │  │  ├─ handlers.rs
│  │  │  └─ service.rs
│  │  ├─ garages/
│  │  │  ├─ mod.rs
│  │  │  ├─ handlers.rs
│  │  │  └─ repository.rs
│  │  ├─ jobs/
│  │  │  ├─ mod.rs
│  │  │  ├─ handlers.rs
│  │  │  ├─ service.rs
│  │  │  └─ repository.rs
│  │  └─ ...                   # invoices, parts, notifications, customers
│  └─ tests/
│     └─ integration_test.rs
├─ crates/                      # optional shared crates (types, utils)
│  └─ common_types/
│     ├─ Cargo.toml
│     └─ src/lib.rs
└─ scripts/                     # helper scripts (seed, migrate wrappers)
