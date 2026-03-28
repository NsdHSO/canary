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

# ECU Commands
canary ecu list                          # List all ECUs
canary ecu list --manufacturer vw        # Filter by manufacturer
canary ecu show vw_golf_mk7_2015_ecm_med1725  # Show ECU details
canary ecu search "golf"                 # Search ECUs

# Module Commands
canary module list ECM                   # List ECUs by module type
canary module list PCM
canary module list TCM

# Lookup OBD-II pinout
canary pinout                            # Show all pins
canary pinout --pin 6                    # Show specific pin

# Decode CAN bus frame
canary decode 00 00 07 E8 03 41 0C

# Lookup DTC code
canary dtc P0301                         # Specific code
canary dtc --search misfire              # Search by keyword

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

### ECU Commands

#### List All ECUs

```bash
$ canary ecu list
All available ECUs:
══════════════════════════════════════════════════════

VW:
  vw_passat_b7_2014_tcm_09g (TCM, Continental)
  vw_golf_mk7_2015_ecm_med1725 (ECM, Bosch)

AUDI:
  audi_a4_b8_2012_ecm_med1711 (ECM, Bosch)

GM:
  gm_silverado_2017_ecm_e78 (ECM, Delphi)
  gm_corvette_c7_2016_pcm_e92 (PCM, Delphi)

FORD:
  ford_f150_2018_pcm_eec7 (PCM, Continental)
  ford_mustang_gt_2019_bcm_ii (BCM, Continental)

TOYOTA:
  toyota_rav4_hybrid_2020_ecm_denso (ECM, Denso)
  toyota_camry_2018_ecm_denso (ECM, Denso)

BMW:
  bmw_e90_335i_2010_ecm_msv80 (ECM, Bosch)

Total: 10 ECUs
```

#### Show Specific ECU

```bash
$ canary ecu show vw_golf_mk7_2015_ecm_med1725
ECU Details: vw_golf_mk7_2015_ecm_med1725
══════════════════════════════════════════════════════
Module Type: ECM
Manufacturer: vw (ECU by Bosch)
Part Numbers: 03L906018JJ, 03L906018LG, 03L906018MM

Vehicle Compatibility:
  Volkswagen Golf Mk7 (2015-2020, 2.0L TDI)

Connectors: 1
  T121 - 121-pin Bosch EV1.4 (13 pins)

Power Requirements:
  Voltage: 9-16V (nominal: 12V)
  Max Current: 20A
  Fuse Rating: 25A

Protocols: can_2.0b, kwp2000, uds

Memory:
  Flash: 2048 KB
  RAM: 256 KB
  EEPROM: 64 KB
  CPU: Infineon TriCore TC1797
```

#### List by Manufacturer

```bash
$ canary ecu list --manufacturer ford
ECUs for manufacturer 'ford':
══════════════════════════════════════════════════════

Found 2 ECU(s):

  ford_f150_2018_pcm_eec7 (PCM)
    Manufacturer: Continental
    Part Numbers: JL3A-12A650-AKA, JL3A-12A650-ALA

  ford_mustang_gt_2019_bcm_ii (BCM)
    Manufacturer: Continental
    Part Numbers: JR3T-14B476-AA, JR3T-14B476-AB
```

#### Search ECUs

```bash
$ canary ecu search "corvette"
ECU Search Results for 'corvette':
══════════════════════════════════════════════════════

Found 1 ECU(s):

  gm_corvette_c7_2016_pcm_e92 (PCM)
    Name: Corvette C7 PCM (E92)
    Manufacturer: Delphi
```

#### List by Module Type

```bash
$ canary module list ECM
ECUs with module type 'ECM':
══════════════════════════════════════════════════════
  vw_golf_mk7_2015_ecm_med1725 (Bosch)
  audi_a4_b8_2012_ecm_med1711 (Bosch)
  gm_silverado_2017_ecm_e78 (Delphi)
  toyota_rav4_hybrid_2020_ecm_denso (Denso)
  toyota_camry_2018_ecm_denso (Denso)
  bmw_e90_335i_2010_ecm_msv80 (Bosch)
```

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

- ✅ ECU pinout database (10 ECUs from 6 manufacturers)
- ✅ ECU search and filtering by manufacturer/module type
- ✅ Detailed ECU specifications (connectors, power, memory, protocols)
- ✅ OBD-II pinout lookup with detailed pin information
- ✅ CAN bus frame decoding from hex bytes
- ✅ DTC code lookup and keyword search
- ✅ Service procedure display with step-by-step instructions
- ✅ List all available data (ECUs, pinouts, protocols, DTCs, procedures)
- ✅ Clean, formatted terminal output
- ✅ Fast startup with lazy loading (<5ms per manufacturer)
- ✅ Compact binary (~6MB with all data)

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
- **Lazy load time**: 2-5ms per manufacturer
- **Memory usage**: ~10-15MB (with all manufacturers loaded)
- **Data source**: Embedded with lazy loading (no network/database)
- **Binary size**: ~6.1MB (includes compressed data)
- **Compression**: Gzip for manufacturer data (~60% reduction)

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
