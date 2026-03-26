use canary_core::{
    embedded::DtcSystem, DtcService, PinoutService, ProtocolDecoder, ProtocolFactory,
    ServiceProcedureService,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "canary")]
#[command(about = "Automotive reverse engineering toolkit", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lookup OBD-II pinout information
    Pinout {
        /// Show specific pin number (1-16)
        #[arg(short, long)]
        pin: Option<u8>,
    },

    /// Decode CAN bus frame
    Decode {
        /// Hex bytes to decode (e.g., "00 00 07 E8 03 41 0C")
        #[arg(required = true)]
        bytes: Vec<String>,
    },

    /// Lookup diagnostic trouble code
    Dtc {
        /// DTC code to lookup (e.g., P0301)
        code: String,

        /// Search by keyword instead
        #[arg(short, long)]
        search: bool,
    },

    /// Show service procedure
    Service {
        /// Procedure ID (oil_change, brake_bleeding)
        id: String,

        /// Show detailed steps
        #[arg(short, long)]
        verbose: bool,
    },

    /// List available data
    List {
        /// What to list (pinouts, protocols, dtc, procedures)
        what: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize without database (embedded data only)
    canary_core::initialize(None).await?;

    match cli.command {
        Commands::Pinout { pin } => handle_pinout(pin)?,
        Commands::Decode { bytes } => handle_decode(bytes)?,
        Commands::Dtc { code, search } => handle_dtc(&code, search)?,
        Commands::Service { id, verbose } => handle_service(&id, verbose)?,
        Commands::List { what } => handle_list(&what)?,
    }

    Ok(())
}

fn handle_pinout(pin: Option<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let obd2 = PinoutService::get_obd2_pinout()?;

    println!("OBD-II J1962 16-Pin Connector");
    println!("══════════════════════════════════════════════════════");

    if let Some(pin_num) = pin {
        // Show specific pin
        if pin_num < 1 || pin_num > 16 {
            return Err("Pin number must be between 1 and 16".into());
        }

        let pin = &obd2.pins[(pin_num - 1) as usize];
        println!("\nPin {}: {}", pin.pin_number, pin.signal_name);

        if let Some(voltage) = pin.voltage {
            println!("  Voltage: {}V", voltage);
        }
        if let Some(protocol) = &pin.protocol {
            println!("  Protocol: {}", protocol);
        }
        if let Some(notes) = &pin.notes {
            println!("  Notes: {}", notes);
        }
    } else {
        // Show all pins
        println!("\n{:<4} {:<30} {:<10} {:<15}", "Pin", "Signal", "Voltage", "Protocol");
        println!("{}", "─".repeat(65));

        for pin in &obd2.pins {
            let voltage = pin
                .voltage
                .map(|v| format!("{}V", v))
                .unwrap_or_else(|| "-".to_string());

            let protocol = pin
                .protocol
                .as_ref()
                .map(|p| p.as_str())
                .unwrap_or("-");

            println!(
                "{:<4} {:<30} {:<10} {:<15}",
                pin.pin_number, pin.signal_name, voltage, protocol
            );
        }
    }

    Ok(())
}

fn handle_decode(bytes: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Parse hex bytes
    let raw_bytes: Result<Vec<u8>, _> = bytes
        .iter()
        .map(|s| u8::from_str_radix(s, 16))
        .collect();

    let raw_bytes = raw_bytes.map_err(|e| format!("Invalid hex byte: {}", e))?;

    println!("CAN Bus Frame Decoder");
    println!("══════════════════════════════════════════════════════");

    let decoder = ProtocolFactory::create_can_decoder()?;
    let frame = decoder.decode(&raw_bytes)?;

    println!("\nDecoded Frame:");
    println!("  CAN ID: 0x{:04X}", frame.id);
    println!("  Data Length: {} bytes", frame.data.len());
    println!("  Data: {:02X?}", frame.data);
    println!("  Timestamp: {}", frame.timestamp);

    Ok(())
}

