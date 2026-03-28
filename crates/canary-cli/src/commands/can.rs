use canary_hardware::{create_adapter, AdapterType, CanAdapter, CanFrame, VirtualAdapter};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct CanArgs {
    #[command(subcommand)]
    pub command: CanCommand,
}

#[derive(Subcommand)]
pub enum CanCommand {
    /// Send a raw CAN frame
    Send {
        /// CAN ID (hex, e.g., 0x7E0)
        id: String,

        /// Data bytes (hex, space-separated, e.g., "02 10 01")
        #[arg(long)]
        data: String,

        /// Adapter to use
        #[arg(long, default_value = "vcan0")]
        adapter: String,
    },

    /// Listen for CAN frames
    Listen {
        /// Filter by CAN ID (hex, e.g., 0x7E8)
        #[arg(long)]
        filter: Option<String>,

        /// Adapter to use
        #[arg(long, default_value = "vcan0")]
        adapter: String,

        /// Timeout in milliseconds (0 = indefinite)
        #[arg(long, default_value = "5000")]
        timeout: u64,

        /// Maximum number of frames to capture
        #[arg(long, default_value = "10")]
        count: usize,
    },
}

pub async fn handle_can(args: CanArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        CanCommand::Send { id, data, adapter } => handle_send(&id, &data, &adapter).await,
        CanCommand::Listen {
            filter,
            adapter,
            timeout,
            count,
        } => handle_listen(filter.as_deref(), &adapter, timeout, count).await,
    }
}

fn parse_hex_id(s: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(s, 16).map_err(|e| format!("Invalid hex ID '{}': {}", s, e).into())
}

fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    s.split_whitespace()
        .map(|b| {
            let b = b.trim_start_matches("0x").trim_start_matches("0X");
            u8::from_str_radix(b, 16).map_err(|e| format!("Invalid hex byte '{}': {}", b, e).into())
        })
        .collect()
}

async fn connect_adapter(
    adapter_name: &str,
) -> Result<Box<dyn CanAdapter>, Box<dyn std::error::Error>> {
    if adapter_name.starts_with("vcan") {
        let mut a = VirtualAdapter::new(adapter_name);
        a.connect().await?;
        Ok(Box::new(a))
    } else if adapter_name.starts_with("can") {
        let mut a = create_adapter(AdapterType::SocketCan, adapter_name);
        a.connect().await?;
        Ok(a)
    } else {
        Err(format!(
            "Unsupported adapter '{}' for raw CAN. Use vcan0 or can0.",
            adapter_name
        )
        .into())
    }
}

async fn handle_send(
    id: &str,
    data: &str,
    adapter_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let can_id = parse_hex_id(id)?;
    let data_bytes = parse_hex_bytes(data)?;

    if data_bytes.len() > 8 {
        return Err("CAN frame data cannot exceed 8 bytes".into());
    }

    println!("Sending CAN frame via {}...", adapter_name);
    println!("======================================================");

    let adapter = connect_adapter(adapter_name).await?;

    let frame = CanFrame::new(can_id, data_bytes.clone());
    adapter.send_frame(&frame).await?;

    let data_hex: Vec<String> = data_bytes.iter().map(|b| format!("{:02X}", b)).collect();
    println!(
        "\n  Sent: ID=0x{:03X} Data=[{}] ({} bytes)",
        can_id,
        data_hex.join(" "),
        data_bytes.len()
    );

    Ok(())
}

async fn handle_listen(
    filter: Option<&str>,
    adapter_name: &str,
    timeout: u64,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let filter_id = filter.map(|f| parse_hex_id(f)).transpose()?;

    println!("Listening for CAN frames on {}...", adapter_name);
    if let Some(fid) = filter_id {
        println!("  Filter: 0x{:03X}", fid);
    }
    println!("  Timeout: {}ms", timeout);
    println!("  Max frames: {}", count);
    println!("======================================================");
    println!(
        "\n{:<12} {:<10} {:<5} {}",
        "Timestamp", "ID", "Len", "Data"
    );
    println!("{}", "-".repeat(60));

    let adapter = connect_adapter(adapter_name).await?;

    let mut received = 0;
    let start = std::time::Instant::now();

    while received < count {
        let remaining = if timeout > 0 {
            let elapsed = start.elapsed().as_millis() as u64;
            if elapsed >= timeout {
                break;
            }
            timeout - elapsed
        } else {
            5000 // Default chunk timeout
        };

        match adapter.recv_frame(remaining.min(1000)).await {
            Ok(frame) => {
                if let Some(fid) = filter_id {
                    if frame.id != fid {
                        continue;
                    }
                }

                let elapsed_ms = start.elapsed().as_millis();
                let data_hex: Vec<String> =
                    frame.data.iter().map(|b| format!("{:02X}", b)).collect();
                println!(
                    "{:<12} 0x{:03X}     {:<5} [{}]",
                    format!("{}ms", elapsed_ms),
                    frame.id,
                    frame.data.len(),
                    data_hex.join(" ")
                );

                received += 1;
            }
            Err(canary_hardware::CanError::Timeout(_)) => {
                continue;
            }
            Err(e) => {
                println!("\nError receiving frame: {}", e);
                break;
            }
        }
    }

    println!("\n  Received {} frame(s)", received);

    Ok(())
}
