use canary_hardware::{create_adapter, AdapterType, CanAdapter, VirtualAdapter};
use canary_uds::{SessionType, UdsSession};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct LiveArgs {
    #[command(subcommand)]
    pub command: LiveCommand,
}

#[derive(Subcommand)]
pub enum LiveCommand {
    /// Read DTCs from an ECU
    Dtc {
        /// ECU CAN ID (hex, e.g., 0x7E0)
        #[arg(long, default_value = "0x7E0")]
        ecu: String,

        /// Adapter to use (e.g., vcan0)
        #[arg(long, default_value = "vcan0")]
        adapter: String,

        /// Only show active DTCs
        #[arg(long)]
        active_only: bool,
    },

    /// Monitor live PIDs from an ECU
    Monitor {
        /// PIDs to monitor (hex, comma-separated, e.g., 0x0C,0x0D)
        #[arg(long)]
        pid: String,

        /// ECU CAN ID
        #[arg(long, default_value = "0x7E0")]
        ecu: String,

        /// Adapter to use
        #[arg(long, default_value = "vcan0")]
        adapter: String,

        /// Update interval in milliseconds
        #[arg(long, default_value = "500")]
        interval: u64,
    },

    /// Start a diagnostic session
    Session {
        /// Session type: default, extended, programming
        #[arg(long, default_value = "extended")]
        session_type: String,

        /// ECU CAN ID
        #[arg(long, default_value = "0x7E0")]
        ecu: String,

        /// Adapter to use
        #[arg(long, default_value = "vcan0")]
        adapter: String,
    },
}

pub async fn handle_live(args: LiveArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        LiveCommand::Dtc {
            ecu,
            adapter,
            active_only,
        } => handle_dtc(&ecu, &adapter, active_only).await,
        LiveCommand::Monitor {
            pid,
            ecu,
            adapter,
            interval,
        } => handle_monitor(&pid, &ecu, &adapter, interval).await,
        LiveCommand::Session {
            session_type,
            ecu,
            adapter,
        } => handle_session(&session_type, &ecu, &adapter).await,
    }
}

fn parse_hex_id(s: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(s, 16).map_err(|e| format!("Invalid hex ID '{}': {}", s, e).into())
}

async fn create_session(
    adapter_name: &str,
    ecu_id: u32,
) -> Result<UdsSession, Box<dyn std::error::Error>> {
    let adapter: Box<dyn CanAdapter> = if adapter_name.starts_with("vcan") {
        let mut a = VirtualAdapter::new(adapter_name);
        a.connect().await?;
        Box::new(a)
    } else if adapter_name.starts_with("can") {
        let mut a = create_adapter(AdapterType::SocketCan, adapter_name);
        a.connect().await?;
        a
    } else {
        return Err(format!("Unsupported adapter '{}' for live diagnostics", adapter_name).into());
    };

    Ok(UdsSession::new(adapter, ecu_id))
}

async fn handle_dtc(
    ecu: &str,
    adapter: &str,
    active_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let ecu_id = parse_hex_id(ecu)?;

    println!(
        "Reading DTCs from ECU 0x{:03X} via {}...",
        ecu_id, adapter
    );
    println!("======================================================");

    let session = create_session(adapter, ecu_id).await?;

    let result = if active_only {
        session.read_active_dtcs().await
    } else {
        session.read_dtcs().await
    };

    match result {
        Ok(response) => {
            if response.dtcs.is_empty() {
                println!("\nNo DTCs found. System is clean.");
            } else {
                println!("\nFound {} DTC(s):\n", response.dtcs.len());
                for dtc in &response.dtcs {
                    let code = dtc.to_code_string();
                    let status = dtc.status.description();
                    println!("  {} - Status: {}", code, status);
                    println!(
                        "    Raw: [{:02X} {:02X} {:02X}] Status: 0x{:02X}",
                        dtc.dtc_high, dtc.dtc_mid, dtc.dtc_low, dtc.status.raw
                    );
                }
            }
        }
        Err(e) => {
            println!("\nFailed to read DTCs: {}", e);
            println!("\nTroubleshooting:");
            println!("  1. Ensure the ECU simulator is running");
            println!("  2. Check the CAN adapter connection");
            println!("  3. Verify the ECU ID (try 0x7E0 for standard OBD)");
        }
    }

    Ok(())
}

async fn handle_monitor(
    pids: &str,
    ecu: &str,
    adapter: &str,
    interval: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let ecu_id = parse_hex_id(ecu)?;

    let pid_list: Result<Vec<u16>, _> = pids
        .split(',')
        .map(|p| {
            let p = p.trim().trim_start_matches("0x").trim_start_matches("0X");
            u16::from_str_radix(p, 16)
        })
        .collect();

    let pid_list = pid_list.map_err(|e| format!("Invalid PID: {}", e))?;

    println!(
        "Monitoring PIDs from ECU 0x{:03X} via {} ({}ms interval)",
        ecu_id, adapter, interval
    );
    println!("======================================================");
    println!("PIDs: {}", pids);
    println!("Press Ctrl+C to stop\n");

    let session = create_session(adapter, ecu_id).await?;

    // Single read for demonstration (continuous monitoring would loop)
    let results = session.read_multiple_dids(&pid_list.iter().map(|p| *p as u16).collect::<Vec<_>>()).await?;

    for result in &results {
        println!(
            "  DID 0x{:04X}: {} ({})",
            result.did,
            result.as_hex_string(),
            result
                .as_string()
                .unwrap_or_else(|| format!("{} bytes", result.data.len()))
        );
    }

    if results.is_empty() {
        println!("  No data received. ECU may not support these PIDs.");
    }

    Ok(())
}

async fn handle_session(
    session_type: &str,
    ecu: &str,
    adapter: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ecu_id = parse_hex_id(ecu)?;

    let st = match session_type.to_lowercase().as_str() {
        "default" => SessionType::Default,
        "extended" => SessionType::Extended,
        "programming" => SessionType::Programming,
        other => {
            return Err(
                format!("Unknown session type '{}'. Use: default, extended, programming", other)
                    .into(),
            )
        }
    };

    println!(
        "Starting {} diagnostic session with ECU 0x{:03X} via {}...",
        st.name(),
        ecu_id,
        adapter
    );
    println!("======================================================");

    let mut session = create_session(adapter, ecu_id).await?;

    match session.start_session(st).await {
        Ok(response) => {
            println!("\nSession established!");
            println!("  Type: {} (0x{:02X})", response.session_type.name(), response.session_type as u8);
            println!("  P2 max: {}ms", response.p2_server_max_ms);
            println!("  P2* max: {}ms", response.p2_star_server_max_ms);
        }
        Err(e) => {
            println!("\nFailed to start session: {}", e);
        }
    }

    Ok(())
}
