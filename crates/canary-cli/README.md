# Canary CLI

Command-line interface for the Canary automotive reverse engineering library.

## Installation

```bash
# From source
cargo install --path crates/canary-cli

# Or build locally
cargo build -p canary-cli --release
./target/release/canary --help
```

## Usage

```bash
# Show help
canary --help

# Lookup OBD-II pinout
canary pinout                    # Show all pins
canary pinout --pin 6            # Show specific pin

# Decode CAN bus frame
canary decode 00 00 07 E8 03 41 0C

# Lookup DTC code
canary dtc P0301                 # Specific code
canary dtc --search misfire      # Search by keyword

# Show service procedure
canary service oil_change
canary service brake_bleeding --verbose

# List available data
canary list pinouts
canary list protocols
canary list dtc
canary list procedures
```

## Examples

### OBD-II Pinout

```bash
$ canary pinout --pin 6
OBD-II J1962 16-Pin Connector
══════════════════════════════════════════════════════

Pin 6: CAN_H (ISO 15765-4)
  Protocol: can_2.0b
  Notes: High-speed CAN
```

### CAN Frame Decoding

```bash
$ canary decode 00 00 07 E8 03 41 0C
CAN Bus Frame Decoder
══════════════════════════════════════════════════════

Decoded Frame:
  CAN ID: 0x07E8
  Data Length: 3 bytes
  Data: [03, 41, 0C]
  Timestamp: 2026-03-25 23:33:55 UTC
```

### DTC Lookup

```bash
$ canary dtc P0301
DTC Lookup: P0301
══════════════════════════════════════════════════════

Code: P0301
System: Powertrain
Description: Cylinder 1 Misfire Detected

System Info: Powertrain (Engine/Transmission)
```

### DTC Search

```bash
$ canary dtc --search misfire
DTC Search: 'misfire'
══════════════════════════════════════════════════════

Found 5 code(s):

  P0300 (Powertrain)
    Random/Multiple Cylinder Misfire Detected

  P0301 (Powertrain)
    Cylinder 1 Misfire Detected

  P0302 (Powertrain)
    Cylinder 2 Misfire Detected

  P0303 (Powertrain)
    Cylinder 3 Misfire Detected

  P0304 (Powertrain)
    Cylinder 4 Misfire Detected
```

### Service Procedure

```bash
$ canary service oil_change
Service Procedure: Engine Oil Change
══════════════════════════════════════════════════════

Category: Maintenance
Description: Standard engine oil and filter replacement procedure
Estimated Time: 30 minutes

Required Tools:
  • Oil filter wrench
  • Drain pan
  • Socket wrench
  • Funnel

Steps: 10 total (use --verbose for details)
```

### List Data

```bash
$ canary list dtc
Available DTC Codes: 17
══════════════════════════════════════════════════════

Powertrain (P-codes): 17
  P0300 - Random/Multiple Cylinder Misfire Detected
  P0301 - Cylinder 1 Misfire Detected
  P0302 - Cylinder 2 Misfire Detected
  ...
```

## Features

- ✅ OBD-II pinout lookup with detailed pin information
- ✅ CAN bus frame decoding from hex bytes
- ✅ DTC code lookup and keyword search
- ✅ Service procedure display with step-by-step instructions
- ✅ List all available data (pinouts, protocols, DTCs, procedures)
- ✅ Clean, formatted terminal output
- ✅ Fast startup (embedded data, no database required)

## Architecture

The CLI uses the `canary-core` library with embedded data only (no database).

```rust
use canary_core::{PinoutService, DtcService, ProtocolFactory};
use clap::{Parser, Subcommand};

// Simple CLI parsing with clap
// Direct library calls for data access
// Formatted terminal output
```

## Performance

- **Startup time**: < 100ms
- **Memory usage**: ~10MB
- **Data source**: Embedded (no network/database)
- **Binary size**: ~8MB (includes all data)

## Future Enhancements

- Interactive REPL mode
- Real-time CAN bus monitoring
- Export results to JSON/CSV
- Custom data file loading
- Terminal UI with colors and tables
- Shell completions (bash, zsh, fish)
- Configuration file support

## License

MIT OR Apache-2.0
