# Canary Features Overview

## 📌 Pin Mapping Database

Access to standardized and manufacturer-specific connector pinouts.

### Available Data
- **OBD-II J1962 16-pin** - Universal OBD-II connector (1996+)
- Manufacturer-specific pinouts (expandable)

### API Examples

```rust
// Get OBD-II pinout
let obd2 = PinoutService::get_obd2_pinout()?;
println!("Pin 6: {}", obd2.pins[5].signal_name);  // CAN_H (ISO 15765-4)
println!("Pin 14: {}", obd2.pins[13].signal_name); // CAN_L (ISO 15765-4)

// Search by vehicle
let pinouts = PinoutService::get_manufacturer_pinout("vw", "golf", 2018)?;

// Get by ID
let pinout = PinoutService::get_by_id("obd2_j1962")?;

// List all
let all_pinouts = PinoutService::list_all();
```

---

## 🔌 Protocol Decoders

Decode and encode automotive communication protocols.

### Supported Protocols
- **CAN Bus 2.0B** - High-speed CAN (500 kbps)
- **K-Line (KWP2000)** - Older diagnostic protocol
- **LIN Bus** (planned)

### API Examples

```rust
// CAN Bus
let decoder = ProtocolFactory::create_can_decoder()?;
let frame = decoder.decode(&raw_bytes)?;
println!("CAN ID: 0x{:X}", frame.id);
println!("Data: {:?}", frame.data);

// Encode back
let encoded = decoder.encode(&frame)?;

// K-Line
let kline = ProtocolFactory::create_kline_decoder()?;
let frame = kline.decode(&raw_bytes)?;

// List available protocols
let protocols = ProtocolFactory::list_available_protocols();
```

### Encode/Decode Symmetry
All decoders guarantee encode/decode symmetry - data can be decoded and re-encoded without loss.

---

## ⚠️ Diagnostic Trouble Codes (DTC)

Comprehensive database of OBD-II diagnostic codes.

### Current Database
- **17 Powertrain (P) codes** including:
  - P0301-P0304: Cylinder misfires
  - P0420: Catalyst efficiency
  - P0171/P0172: Fuel trim (lean/rich)
  - P0102-P0118: Sensor circuits
  - P0335/P0340: Crank/cam position sensors
  - P0442/P0455: EVAP leaks
  - P0300: Random misfire

### API Examples

```rust
// Lookup specific code
let dtc = DtcService::lookup_code("P0301")?;
println!("Code: {}", dtc.code);
println!("System: {:?}", dtc.system);  // Powertrain
println!("Description: {}", dtc.description);

// Parse system from code
let system = DtcService::parse_system("P0420")?;  // Powertrain

// Search by keyword
let codes = DtcService::search_by_description("misfire");
// Returns: P0300, P0301, P0302, P0303, P0304

// Get all codes for a system
let powertrain = DtcService::get_by_system(DtcSystem::Powertrain);

// List all codes
let all_codes = DtcService::list_all();
```

### DTC Systems
- **P codes**: Powertrain (Engine, Transmission)
- **B codes**: Body (Doors, Windows, Seats)
- **C codes**: Chassis (Brakes, Steering, Suspension)
- **U codes**: Network (CAN bus, Communication)

---

## 🔧 Service Procedures

Step-by-step automotive service and maintenance procedures.

### Current Procedures
1. **Engine Oil Change** (30 minutes)
   - 10 steps with detailed instructions
   - Safety warnings included
   - Required tools list

2. **Brake System Bleeding** (45 minutes)
   - 11 steps with detailed instructions
   - 13 safety warnings
   - Proper bleeding sequence

### API Examples

```rust
// Get specific procedure
let proc = ServiceProcedureService::get_procedure("oil_change")?;
println!("Name: {}", proc.name);
println!("Estimated time: {} min", proc.estimated_time_minutes.unwrap());
println!("Tools: {:?}", proc.tools_required);

// Iterate through steps
for step in &proc.steps {
    println!("{}. {}", step.order, step.instruction);
    for warning in &step.warnings {
        println!("  ⚠️  {}", warning);
    }
}

// Search by category
let maintenance = ServiceProcedureService::get_maintenance_procedures();
let repair = ServiceProcedureService::get_repair_procedures();
let diagnostic = ServiceProcedureService::get_diagnostic_procedures();

// Search by name
let brake_procs = ServiceProcedureService::search_by_name("brake");

// Filter by time
let quick = ServiceProcedureService::get_by_time_range(0, 35);

// List all
let all_procedures = ServiceProcedureService::list_all();
```

