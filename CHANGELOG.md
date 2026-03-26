# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- WebAssembly compilation support
- C FFI bindings
- LIN bus protocol decoder
- Real-time CAN streaming
- Body, Chassis, Network DTC codes
- Expanded service procedures
- Vehicle-specific pinout database

## [0.1.0] - 2026-03-26

### Added

#### Core Architecture
- Workspace structure with 8 focused crates
- SOLID principles applied throughout
- Declarative Rust coding patterns
- Comprehensive error handling with `thiserror`
- Type-safe APIs with Result types

#### Data Models (canary-models)
- Embedded data structures (Manufacturer, ConnectorPinout, ProtocolSpec, DiagnosticCode, ServiceProcedure)
- Internal request/response types for database operations
- SeaORM entity definitions for custom tables
- Custom error types with conversions

#### Embedded Data (canary-data)
- Compile-time TOML embedding with `include_str!`
- Build-time validation via `build.rs`
- Lazy static loading with `once_cell::Lazy`
- HashMap-based O(1) lookups
- OBD-II J1962 16-pin pinout
- CAN Bus 2.0B protocol specification
- 17 Powertrain DTC codes
- 2 service procedures (oil change, brake bleeding)

#### Database Integration (canary-database)
- `OnceCell<DatabaseConnection>` singleton pattern
- Optional PostgreSQL/SQLite support
- Connection pool shared across all features
- Custom schema isolation (`canary` schema)

#### Custom Migration System
- Schema-isolated migrations avoiding public schema
- Shell wrapper script (`run_migration.sh`)
- Custom migration binary (`run_canary_migrations`)
- 4 migrations: schema creation + 3 tables
- Migration tracking in `canary.seaql_migrations`

#### Pin Mapping Service (canary-pinout)
- OBD-II J1962 pinout lookup
- Manufacturer/model/year filtering
- Pin mapping search by ID
- List all available pinouts

#### Protocol Decoders (canary-protocol)
- `ProtocolDecoder` trait for extensibility
- CAN Bus 2.0B encoder/decoder
- K-Line (KWP2000) encoder/decoder
- Encode/decode symmetry guarantees
- Protocol factory pattern
- Available protocol listing

#### DTC Service (canary-dtc)
- DTC code lookup (17 powertrain codes)
- System parsing from code (P/B/C/U)
- Keyword search in descriptions
- Filter by system type
- List all codes

#### Service Procedures (canary-service-proc)
- Procedure lookup by ID
- Category-based filtering (Maintenance/Repair/Diagnostic/Installation)
- Name-based search
- Time range filtering
- Oil change procedure (10 steps, 30 min)
- Brake bleeding procedure (11 steps, 45 min, 13 warnings)

#### Public API (canary-core)
- Facade pattern for convenient access
- Service re-exports
- Optional database initialization
- Comprehensive examples
- Doc tests

#### Testing
- 33+ unit tests across all crates
- Integration tests
- Doc tests
- Encode/decode symmetry tests
- 100% public API coverage

#### Examples
- `basic_usage.rs` - All features demonstration
- `protocol_decoding.rs` - CAN/K-Line detailed examples
- `dtc_analysis.rs` - DTC search and analysis
- `service_procedures.rs` - Service procedure walkthrough

#### Documentation
- Comprehensive README with quick start
- FEATURES.md with detailed API documentation
- PROJECT_SUMMARY.md with implementation details
- CONTRIBUTING.md with contribution guidelines
- Rustdoc comments on all public APIs
- MIT and Apache-2.0 dual licensing

### Technical Details

#### Performance
- O(1) embedded data lookups via HashMap
- Lazy loading (data parsed on first access)
- Zero-cost abstractions
- ~5-10MB memory footprint
- ~2MB binary size increase

#### Design Patterns
- Facade: `Canary` struct for convenience
- Strategy: Protocol decoders via traits
- Builder: Query builders for complex searches
- Newtype: Type-safe wrappers
- RAII: Automatic resource cleanup
- Singleton: Database connection pool

#### Code Quality
- Zero compiler errors
- Minimal warnings (2 dead_code)
- All tests passing
- Clippy clean
- Formatted with rustfmt

### Dependencies
- `sea-orm` 1.1 - Database ORM
- `tokio` 1.x - Async runtime
- `serde` 1.0 - Serialization
- `thiserror` 2.0 - Error handling
- `chrono` 0.4 - Date/time handling
- `toml` 0.8 - TOML parsing
- `once_cell` 1.21 - Lazy statics

### Breaking Changes
None - initial release

### Deprecated
None - initial release

### Security
- Input validation on all external data
- Type-safe database queries via SeaORM
- No SQL injection vulnerabilities
- Safe handling of binary protocol data

---

## Release Notes

### v0.1.0 - Initial Release

This is the first production-ready release of Canary, a comprehensive Rust library for automotive reverse engineering.

**Key Features:**
- 📌 Pin mapping database (OBD-II J1962)
- 🔌 Protocol decoders (CAN Bus, K-Line)
- ⚠️ DTC database (17 codes)
- 🔧 Service procedures (2 procedures)

**Architecture:**
- 8-crate modular workspace
- SOLID principles applied
- Declarative Rust patterns
- Optional database integration

**Quality:**
- 33+ tests passing
- 4 working examples
- Comprehensive documentation
- Production ready

**Installation:**
```toml
[dependencies]
canary-core = "0.1.0"
```

**Quick Start:**
```rust
use canary_core::{PinoutService, DtcService};

let obd2 = PinoutService::get_obd2_pinout()?;
let dtc = DtcService::lookup_code("P0301")?;
```

See README.md and FEATURES.md for complete documentation.

---

[Unreleased]: https://github.com/canary-automotive/canary/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/canary-automotive/canary/releases/tag/v0.1.0