fn handle_dtc(code: &str, search: bool) -> Result<(), Box<dyn std::error::Error>> {
    if search {
        // Search by keyword
        println!("DTC Search: '{}'", code);
        println!("══════════════════════════════════════════════════════");

        let results = DtcService::search_by_description(code);

        if results.is_empty() {
            println!("\nNo DTCs found matching '{}'", code);
        } else {
            println!("\nFound {} code(s):\n", results.len());

            for dtc in results {
                println!("  {} ({:?})", dtc.code, dtc.system);
                println!("    {}", dtc.description);
                println!();
            }
        }
    } else {
        // Lookup specific code
        let dtc = DtcService::lookup_code(code)?;

        println!("DTC Lookup: {}", code);
        println!("══════════════════════════════════════════════════════");
        println!("\nCode: {}", dtc.code);
        println!("System: {:?}", dtc.system);
        println!("Description: {}", dtc.description);

        // Parse system info
        let system_desc = match dtc.system {
            DtcSystem::Powertrain => "Powertrain (Engine/Transmission)",
            DtcSystem::Body => "Body (Doors/Windows/Seats)",
            DtcSystem::Chassis => "Chassis (Brakes/Steering/Suspension)",
            DtcSystem::Network => "Network (CAN bus/Communication)",
        };

        println!("\nSystem Info: {}", system_desc);
    }

    Ok(())
}

fn handle_service(id: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let procedure = ServiceProcedureService::get_procedure(id)?;

    println!("Service Procedure: {}", procedure.name);
    println!("══════════════════════════════════════════════════════");
    println!("\nCategory: {:?}", procedure.category);
    println!("Description: {}", procedure.description);

    if let Some(time) = procedure.estimated_time_minutes {
        println!("Estimated Time: {} minutes", time);
    }

    println!("\nRequired Tools:");
    for tool in &procedure.tools_required {
        println!("  • {}", tool);
    }

    if verbose {
        println!("\nDetailed Steps:");
        for step in &procedure.steps {
            println!("\n{}. {}", step.order, step.instruction);

            if !step.warnings.is_empty() {
                for warning in &step.warnings {
                    println!("   ⚠️  WARNING: {}", warning);
                }
            }
        }
    } else {
        println!("\nSteps: {} total (use --verbose for details)", procedure.steps.len());
    }

    Ok(())
}

fn handle_list(what: &str) -> Result<(), Box<dyn std::error::Error>> {
    match what.to_lowercase().as_str() {
        "pinouts" => {
            let pinouts = PinoutService::list_all();
            println!("Available Pinouts: {}", pinouts.len());
            println!("══════════════════════════════════════════════════════");
            for pinout in pinouts {
                println!("\n  ID: {}", pinout.id);
                println!("  Type: {}", pinout.connector_type);
                println!("  Pins: {}", pinout.pins.len());
            }
        }

        "protocols" => {
            let protocols = ProtocolFactory::list_available_protocols();
            println!("Available Protocols: {}", protocols.len());
            println!("══════════════════════════════════════════════════════");
            for protocol in protocols {
                println!("  • {}", protocol);
            }
        }

        "dtc" => {
            let all_codes = DtcService::list_all();
            println!("Available DTC Codes: {}", all_codes.len());
            println!("══════════════════════════════════════════════════════");

            // Group by system
            let powertrain = DtcService::get_by_system(DtcSystem::Powertrain);

            println!("\nPowertrain (P-codes): {}", powertrain.len());
            for dtc in powertrain {
                println!("  {} - {}", dtc.code, dtc.description);
            }
        }

        "procedures" => {
            let procedures = ServiceProcedureService::list_all();
            println!("Available Service Procedures: {}", procedures.len());
            println!("══════════════════════════════════════════════════════");
            for proc in procedures {
                let time = proc
                    .estimated_time_minutes
                    .map(|t| format!("{} min", t))
                    .unwrap_or_else(|| "varies".to_string());

                println!("\n  ID: {}", proc.id);
                println!("  Name: {}", proc.name);
                println!("  Category: {:?}", proc.category);
                println!("  Time: {}", time);
            }
        }

        _ => {
            return Err(format!(
                "Unknown item '{}'. Try: pinouts, protocols, dtc, procedures",
                what
            )
            .into());
        }
    }

    Ok(())
}
