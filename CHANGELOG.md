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
- Additional ECU pinouts

## [0.2.0] - 2026-03-27

### Added

#### ECU Pinout Database
- 10 ECU pinouts from 6 manufacturers (VW, Audi, GM, Ford, Toyota, BMW)
- Detailed ECU specifications:
  - Module types (ECM, PCM, TCM, BCM)
  - Signal types (Power, Ground, CAN-H, CAN-L, Analog, Digital, PWM, LIN)
  - Vehicle compatibility matrices
  - Power requirements and fuse ratings
  - Flash memory specifications
  - Supported communication protocols
- Connector specifications with pin layouts
- Part number cross-references
- 4 module types in use across manufacturers

#### Tiered Storage Architecture
- Directory reorganization: `universal/` and `manufacturers/` separation
- Lazy loading infrastructure for manufacturer-specific data
- Gzip compression for all manufacturer data files (.toml.gz)
- Per-manufacturer lazy loaders with on-demand decompression
- Universal data (OBD-II, protocols) preloaded for instant access
- Manufacturer data loaded only when accessed

#### CLI Commands (canary-cli)
- `canary ecu list` - List all available ECUs
- `canary ecu list --manufacturer <mfr>` - Filter ECUs by manufacturer
- `canary ecu show <id>` - Show detailed ECU information
- `canary ecu search <query>` - Search ECUs by name/description
- `canary module list <type>` - List ECUs by module type (ECM, PCM, TCM, BCM)
- Enhanced existing commands with ECU data support

#### Data Models (canary-models)
- `ModuleType` enum (15 types: ECM, PCM, TCM, BCM, DDM, PDM, HVAC, ABS, SRS, EPB, IPC, InfoCenter, Gateway, Telematics, OBD)
- `SignalType` enum (8 types: Power, Ground, CAN-H, CAN-L, Analog, Digital, PWM, LIN)
- `EcuPinout` struct with comprehensive ECU specifications
- `Connector` struct with pin layouts
- `Pin` struct with signal metadata
- `VehicleModel` struct for compatibility tracking
- `PowerRequirements` struct for electrical specifications
- `FlashMemory` struct for memory specifications

#### Pinout Service Functions (canary-pinout)
- `get_ecu_by_id(id: &str)` - Get specific ECU pinout
- `get_ecus_by_manufacturer(mfr: &str)` - Get all ECUs for manufacturer
- `get_ecus_by_module_type(module_type: ModuleType)` - Filter by module type
- `list_manufacturers()` - List all available manufacturers
- Integration with lazy loader system

#### Performance Optimizations
- Lazy loading: 2-5ms per manufacturer (well under 100ms target)
- Gzip compression: ~60% size reduction
- HashMap-based O(1) lookups after loading
- Binary size: ~6MB (under 15MB target)
- Memory efficient: Only load manufacturers as needed

### Changed
- Data directory structure: Split into `universal/` and `manufacturers/`
- `canary-data` crate: Added lazy loader infrastructure
- `canary-pinout` service: Enhanced with ECU-specific functions
- Build system: Automated gzip compression in build.rs
- Documentation: Updated for ECU features

### Technical Details

#### Directory Structure
```
crates/canary-data/data/
├── universal/
│   ├── pinouts/
│   │   └── obd2_j1962.toml
│   ├── protocols/
│   │   ├── can_2.0b.toml
│   │   └── kwp2000.toml
│   ├── dtc/
│   │   └── powertrain_codes.toml
│   └── procedures/
│       ├── oil_change.toml
│       └── brake_bleeding.toml
└── manufacturers/
    ├── index.json
    ├── vw/
    │   ├── ecm.toml.gz
    │   └── tcm.toml.gz
    ├── audi/
    │   └── ecm.toml.gz
    ├── gm/
    │   ├── ecm.toml.gz
    │   └── pcm.toml.gz
    ├── ford/
    │   ├── pcm.toml.gz
    │   └── bcm.toml.gz
    ├── toyota/
    │   ├── ecm.toml.gz
    │   └── rav4_ecm.toml.gz
    └── bmw/
        └── ecm.toml.gz
```

#### ECU Coverage by Manufacturer
- **Volkswagen** (2 ECUs): Golf Mk7 ECM (MED17.25), Passat B7 TCM (09G)
- **Audi** (1 ECU): A4 B8 ECM (MED17.1.1)
- **General Motors** (2 ECUs): Corvette C7 PCM (E92), Silverado ECM (E78)
- **Ford** (2 ECUs): F-150 PCM (EEC-VII), Mustang GT BCM (II)
- **Toyota** (2 ECUs): Camry ECM (Denso), RAV4 Hybrid ECM (Denso)
- **BMW** (1 ECU): E90 335i ECM (MSV80)

#### Performance Metrics
- Universal data loading: <1ms (preloaded)
- Manufacturer lazy load: 2-5ms average
- Gzip decompression: Negligible overhead
- Total binary size: ~6.1MB
- Per-ECU memory footprint: ~50-100KB

### Dependencies
- Added `flate2` 1.0 - Gzip compression/decompression
- Added `clap` 4.6 - CLI argument parsing (canary-cli)

---

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

[Unreleased]: https://github.com/canary-automotive/canary/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/canary-automotive/canary/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/canary-automotive/canary/releases/tag/v0.1.0
