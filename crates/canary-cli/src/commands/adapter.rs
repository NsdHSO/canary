use canary_hardware::{
    list_adapter_types, create_adapter, AdapterType, BluetoothAdapter,
};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct AdapterArgs {
    #[command(subcommand)]
    pub command: AdapterCommand,
}

#[derive(Subcommand)]
pub enum AdapterCommand {
    /// List available adapter types
    List,

    /// Connect to a CAN adapter
    Connect {
        /// Adapter type: vcan0, can0, wifi, bluetooth
        adapter: String,

        /// Connection target (IP for WiFi, device name for Bluetooth)
        #[arg(short, long)]
        target: Option<String>,
    },

    /// Test adapter connection
    Test {
        /// Adapter to test (e.g., vcan0)
        adapter: String,
    },

    /// Scan for Bluetooth OBD adapters
    Scan {
        /// Scan type: bluetooth
        #[arg(default_value = "bluetooth")]
        scan_type: String,

        /// Scan timeout in seconds
        #[arg(short, long, default_value = "5")]
        timeout: u64,
    },
}

pub async fn handle_adapter(args: AdapterArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        AdapterCommand::List => handle_list(),
        AdapterCommand::Connect { adapter, target } => handle_connect(&adapter, target.as_deref()).await,
        AdapterCommand::Test { adapter } => handle_test(&adapter).await,
        AdapterCommand::Scan { scan_type, timeout } => handle_scan(&scan_type, timeout).await,
    }
}

fn handle_list() -> Result<(), Box<dyn std::error::Error>> {
    let adapters = list_adapter_types();

    println!("Available CAN Adapter Types");
    println!("======================================================");

    for adapter in adapters {
        println!("\n  Type: {}", adapter.adapter_type);
        println!("  Name: {}", adapter.name);
        println!("  Description: {}", adapter.description);
    }

    println!("\nUsage:");
    println!("  canary adapter connect vcan0              # Virtual CAN (testing)");
    println!("  canary adapter connect can0               # SocketCAN (USB)");
    println!("  canary adapter connect wifi -t 192.168.4.1  # WiFi adapter");
    println!("  canary adapter connect bluetooth -t OBDLink  # Bluetooth");

    Ok(())
}

async fn handle_connect(
    adapter_name: &str,
    target: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (adapter_type, connection) = parse_adapter_spec(adapter_name, target)?;

    println!("Connecting to {} adapter '{}'...", adapter_type, connection);

    let mut adapter = create_adapter(adapter_type, &connection);

    match adapter.connect().await {
        Ok(_) => {
            println!("Connected successfully to {}", connection);
            println!("  Adapter type: {}", adapter.adapter_type());
            println!("  Interface: {}", adapter.adapter_name());
            println!("  Status: Connected");

            // Test connection
            match adapter.test_connection().await {
                Ok(true) => println!("  Ping test: PASSED"),
                Ok(false) => println!("  Ping test: FAILED (adapter connected but no ECU response)"),
                Err(e) => println!("  Ping test: ERROR ({})", e),
            }

            adapter.disconnect().await?;
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("\nTroubleshooting:");
            match adapter_type {
                AdapterType::SocketCan | AdapterType::Virtual => {
                    println!("  1. Ensure the interface exists: ip link show {}", connection);
                    println!("  2. Create virtual CAN: sudo ip link add dev {} type vcan", connection);
                    println!("  3. Bring it up: sudo ip link set up {}", connection);
                }
                AdapterType::WiFi => {
                    println!("  1. Ensure the adapter is powered on");
                    println!("  2. Connect to the adapter's WiFi network");
                    println!("  3. Check IP: ping {}", connection);
                }
                AdapterType::Bluetooth => {
                    println!("  1. Ensure Bluetooth is enabled");
                    println!("  2. Put the adapter in pairing mode");
                    println!("  3. Scan: canary adapter scan bluetooth");
                }
            }
        }
    }

    Ok(())
}

async fn handle_test(adapter_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (adapter_type, connection) = parse_adapter_spec(adapter_name, None)?;

    println!("Testing {} adapter '{}'...", adapter_type, connection);

    let mut adapter = create_adapter(adapter_type, &connection);

    match adapter.connect().await {
        Ok(_) => {
            println!("  Connection: OK");
            match adapter.test_connection().await {
                Ok(true) => {
                    println!("  Communication: OK");
                    println!("\n  Result: PASS - Adapter is working correctly");
                }
                Ok(false) => {
                    println!("  Communication: NO RESPONSE");
                    println!("\n  Result: PARTIAL - Connected but no ECU response");
                    println!("  (This is normal if no ECU/simulator is running)");
                }
                Err(e) => {
                    println!("  Communication: ERROR - {}", e);
                    println!("\n  Result: FAIL");
                }
            }
            adapter.disconnect().await?;
        }
        Err(e) => {
            println!("  Connection: FAILED - {}", e);
            println!("\n  Result: FAIL");
        }
    }

    Ok(())
}

async fn handle_scan(scan_type: &str, timeout: u64) -> Result<(), Box<dyn std::error::Error>> {
    match scan_type {
        "bluetooth" => {
            println!("Scanning for Bluetooth OBD adapters ({} seconds)...", timeout);
            let devices = BluetoothAdapter::scan_devices(timeout).await?;

            if devices.is_empty() {
                println!("\nNo Bluetooth OBD adapters found.");
                println!("\nTips:");
                println!("  1. Ensure Bluetooth is enabled on this computer");
                println!("  2. Put the OBD adapter in pairing mode");
                println!("  3. Try a longer scan: canary adapter scan bluetooth -t 10");
            } else {
                println!("\nFound {} device(s):\n", devices.len());
                for device in &devices {
                    let obd_tag = if device.is_obd_adapter { " [OBD]" } else { "" };
                    let rssi = device
                        .rssi
                        .map(|r| format!(" ({}dBm)", r))
                        .unwrap_or_default();
                    println!("  {} - {}{}{}", device.address, device.name, obd_tag, rssi);
                }
                println!("\nConnect with: canary adapter connect bluetooth -t \"<device_name>\"");
            }
        }
        other => {
            return Err(format!("Unknown scan type '{}'. Supported: bluetooth", other).into());
        }
    }

    Ok(())
}

/// Parse adapter name into type and connection string
fn parse_adapter_spec(
    adapter_name: &str,
    target: Option<&str>,
) -> Result<(AdapterType, String), Box<dyn std::error::Error>> {
    match adapter_name {
        "wifi" => {
            let host = target.ok_or("WiFi adapter requires --target <IP:PORT>")?;
            Ok((AdapterType::WiFi, host.to_string()))
        }
        "bluetooth" | "bt" => {
            let device = target.ok_or("Bluetooth adapter requires --target <device_name>")?;
            Ok((AdapterType::Bluetooth, device.to_string()))
        }
        name if name.starts_with("vcan") => Ok((AdapterType::Virtual, name.to_string())),
        name if name.starts_with("can") => Ok((AdapterType::SocketCan, name.to_string())),
        other => Err(format!(
            "Unknown adapter '{}'. Use: vcan0, can0, wifi, bluetooth",
            other
        )
        .into()),
    }
}
