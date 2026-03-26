# Canary 🐤

> Automotive Reverse Engineering Library for Rust

**Canary** is a comprehensive Rust library providing standardized access to automotive diagnostic data, protocol decoders, and service procedures. Named after the CAN bus protocol, it serves as an early warning system for inconsistent automotive data - like a canary in a coal mine.

## Features

- **📌 Pin Mapping Database** - Universal and manufacturer-specific connector pinouts (OBD-II, ECU, sensors)
- **🔌 Protocol Decoders** - CAN Bus, K-Line, LIN bus decoding and encoding
- **⚠️ Diagnostic Codes** - Comprehensive DTC (Diagnostic Trouble Code) database
- **🔧 Service Procedures** - Repair and maintenance procedure documentation

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
canary-core = { path = "path/to/canary/crates/canary-core" }
```

### Example Usage

```rust
use canary_core::{PinoutService, ProtocolFactory, ProtocolDecoder, DtcService, ServiceProcedureService};

fn main() -> Result<(), canary_core::CanaryError> {
    // Get OBD-II pinout
    let obd2 = PinoutService::get_obd2_pinout()?;
    println!("Pin 6: {}", obd2.pins[5].signal_name);  // CAN_H

    // Decode CAN frame
    let decoder = ProtocolFactory::create_can_decoder()?;
    let frame = decoder.decode(&raw_bytes)?;

    // Lookup DTC
    let dtc = DtcService::lookup_code("P0301")?;
    println!("DTC: {}", dtc.description);  // Cylinder 1 Misfire

    // Get service procedure
    let proc = ServiceProcedureService::get_procedure("oil_change")?;
    println!("Steps: {}", proc.steps.len());  // 10 steps

    Ok(())
}
```

## Architecture

Canary follows a modular workspace structure:

```
canary/
├── crates/
│   ├── canary-core/         # Public API facade
│   ├── canary-models/       # Data structures and error types
│   ├── canary-database/     # Database connection pool
│   ├── canary-pinout/       # Pin mapping service
│   ├── canary-protocol/     # Protocol decoders (CAN, K-Line)
│   ├── canary-dtc/          # Diagnostic code service
│   ├── canary-service-proc/ # Service procedure service
│   └── canary-data/         # Embedded TOML data + loaders
└── migration/               # Database migrations
```

### Data Strategy

**Hybrid Storage:**
- **Embedded Data** (compiled into library): Universal pinouts, standard protocols, common DTCs
- **External Database** (PostgreSQL/SQLite): Custom pinouts, personal notes, service logs

### Design Principles

- **SOLID Principles**: Each crate has a single responsibility
- **Declarative Rust**: Functional patterns (iterators, pattern matching) over imperative code
- **Zero-cost Abstractions**: `Lazy<HashMap>` for O(1) lookups
- **Schema Isolation**: Custom PostgreSQL schema (`canary`) avoids touching `public` schema

## Running Tests

```bash
# Run all tests
cargo test --workspace

# Run example
cargo run --example basic_usage
```

## Database Migrations

Custom migration system avoids PostgreSQL's public schema:

```bash
# Set database URL
export DATABASE_URL="postgresql://user:pass@localhost/canary_db"

# Run migrations
./run_migration.sh
```

## Project Status

✅ **Core Features Complete:**
- Workspace structure (8 crates)
- Embedded data loading with compile-time validation
- All feature services implemented
- Custom schema-isolated migrations
- Full test coverage (33+ tests passing)

📦 **Sample Data Included:**
- OBD-II J1962 16-pin pinout
- CAN 2.0B protocol specification
- 7 powertrain DTC codes
- Oil change service procedure

## Future Enhancements

- WebAssembly support for browser use
- C FFI bindings for C/C++ integration
- Real-time CAN bus streaming
- ML-based DTC prediction
- Cloud sync for custom data

## License

MIT OR Apache-2.0