### Procedure Categories
- **Maintenance**: Regular service (oil, filters, fluids)
- **Repair**: Fix broken components
- **Diagnostic**: Troubleshooting procedures
- **Installation**: Adding new components

---

## 🗄️ Database Integration (Optional)

Store custom data alongside embedded library data.

### Features
- Optional PostgreSQL/SQLite database
- Custom schema (`canary`) isolation
- Stores user-specific data:
  - Custom pinouts
  - DTC repair notes
  - Service history logs

### Initialization

```rust
// Without database (embedded data only)
canary_core::initialize(None).await?;

// With SQLite
canary_core::initialize(Some("sqlite://canary.db")).await?;

// With PostgreSQL
canary_core::initialize(Some("postgresql://user:pass@localhost/canary_db")).await?;

// Check if initialized
if canary_core::is_database_initialized() {
    println!("Database connected");
}
```

### Database Schema
- `custom_pinouts`: User-added connector pinouts
- `custom_dtc_notes`: Personal DTC repair notes
- `service_logs`: Maintenance history tracking

---

## 🏗️ Architecture

### Workspace Structure
```
canary/
├── canary-core          # Public API facade
├── canary-models        # Data structures
├── canary-database      # DB connection pool
├── canary-data          # Embedded TOML loaders
├── canary-pinout        # Pin mapping service
├── canary-protocol      # Protocol decoders
├── canary-dtc           # DTC service
└── canary-service-proc  # Service procedures
```

### Data Loading
- **Compile-time**: TOML files embedded with `include_str!`
- **Build-time validation**: `build.rs` validates all data
- **Runtime**: `Lazy<HashMap>` for O(1) lookups
- **Zero overhead**: Data parsed once, cached forever

### Design Patterns
- **SOLID principles**: Single responsibility per crate
- **Declarative Rust**: Functional patterns over imperative
- **Strategy pattern**: Protocol decoders via traits
- **Facade pattern**: `Canary` struct for convenience
- **RAII**: Automatic resource cleanup

---

## 📊 Performance Characteristics

- **Embedded data lookup**: O(1) via HashMap
- **Memory footprint**: ~5-10MB for embedded data
- **Binary size impact**: ~2MB additional
- **Lazy loading**: Data loaded on first access only
- **Zero network latency**: All core data embedded
- **Database connection pool**: Shared across all features

---

## 🔮 Future Enhancements

### Planned Features
- WebAssembly compilation for browser use
- C FFI bindings for C/C++ integration
- Real-time CAN bus streaming
- ML-based DTC prediction
- Cloud sync for custom data
- LIN bus protocol support
- Expanded DTC database (Body, Chassis, Network codes)
- More service procedures
- Vehicle-specific pinout database
- OEM repair manual integration

### Extensibility
The library is designed for easy extension:
- Add new protocols by implementing `ProtocolDecoder` trait
- Add new data by creating TOML files
- Custom database tables for application-specific needs
- Plugin architecture for third-party extensions

---

## 📚 Examples

Run the included examples:

```bash
# Basic usage (all features)
cargo run --example basic_usage

# Protocol decoding deep dive
cargo run --example protocol_decoding

# DTC analysis and search
cargo run --example dtc_analysis

# Service procedures
cargo run --example service_procedures
```

---

## 🧪 Testing

All features are fully tested:

```bash
# Run all tests
cargo test --workspace

# Test specific crate
cargo test -p canary-dtc

# Run with output
cargo test --workspace -- --nocapture
```

**Test Coverage:**
- 33+ unit tests across all crates
- Integration tests for end-to-end workflows
- Compile-time data validation
- Encode/decode symmetry verification
- All tests passing ✅
