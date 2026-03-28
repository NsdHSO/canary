# ECU Simulator

Rust-based ECU (Electronic Control Unit) simulators that respond to UDS (Unified Diagnostic Services) diagnostic commands.

## Overview

This crate provides simulated ECU behavior for testing automotive diagnostic tools without physical hardware. It currently includes three vehicle-specific ECU simulators:

1. **VW Golf ECM** (Engine Control Module) - CAN ID: 0x7E0
2. **GM Silverado ECM** (Engine Control Module) - CAN ID: 0x7E1
3. **Ford F-150 PCM** (Powertrain Control Module) - CAN ID: 0x7E2

## Supported UDS Services

Each simulator implements the following UDS services:

- **0x10**: Diagnostic Session Control (Default, Programming, Extended)
- **0x19**: Read DTC Information (DTCs with status masks)
- **0x22**: Read Data By Identifier (VIN, live engine parameters)
- **0x27**: Security Access (seed/key authentication)
- **0x3E**: Tester Present (keep-alive)

## Simulated DTCs

### VW Golf ECM
- P0301: Cylinder 1 Misfire (Confirmed, MIL on)
- P0420: Catalyst System Efficiency Below Threshold (Stored)
- P0171: System Too Lean (Pending)
- P0506: Idle Control System RPM Lower Than Expected (Stored)

### GM Silverado ECM
- P0128: Coolant Thermostat Issue (Confirmed, MIL on)
- P0300: Random/Multiple Cylinder Misfire (Pending)
- P0455: EVAP System Large Leak (Stored)
- P0101: MAF Circuit Range/Performance (Stored)
- P0606: PCM Processor Fault (Stored)

### Ford F-150 PCM
- P0234: Turbocharger Overboost (Confirmed, MIL on)
- P0299: Turbocharger Underboost (Pending)
- P0401: EGR Flow Insufficient (Stored)
- P0562: System Voltage Low (Stored)
- P2263: Turbo Boost System Performance (Stored)
- P0193: Fuel Rail Pressure Sensor High (Stored)

## Live Data Parameters

Each simulator provides realistic live data for:

- VIN (Vehicle Identification Number)
- Engine RPM
- Vehicle Speed
- Coolant Temperature
- Throttle Position
- Engine Load
- MAF (Mass Air Flow)
- Intake Air Temperature
- Fuel System Status
- Fuel Trims (Bank 1 & 2 for V6/V8)
- O2 Sensor Voltages
- **Ford F-150 only**: Turbo Boost Pressure, Turbo RPM

## Usage

### As a Library

```rust
use ecu_simulator::simulators::{VwGolfEcm, GmSilveradoEcm, FordF150Pcm};

fn main() {
    let mut vw_golf = VwGolfEcm::new();

    // Send UDS request: Read DTCs with status mask 0xFF
    let request = vec![0x19, 0x02, 0xFF];
    let response = vw_golf.handle_uds_request(&request);

    // Response: [0x59, 0x02, availability_mask, ...DTCs...]
    println!("Response: {:02X?}", response);
}
```

### As a Binary

```bash
cargo run --release
```

Output:
```
=== ECU Simulator Starting ===

Initialized ECU Simulators:
  1. VW Golf ECM      - CAN ID: 0x7E0
  2. GM Silverado ECM - CAN ID: 0x7E1
  3. Ford F-150 PCM   - CAN ID: 0x7E2

ECU Simulators ready for UDS diagnostic commands.
...
```

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

All 28 unit tests cover:
- UDS service handlers (session control, DTC reading, data reading, security)
- Vehicle-specific implementations
- Error handling (invalid requests, unsupported services)
- DTC encoding/decoding

## Docker Integration

The simulator is designed to run in a Docker container with a virtual CAN interface (vcan0).

### Build Docker Image

```bash
docker build -t canary-can-simulator .
```

The multi-stage Dockerfile:
1. **Stage 1**: Builds the Rust binary using `rust:latest`
2. **Stage 2**: Creates a minimal Alpine runtime image with the compiled binary

### Run Container

```bash
docker run --privileged canary-can-simulator
```

The container will:
1. Set up the vcan0 interface
2. Start the ECU simulators
3. Wait for CAN integration (Task 4)

## Architecture

```
src/
├── lib.rs                          # Library exports
├── main.rs                         # Binary entry point
├── ecu_simulator.rs                # Base ECU simulator with UDS handlers
└── simulators/
    ├── mod.rs                      # Simulator module exports
    ├── vw_golf_ecm.rs             # VW Golf specific implementation
    ├── gm_silverado_ecm.rs        # GM Silverado specific implementation
    └── ford_f150_pcm.rs           # Ford F-150 specific implementation
```

## UDS Protocol Details

### Request Format
```
[Service ID] [Sub-function/Parameters...]
```

### Response Format

**Positive Response:**
```
[Service ID + 0x40] [Data...]
```

**Negative Response:**
```
[0x7F] [Service ID] [NRC (Negative Response Code)]
```

### Example: Read VIN

**Request:**
```
[0x22, 0xF1, 0x90]  // Read Data By ID: VIN
```

**Response (VW Golf):**
```
[0x62, 0xF1, 0x90, 'W', 'V', 'W', 'Z', 'Z', 'Z', '1', 'K', 'Z', 'B', 'W', '1', '2', '3', '4', '5', '6']
```

## Future Enhancements (Task 4)

The current implementation focuses on UDS protocol logic. Task 4 will add:

- SocketCAN integration for real CAN frame handling
- CAN ISO-TP (Transport Protocol) layer
- Multi-frame message support
- Proper CAN timing and flow control

## Dependencies

- `serde`: Serialization framework
- `thiserror`: Error handling

## License

Dual-licensed under MIT or Apache-2.0.
