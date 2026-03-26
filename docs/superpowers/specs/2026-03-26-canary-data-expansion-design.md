# Canary Data Expansion - Comprehensive ECU/Pinout Database

**Date:** 2026-03-26
**Author:** Claude Sonnet 4.5
**Status:** Approved
**Version:** 1.0

---

## Executive Summary

Expand the Canary automotive reverse engineering library with comprehensive ECU (Electronic Control Unit) and connector pinout data covering all major automotive control modules and global manufacturers. This design implements a three-phase approach: Foundation + MVP (manual curation of 10-15 critical ECUs), Automation (web scrapers and PDF parsers), and Community Scale (crowdsourced contributions and verification).

**Key Decisions:**
- **Module Coverage:** All automotive control modules (ECM, TCM, BCM, ABS, airbag, cluster, HVAC, gateway, etc.)
- **Manufacturer Scope:** Global comprehensive coverage (VW, GM, Ford, Toyota, BMW, Honda, Nissan, Mercedes, etc.)
- **Data Collection:** Hybrid approach - automated scraping + PDF parsing + community contributions
- **Storage Strategy:** Tiered with lazy loading - universal standards embedded, manufacturer data compressed and loaded on-demand
- **Implementation:** MVP + parallel automation tracks, then community-driven scale

**Success Metrics:**
- Phase 1: 10-15 ECUs with 100% accuracy, < 15MB binary size
- Phase 2: 500+ ECUs from automated collection, 80%+ parser accuracy
- Phase 3: 800-1000 total ECUs, community contribution system live

---

## 1. Architecture Overview

### 1.1 Tiered Storage Strategy

The architecture uses three storage layers optimized for different data access patterns:

#### **Layer 1: Embedded (Compile-Time)**

**Purpose:** Universal automotive standards that apply across all manufacturers.

**Contents:**
- OBD-II J1962 standard pinout
- SAE J1939 (heavy-duty CAN bus)
- ISO standards (ISO 15765-4 CAN, ISO 9141-2 K-Line)
- Protocol specifications (CAN 2.0B, K-Line, LIN 2.0, J1850)
- Critical safety DTCs (airbag, ABS fault codes)

**Implementation:**
- TOML files compiled into binary via `include_str!`
- Parsed at compile-time into `static Lazy<HashMap>`
- Always available, zero runtime I/O
- Total size: ~5-10MB embedded data

**Benefits:**
- Offline-first operation
- Zero latency for universal standards
- No decompression overhead
- Works without database or internet

#### **Layer 2: Compressed Data (Lazy-Loaded)**

**Purpose:** Manufacturer-specific ECU pinouts loaded on first access per manufacturer.

**Contents:**
- ECU pinouts organized by manufacturer/brand
- Model-year specific variations
- Connector specifications by OEM
- Module-specific DTCs

**Storage Format:**
- `.toml.gz` compressed files (gzip level 9)
- Organized by manufacturer: `data/manufacturers/vw/ecm.toml.gz`
- Metadata index: `data/manufacturers/index.json`
- Typical size: 100-500KB compressed, 1-5MB decompressed per manufacturer

**Lazy Loading Mechanism:**

```rust
static VW_ECUS: Lazy<HashMap<String, EcuPinout>> = Lazy::new(|| {
    // Load on first VW ECU request
    let compressed = include_bytes!("../data/manufacturers/vw/ecm.toml.gz");
    let decompressed = decompress(compressed).expect("Failed to decompress VW data");
    let ecus: Vec<EcuPinout> = toml::from_str(&decompressed).expect("Failed to parse VW ECUs");
    ecus.into_iter().map(|e| (e.id.clone(), e)).collect()
});

pub fn get_vw_ecu(id: &str) -> Option<&'static EcuPinout> {
    VW_ECUS.get(id)  // First call triggers decompression, subsequent calls hit cache
}
```

**Performance Characteristics:**
- First access: 50-100ms (decompression + parsing)
- Cached access: < 1ms (HashMap lookup)
- Memory: Only loaded manufacturers stay in memory
- Load time scales with manufacturer data size, not total database size

**Benefits:**
- Small binary size (compressed data)
- Fast startup (no decompression until needed)
- Memory efficient (only active manufacturers loaded)
- Easy updates (replace .gz files without recompilation)
- Offline operation (all data ships with library)

#### **Layer 3: Database (Optional, User Data)**

**Purpose:** User-contributed custom data and personal annotations.

**Contents:**
- User-added custom ECU pinouts
- Personal notes on existing ECUs
- Service history logs
- Repair records and modifications

**Implementation:**
- PostgreSQL or SQLite via SeaORM
- Schema: `custom_ecus`, `ecu_notes`, `service_logs` tables
- Optional - library works without database
- Follows existing Canary database patterns

**Benefits:**
- Dynamic user content without recompilation
- Personal data separate from library data
- Database migrations handle schema evolution

### 1.2 Directory Structure

```
canary/
├── crates/
│   ├── canary-data/
│   │   ├── build.rs                  # Compression + validation
│   │   ├── src/
│   │   │   ├── lib.rs                # Embedded + lazy loaders
│   │   │   └── lazy.rs               # Decompression logic
│   │   └── data/
│   │       ├── universal/            # Embedded (compiled in)
│   │       │   ├── obd2_j1962.toml
│   │       │   ├── j1939.toml
│   │       │   └── iso_standards.toml
│   │       ├── protocols/            # Embedded
│   │       │   ├── can_2.0b.toml
│   │       │   ├── kwp2000.toml
│   │       │   └── lin_2.0.toml
│   │       ├── manufacturers/        # Compressed (lazy-loaded)
│   │       │   ├── index.json        # Manufacturer metadata
│   │       │   ├── vw/
│   │       │   │   ├── ecm.toml.gz
│   │       │   │   ├── tcm.toml.gz
│   │       │   │   ├── bcm.toml.gz
│   │       │   │   └── metadata.json
│   │       │   ├── gm/
│   │       │   ├── ford/
│   │       │   ├── toyota/
│   │       │   ├── bmw/
│   │       │   ├── honda/
│   │       │   └── ...
│   │       └── connectors/           # Embedded connector library
│   │           ├── bosch_ev1_4.toml
│   │           ├── delphi_gt150.toml
│   │           └── molex_mx150.toml
│   │
│   ├── canary-scraper/               # NEW: Data collection tools
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── scrapers/
│   │   │   │   ├── ecu_design.rs
│   │   │   │   └── xtuning.rs
│   │   │   ├── parsers/
│   │   │   │   ├── pdf.rs
│   │   │   │   └── table_extraction.rs
│   │   │   ├── validators/
│   │   │   │   ├── schema.rs
│   │   │   │   └── conflict_resolver.rs
│   │   │   └── bin/
│   │   │       └── canary_scraper.rs # CLI tool
│   │   └── tests/
│   │       └── integration.rs
│   │
│   └── canary-models/
│       └── src/
│           └── embedded/
│               ├── ecu.rs            # NEW: ECU data models
│               ├── connector.rs      # NEW: Connector specs
│               └── ...
```

### 1.3 Build Process

**Compile-Time Operations (`build.rs`):**

```rust
fn main() {
    println!("cargo:rerun-if-changed=data/");

    // 1. Validate all TOML files
    validate_universal_data();
    validate_manufacturer_data();
    validate_connectors();

    // 2. Compress manufacturer data
    compress_manufacturer_files();

    // 3. Generate metadata index
    generate_manufacturer_index();

    // 4. Verify compressed data integrity
    verify_compressed_files();
}

fn compress_manufacturer_files() {
    let manufacturers = ["vw", "gm", "ford", "toyota", "bmw", /* ... */];

    for mfr in manufacturers {
        let input_dir = format!("data/manufacturers/{}/", mfr);

        for entry in glob(&format!("{}*.toml", input_dir))? {
            let toml_path = entry?;
            let gz_path = toml_path.with_extension("toml.gz");

            let toml_content = std::fs::read_to_string(&toml_path)?;
            let compressed = compress_gzip(&toml_content)?;

            std::fs::write(&gz_path, compressed)?;

            // Verify compression ratio
            let original_size = toml_content.len();
            let compressed_size = compressed.len();
            let ratio = compressed_size as f32 / original_size as f32;

            println!("Compressed {}: {:.1}% of original",
                     toml_path.display(), ratio * 100.0);
        }
    }
}
```

**Expected Compression Ratios:**
- TOML files: 70-85% size reduction (text compresses well)
- Example: 5MB TOML → ~1MB .gz
- Total manufacturer data: ~500MB uncompressed → ~100MB compressed

---

## 2. Extended Data Models

### 2.1 Core ECU Types

#### **ModuleType Enum**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleType {
    // Powertrain
    ECM,         // Engine Control Module
    PCM,         // Powertrain Control Module (combined engine+trans)
    TCM,         // Transmission Control Module

    // Body & Comfort
    BCM,         // Body Control Module
    DDM,         // Driver Door Module
    PDM,         // Passenger Door Module
    HVAC,        // Climate Control Module

    // Safety Systems
    ABS,         // Anti-lock Braking System
    SRS,         // Airbag/Supplemental Restraint System
    EPB,         // Electronic Parking Brake

    // Instrumentation
    IPC,         // Instrument Panel Cluster
    InfoCenter,  // Information/Entertainment Center

    // Network & Communication
    Gateway,     // CAN Gateway / Network Controller
    Telematics,  // Connected Services / OnStar / mbrace

    // Diagnostics
    OBD,         // OBD-II Diagnostic Port
}
```

**Design Rationale:**
- Comprehensive coverage of all major automotive modules
- Clear categorization by system (powertrain, body, safety, etc.)
- Extensible for future module types
- Used for filtering and organization

#### **EcuPinout Structure**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcuPinout {
    /// Unique identifier: "vw_golf_mk7_2015_ecm_bosch_med17_5"
    pub id: String,

    /// Type of module (ECM, TCM, BCM, etc.)
    pub module_type: ModuleType,

    /// Vehicle manufacturer: "vw", "gm", "ford"
    pub manufacturer_id: String,

    /// ECU manufacturer: "bosch", "continental", "denso"
    pub ecu_manufacturer: String,

    /// OEM part numbers for identification
    pub part_numbers: Vec<String>,  // ["06J 906 026 CQ", "0 261 S09 098"]

    /// Compatible vehicles
    pub vehicle_models: Vec<VehicleModel>,

    /// Connector specifications (often multiple connectors per ECU)
    pub connectors: Vec<ConnectorSpec>,

    /// Power supply requirements
    pub power_requirements: PowerSpec,

    /// Supported communication protocols
    pub supported_protocols: Vec<String>,  // ["can_2.0b", "kwp2000"]

    /// Flash memory specifications (for tuning/programming)
    pub flash_memory: Option<MemorySpec>,

    /// Data source and quality metadata
    #[serde(default)]
    pub metadata: DataMetadata,
}
```

**Key Features:**
- **Unique ID:** Hierarchical naming: `{manufacturer}_{model}_{year}_{module}_{ecu_mfr}_{version}`
- **Part Numbers:** Multiple variants (OEM numbers, ECU manufacturer codes)
- **Multi-Connector:** ECUs often have 2-4 connectors (e.g., 121-pin + 60-pin)
- **Extensible:** Optional fields allow incomplete data

#### **ConnectorSpec Structure**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorSpec {
    /// Connector identifier within ECU: "T121", "C1", "X60"
    pub connector_id: String,

    /// Connector type/series: "121-pin Bosch EV1.4", "60-pin Delphi GT150"
    pub connector_type: String,

    /// Physical color for identification
    pub color: Option<String>,  // "Black", "Grey", "Green"

    /// Pin mappings (all pins in this connector)
    pub pins: Vec<PinMapping>,

    /// Reference to standard connector definition (if available)
    pub standard_connector_id: Option<String>,  // "bosch_ev1_4_121pin"
}
```

**Design Rationale:**
- **Multiple Connectors:** Modern ECUs have 2-5 connectors
- **Color Coding:** Physical identification aid
- **Standard Reference:** Links to reusable connector library
- **Flexible:** Works with custom or standard connectors

#### **PinMapping Structure**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinMapping {
    /// Pin number (1-255)
    pub pin_number: u8,

    /// Signal name: "Injector Cylinder 1", "CAN High", "Battery Positive"
    pub signal_name: String,

    /// Wire color code: "BK/WT" (Black/White stripe)
    pub wire_color: Option<String>,

    /// Nominal voltage
    pub voltage: Option<f32>,

    /// Maximum current (amperes)
    pub current_max: Option<f32>,

    /// Signal classification
    pub signal_type: SignalType,

    /// Communication protocol (if applicable)
    pub protocol: Option<String>,

    /// Additional notes
    pub notes: Option<String>,

    /// Connected sensor/actuator
    pub sensor_actuator: Option<String>,  // "O2 Sensor Bank 1 Sensor 1"
}
```

**Enhanced Fields:**
- **Wire Color:** Industry standard codes (BK=Black, WT=White, RD=Red, etc.)
- **Current Rating:** Critical for power/ground pins
- **Signal Type:** Enables filtering and validation
- **Sensor/Actuator:** Traces signal to component

#### **SignalType Enum**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    Power,           // Battery voltage, switched power
    Ground,          // Chassis ground, signal ground
    Digital,         // Digital I/O (on/off)
    Analog,          // Analog voltage/current
    PWM,             // Pulse-width modulated
    CAN,             // CAN bus (High/Low)
    LIN,             // LIN bus
    FlexRay,         // FlexRay (high-speed networking)
    SensorInput,     // Sensor voltage input
    ActuatorOutput,  // Actuator control output
}
```

**Usage:**
- Enables semantic search ("show all CAN pins")
- Validation (power pins should have voltage specified)
- Visualization (color-code by type)

### 2.2 Power and Memory Specifications

#### **PowerSpec Structure**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerSpec {
    /// Minimum operating voltage (typically 9.0V)
    pub voltage_min: f32,

    /// Nominal voltage (12.0V or 24.0V for heavy-duty)
    pub voltage_nominal: f32,

    /// Maximum operating voltage (typically 16.0V)
    pub voltage_max: f32,

    /// Idle current draw (amperes)
    pub current_idle: Option<f32>,

    /// Peak current draw (amperes)
    pub current_max: Option<f32>,
}
```

**Typical Values:**
- Light-duty vehicles: 9-16V, 12V nominal
- Heavy-duty trucks: 18-32V, 24V nominal
- Hybrid systems: May have multiple voltage rails

#### **MemorySpec Structure**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpec {
    /// Flash memory size (KB)
    pub flash_size_kb: u32,

    /// RAM size (KB)
    pub ram_size_kb: u32,

    /// EEPROM size (KB), if present
    pub eeprom_size_kb: Option<u32>,

    /// CPU/MCU model
    pub cpu: String,  // "Infineon TriCore TC1797", "NXP MPC5xxx"
}
```

**Purpose:**
- Tuning reference (flash memory size for calibration)
- Programming tool compatibility
- Performance expectations

### 2.3 Connector Library

#### **ConnectorDefinition Structure**

Reusable connector specifications shared across multiple ECUs.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorDefinition {
    /// Unique identifier: "bosch_ev1_4_121pin"
    pub id: String,

    /// Manufacturer: "Bosch", "Delphi", "Molex"
    pub manufacturer: String,

    /// Product series: "EV1.4", "GT150", "MX150"
    pub series: String,

    /// Number of pins
    pub pin_count: u8,

    /// Male/Female/Hermaphroditic
    pub gender: ConnectorGender,

    /// Keying variant (prevents wrong mating)
    pub keying: Option<String>,  // "A", "B", "C"

    /// Seal type
    pub seal_type: Option<String>,  // "Weather Pack", "Environmental"

    /// Current rating per pin (amperes)
    pub current_rating: Option<f32>,

    /// Voltage rating (volts)
    pub voltage_rating: Option<f32>,

    /// Operating temperature range (°C)
    pub temperature_range: Option<(i16, i16)>,  // (-40, 125)

    /// Mounting style
    pub mounting: MountingType,

    /// Industry certifications
    pub certifications: Vec<String>,  // ["USCAR-2", "LV214"]
}
```

**Benefits:**
- **Reusability:** One definition used by many ECUs
- **Standardization:** Industry-standard connectors cataloged
- **Validation:** Ensures pin count matches connector spec
- **Reference:** Links to manufacturer datasheets

#### **Enums**

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConnectorGender {
    Male,            // Pins protruding
    Female,          // Receptacle
    Hermaphroditic,  // Self-mating (same on both sides)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MountingType {
    PCB,       // Surface or through-hole PCB mount
    Bulkhead,  // Panel-mount with nut
    Cable,     // Wire-to-wire inline
    Panel,     // Snap-in panel mount
}
```

### 2.4 Enhanced Diagnostic Codes

#### **DiagnosticCode Structure (Extended)**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCode {
    /// DTC code: "P0301", "C1234", "B0015", "U0100"
    pub code: String,

    /// System category (P/B/C/U)
    pub system: DtcSystem,

    /// Specific module that reports this code
    pub module_type: Option<ModuleType>,

    /// Human-readable description
    pub description: String,

    /// Severity/priority level
    pub severity: DtcSeverity,

    /// Possible root causes
    pub causes: Vec<String>,

    /// Observable symptoms
    pub symptoms: Vec<String>,

    /// Diagnostic/troubleshooting steps
    pub diagnostic_steps: Vec<String>,

    /// Related DTCs (often appear together)
    pub related_codes: Vec<String>,
}
```

**Example:**

```toml
[[dtc]]
code = "P0301"
system = "Powertrain"
module_type = "ECM"
description = "Cylinder 1 Misfire Detected"
severity = "Warning"

causes = [
    "Faulty spark plug",
    "Ignition coil failure",
    "Fuel injector clogged or failed",
    "Low compression in cylinder 1",
    "Vacuum leak affecting cylinder 1"
]

symptoms = [
    "Rough idle",
    "Loss of power",
    "Check engine light illuminated",
    "Increased fuel consumption"
]

diagnostic_steps = [
    "Check spark plug condition and gap",
    "Test ignition coil primary/secondary resistance",
    "Perform compression test on cylinder 1",
    "Check fuel injector operation with noid light",
    "Inspect for vacuum leaks"
]

related_codes = ["P0300", "P0171", "P0420"]
```

#### **DtcSeverity Enum**

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DtcSeverity {
    Critical,      // Stop driving immediately (e.g., airbag fault)
    Warning,       // Address soon (e.g., engine misfire)
    Notice,        // Monitor, non-critical (e.g., EVAP small leak)
    Informational, // FYI only (e.g., oil change reminder)
}
```

### 2.5 Data Metadata

#### **DataMetadata Structure**

Tracks data provenance, quality, and community validation.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataMetadata {
    /// Primary data source
    pub source: DataSource,

    /// All sources if merged from multiple
    pub sources: Vec<String>,

    /// When data was collected/scraped
    pub scraped_date: Option<String>,  // ISO 8601

    /// License
    pub license: String,  // "CC-BY-SA-4.0", "OEM Manual", "Community"

    /// GitHub usernames of contributors
    pub contributors: Vec<String>,

    /// Number of community verifications
    pub verified_by: u32,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Last update timestamp
    pub last_updated: String,  // ISO 8601

    /// Links to source documentation
    pub documentation_urls: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DataSource {
    OemManual,       // Official manufacturer service manual
    EcuDesign,       // ecu.design website
    XTuning,         // xtuning.vn website
    Scribd,          // Scribd PDFs
    Community,       // Community contribution
    Manual,          // Manually curated by maintainers
    Merged,          // Combined from multiple sources
}
```

**Confidence Scoring Algorithm:**

```rust
pub fn calculate_confidence(metadata: &DataMetadata) -> f32 {
    let mut score = 0.0;

    // Official OEM source: +0.4
    if metadata.source == DataSource::OemManual {
        score += 0.4;
    }

    // Multiple agreeing sources: +0.3
    if metadata.sources.len() >= 3 {
        score += 0.3;
    } else if metadata.sources.len() == 2 {
        score += 0.15;
    }

    // Community verifications (5+ users): +0.2
    if metadata.verified_by >= 5 {
        score += 0.2;
    } else if metadata.verified_by >= 2 {
        score += 0.1;
    }

    // Complete data (assessed separately): +0.1
    // (Checked by validator)

    score.min(1.0)
}
```

### 2.6 Backward Compatibility

**Existing Types Preserved:**

```rust
// Existing ConnectorPinout still works for simple cases
pub struct ConnectorPinout {
    pub id: String,
    pub connector_type: String,
    pub manufacturer_id: Option<String>,
    pub vehicle_models: Vec<VehicleModel>,
    pub pins: Vec<PinMapping>,
}

// Services auto-detect which type
impl PinoutService {
    pub fn get_by_id(id: &str) -> Result<PinoutType> {
        if let Some(ecu) = try_load_ecu_pinout(id) {
            Ok(PinoutType::Ecu(ecu))
        } else if let Some(connector) = try_load_connector_pinout(id) {
            Ok(PinoutType::Connector(connector))
        } else {
            Err(CanaryError::NotFound(id.to_string()))
        }
    }
}

pub enum PinoutType {
    Ecu(EcuPinout),
    Connector(ConnectorPinout),
}
```

**No Breaking Changes:**
- Existing API calls continue to work
- OBD-II pinout lookup unchanged
- New features are additive

---

## 3. Data Collection Tools

### 3.1 Scraper Architecture

#### **New Workspace Crate: `canary-scraper`**

```toml
# canary-scraper/Cargo.toml
[package]
name = "canary-scraper"
version = "0.1.0"
edition.workspace = true

[[bin]]
name = "canary-scraper"
path = "src/bin/canary_scraper.rs"

[dependencies]
canary-models = { path = "../canary-models" }
reqwest = { version = "0.11", features = ["gzip", "json"] }
scraper = "0.17"  # HTML parsing (CSS selectors)
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
pdf-extract = "0.7"
regex = "1.10"
anyhow = { workspace = true }
thiserror = { workspace = true }
clap = { version = "4.5", features = ["derive"] }
flate2 = "1.0"  # gzip compression
```

#### **Scraper Trait**

```rust
#[async_trait::async_trait]
pub trait EcuScraper {
    /// Scrape ECUs for a specific manufacturer
    async fn scrape_manufacturer(&self, manufacturer: &str) -> Result<Vec<EcuPinout>>;

    /// Scrape all available ECUs
    async fn scrape_all(&self) -> Result<HashMap<String, Vec<EcuPinout>>>;

    /// Get scraper name for logging
    fn name(&self) -> &str;
}
```

### 3.2 Web Scrapers

#### **ECU.Design Scraper**

```rust
pub struct EcuDesignScraper {
    client: reqwest::Client,
    cache_dir: PathBuf,
    rate_limiter: RateLimiter,
}

impl EcuDesignScraper {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("CanaryScraper/0.1.0")
                .gzip(true)
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            cache_dir,
            rate_limiter: RateLimiter::new(Duration::from_secs(2)), // 1 req / 2s
        }
    }
}

#[async_trait::async_trait]
impl EcuScraper for EcuDesignScraper {
    async fn scrape_manufacturer(&self, manufacturer: &str) -> Result<Vec<EcuPinout>> {
        let url = format!("https://ecu.design/pinout-ecu/?manufacturer={}", manufacturer);

        // Check cache first
        if let Some(cached) = self.load_from_cache(&url)? {
            return Ok(cached);
        }

        // Rate limiting
        self.rate_limiter.wait().await;

        // Fetch page
        let html = self.client.get(&url).send().await?.text().await?;

        // Parse HTML
        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse(".ecu-entry").unwrap();

        let mut ecus = Vec::new();

        for element in document.select(&selector) {
            if let Some(ecu) = self.parse_ecu_element(element)? {
                ecus.push(ecu);
            }
        }

        // Cache results
        self.save_to_cache(&url, &ecus)?;

        Ok(ecus)
    }

    fn name(&self) -> &str {
        "ECU.Design"
    }
}
```

**HTML Parsing Strategy:**
- Use CSS selectors to extract structured data
- Robust to minor HTML changes
- Fallback to regex if selectors fail
- Validate all extracted data

#### **XTuning Scraper**

Similar structure to ECU.Design scraper, adapted for xtuning.vn HTML structure.

**Key Features:**
- Different CSS selectors
- Handles Vietnamese language data
- Supports pagination
- Image download for connector diagrams

### 3.3 PDF Parser

#### **Scribus PDF Parser**

```rust
pub struct ScribdPdfParser {
    cache_dir: PathBuf,
}

impl ScribdPdfParser {
    /// Parse pinout table from PDF document
    pub fn parse_pinout_table(&self, pdf_path: &Path) -> Result<Vec<PinMapping>> {
        // Extract text from PDF
        let text = pdf_extract::extract_text(pdf_path)?;

        // Detect table format
        let table_format = self.detect_table_format(&text)?;

        // Extract rows based on format
        let pins = match table_format {
            TableFormat::PinSignalNotes => self.parse_format_psn(&text)?,
            TableFormat::PinFunctionVoltage => self.parse_format_pfv(&text)?,
            TableFormat::PinWireColorSignal => self.parse_format_pwcs(&text)?,
        };

        Ok(pins)
    }

    fn detect_table_format(&self, text: &str) -> Result<TableFormat> {
        // Look for header patterns
        if text.contains("Pin") && text.contains("Signal") && text.contains("Notes") {
            Ok(TableFormat::PinSignalNotes)
        } else if text.contains("Pin") && text.contains("Function") {
            Ok(TableFormat::PinFunctionVoltage)
        } else if text.contains("Wire Color") {
            Ok(TableFormat::PinWireColorSignal)
        } else {
            Err(CanaryError::UnknownTableFormat)
        }
    }

    fn parse_format_psn(&self, text: &str) -> Result<Vec<PinMapping>> {
        let mut pins = Vec::new();

        // Pattern: "Pin | Signal | Notes"
        let re = regex::Regex::new(
            r"(?m)^\s*(\d+)\s+\|\s+([^|]+)\s+\|\s+([^|]+)$"
        )?;

        for cap in re.captures_iter(text) {
            let pin = PinMapping {
                pin_number: cap[1].parse()?,
                signal_name: cap[2].trim().to_string(),
                notes: Some(cap[3].trim().to_string()),
                ..Default::default()
            };
            pins.push(pin);
        }

        Ok(pins)
    }
}

enum TableFormat {
    PinSignalNotes,        // "Pin | Signal | Notes"
    PinFunctionVoltage,    // "Pin | Function | Voltage"
    PinWireColorSignal,    // "Pin | Wire Color | Signal"
}
```

**OCR Fallback (for scanned PDFs):**

```rust
pub struct OcrPdfParser {
    tesseract: tesseract::Tesseract,
}

impl OcrPdfParser {
    /// Use OCR for scanned PDFs
    pub fn parse_with_ocr(&self, pdf_path: &Path) -> Result<Vec<PinMapping>> {
        // Convert PDF to images
        let images = pdf_to_images(pdf_path)?;

        // OCR each page
        let mut text = String::new();
        for image in images {
            let page_text = self.tesseract.recognize(&image)?;
            text.push_str(&page_text);
        }

        // Parse extracted text
        self.parse_text(&text)
    }
}
```

**Limitations:**
- PDF format variations make 100% accuracy impossible
- OCR accuracy depends on scan quality
- Target: 80% accuracy for well-formatted PDFs
- Manual review for critical ECUs

### 3.4 Validation Pipeline

#### **DataValidator**

```rust
pub struct DataValidator {
    connector_library: HashMap<String, ConnectorDefinition>,
    protocol_list: HashSet<String>,
}

impl DataValidator {
    pub fn validate_ecu(&self, ecu: &EcuPinout) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Required fields
        if ecu.id.is_empty() {
            errors.push("ECU ID is required".to_string());
        }
        if ecu.connectors.is_empty() {
            errors.push("At least one connector required".to_string());
        }

        // ID format
        if !ecu.id.contains('_') {
            warnings.push("ECU ID should use underscores: mfr_model_year_module".to_string());
        }

        // Validate each connector
        for connector in &ecu.connectors {
            self.validate_connector(connector, &mut errors, &mut warnings);
        }

        // Protocol references
        for protocol in &ecu.supported_protocols {
            if !self.protocol_list.contains(protocol) {
                warnings.push(format!("Unknown protocol: {}", protocol));
            }
        }

        // Voltage ranges
        if ecu.power_requirements.voltage_max > 60.0 {
            warnings.push("Voltage max > 60V unusual for automotive".to_string());
        }

        ValidationResult { errors, warnings }
    }

    fn validate_connector(
        &self,
        connector: &ConnectorSpec,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        // Pin count
        if connector.pins.is_empty() {
            errors.push(format!("Connector {} has no pins", connector.connector_id));
        }
        if connector.pins.len() < 10 {
            warnings.push(format!("Connector {} has unusually few pins ({})",
                                  connector.connector_id, connector.pins.len()));
        }

        // Pin number uniqueness
        let pin_numbers: HashSet<_> = connector.pins.iter()
            .map(|p| p.pin_number)
            .collect();
        if pin_numbers.len() != connector.pins.len() {
            errors.push(format!("Duplicate pin numbers in connector {}",
                               connector.connector_id));
        }

        // Pin number range
        for pin in &connector.pins {
            if pin.pin_number == 0 {
                errors.push("Pin number cannot be 0".to_string());
            }
            if pin.signal_name.is_empty() {
                warnings.push(format!("Pin {} has no signal name", pin.pin_number));
            }

            // Voltage validation
            if let Some(v) = pin.voltage {
                if v < 0.0 || v > 24.0 {
                    warnings.push(format!("Pin {} voltage {}V outside typical range (0-24V)",
                                         pin.pin_number, v));
                }
            }
        }

        // Standard connector validation
        if let Some(std_id) = &connector.standard_connector_id {
            if let Some(std_conn) = self.connector_library.get(std_id) {
                if connector.pins.len() > std_conn.pin_count as usize {
                    errors.push(format!("Connector has {} pins but standard {} supports {}",
                                       connector.pins.len(), std_id, std_conn.pin_count));
                }
            }
        }
    }
}

pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}
```

#### **ConflictResolver**

When multiple sources provide data for the same ECU, merge intelligently.

```rust
pub struct ConflictResolver {
    source_priority: HashMap<DataSource, u8>,
}

impl ConflictResolver {
    pub fn new() -> Self {
        let mut priority = HashMap::new();
        priority.insert(DataSource::OemManual, 10);      // Highest priority
        priority.insert(DataSource::Manual, 8);
        priority.insert(DataSource::EcuDesign, 6);
        priority.insert(DataSource::XTuning, 6);
        priority.insert(DataSource::Scribd, 4);
        priority.insert(DataSource::Community, 2);       // Lowest priority

        Self { source_priority }
    }

    pub fn resolve(&self, sources: Vec<EcuPinout>) -> EcuPinout {
        if sources.len() == 1 {
            return sources.into_iter().next().unwrap();
        }

        // Sort by source priority
        let mut sorted = sources;
        sorted.sort_by_key(|ecu| {
            self.source_priority.get(&ecu.metadata.source).copied().unwrap_or(0)
        });
        sorted.reverse(); // Highest priority first

        // Start with highest priority source
        let mut merged = sorted[0].clone();
        merged.metadata.source = DataSource::Merged;
        merged.metadata.sources = sorted.iter()
            .map(|e| format!("{:?}", e.metadata.source))
            .collect();

        // Merge pins from other sources
        for source_ecu in &sorted[1..] {
            self.merge_pins(&mut merged, source_ecu);
        }

        // Recalculate confidence
        merged.metadata.confidence = calculate_confidence(&merged.metadata);

        merged
    }

    fn merge_pins(&self, target: &mut EcuPinout, source: &EcuPinout) {
        for src_connector in &source.connectors {
            // Find matching connector in target
            if let Some(tgt_connector) = target.connectors.iter_mut()
                .find(|c| c.connector_id == src_connector.connector_id)
            {
                // Merge pins
                for src_pin in &src_connector.pins {
                    if let Some(tgt_pin) = tgt_connector.pins.iter_mut()
                        .find(|p| p.pin_number == src_pin.pin_number)
                    {
                        // Pin exists - fill in missing fields
                        if tgt_pin.wire_color.is_none() && src_pin.wire_color.is_some() {
                            tgt_pin.wire_color = src_pin.wire_color.clone();
                        }
                        if tgt_pin.voltage.is_none() && src_pin.voltage.is_some() {
                            tgt_pin.voltage = src_pin.voltage;
                        }
                        // Keep higher-priority signal name
                    } else {
                        // New pin - add it
                        tgt_connector.pins.push(src_pin.clone());
                    }
                }
            }
        }
    }
}
```

**Conflict Resolution Strategy:**
1. **Source Priority:** OEM manual > manual curation > scraped sites > community
2. **Field-Level Merge:** Take non-empty fields from highest priority source
3. **Pin Addition:** Include pins from all sources (union, not intersection)
4. **Confidence Boost:** Multiple agreeing sources increase confidence score

### 3.5 CLI Tool

#### **Binary: `canary-scraper`**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "canary-scraper")]
#[command(about = "Data collection tool for Canary ECU database")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scrape web sources for ECU data
    Scrape {
        /// Source to scrape (ecu-design, xtuning, all)
        source: String,

        /// Manufacturer filter (or "all")
        #[arg(short, long)]
        manufacturer: Option<String>,

        /// Output directory
        #[arg(short, long, default_value = "scraped")]
        output: PathBuf,
    },

    /// Parse PDF documents
    ParsePdf {
        /// Input PDF file or directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "parsed")]
        output: PathBuf,

        /// Use OCR for scanned PDFs
        #[arg(long)]
        ocr: bool,
    },

    /// Validate data files
    Validate {
        /// Directory containing TOML files
        #[arg(short, long)]
        dir: PathBuf,

        /// Strict mode (errors on warnings)
        #[arg(long)]
        strict: bool,
    },

    /// Merge data from multiple sources
    Merge {
        /// Source directories
        #[arg(short, long, num_args = 1..)]
        sources: Vec<PathBuf>,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Compress TOML files to .toml.gz
    Compress {
        /// Input directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
    },
}
```

**Usage Examples:**

```bash
# Scrape ecu.design for VW ECUs
canary-scraper scrape ecu-design --manufacturer vw --output scraped/vw/

# Scrape all manufacturers from xtuning
canary-scraper scrape xtuning --manufacturer all --output scraped/

# Parse PDF with OCR
canary-scraper parse-pdf --input manual.pdf --output parsed/ --ocr

# Validate scraped data
canary-scraper validate --dir scraped/ --strict

# Merge from multiple sources
canary-scraper merge --sources scraped/ parsed/ manual/ --output merged/

# Compress for distribution
canary-scraper compress --input merged/ --output ../canary-data/data/manufacturers/
```

---

## 4. Community Contribution System

### 4.1 Contribution Workflow

#### **CLI Commands**

```bash
# 1. Initialize contribution template
$ canary contribute init --manufacturer honda --model civic --year 2016 --module ecm
Created template: honda_civic_2016_ecm.toml
Edit this file and fill in connector and pin details.

# 2. Validate contribution
$ canary contribute validate honda_civic_2016_ecm.toml
✅ Validation passed
   - ECU ID: honda_civic_2016_ecm
   - Module: ECM
   - Connectors: 1
   - Total pins: 128
   - Confidence score: 0.7 (Community contribution)

⚠️  Warnings:
   - Consider adding wire colors for power pins
   - 5 pins missing voltage specification

# 3. Preview how it will appear
$ canary contribute preview honda_civic_2016_ecm.toml
[Shows formatted output similar to CLI display]

# 4. Submit to GitHub (creates PR)
$ canary contribute submit honda_civic_2016_ecm.toml --message "Add Honda Civic 2016 ECM pinout"
Creating GitHub PR...
✅ Pull request created: https://github.com/canary/canary/pull/123
Please wait for review and approval.
```

#### **Template Generator**

```rust
pub struct ContributionTemplate {
    manufacturer: String,
    model: String,
    year: u16,
    module_type: ModuleType,
}

impl ContributionTemplate {
    pub fn generate(&self) -> String {
        format!(r#"# {manufacturer} {model} {year} {module:?} Pinout
#
# Thank you for contributing to the Canary database!
# Please fill in as much information as you have available.
# Refer to your service manual or ECU label for part numbers.
#
# Fields marked with * are required
# Fields marked with ? are optional but recommended

id = "{manufacturer}_{model}_{year}_{module_lower}"  # * Unique identifier
module_type = "{module:?}"  # * Module type
manufacturer_id = "{manufacturer}"  # * Vehicle manufacturer
ecu_manufacturer = ""  # ? ECU manufacturer (Bosch, Continental, Denso, etc.)

# * OEM Part numbers (check ECU label)
part_numbers = [
    "",  # Example: "37820-5AA-A04"
]

# * Vehicle compatibility
[[vehicle_models]]
brand_id = "{manufacturer}"
model = "{model}"
years = [{year}]  # Add more years if applicable: [2015, 2016, 2017]

# * Connector 1 (add more [[connectors]] blocks if multiple)
[[connectors]]
connector_id = "C1"  # Connector label on ECU
connector_type = ""  # ? e.g., "121-pin Bosch EV1.4"
color = ""  # ? Physical color (Black, Grey, etc.)

# * Pins for Connector 1
# Copy this block for each pin
[[connectors.pins]]
pin_number = 1
signal_name = ""  # * e.g., "Battery Positive", "CAN High", "Injector Cylinder 1"
wire_color = ""  # ? e.g., "BK/WT" for Black with White stripe
voltage = 0.0  # ? Nominal voltage (0.0 if unknown)
signal_type = "Digital"  # Digital, Analog, Power, Ground, CAN, SensorInput, ActuatorOutput
protocol = ""  # ? e.g., "can_2.0b" if CAN pin
notes = ""  # ? Additional information

# Add more pins...

# Power requirements
[power_requirements]
voltage_min = 9.0
voltage_nominal = 12.0
voltage_max = 16.0

# Supported protocols
supported_protocols = []  # e.g., ["can_2.0b", "kwp2000"]

# Metadata - DO NOT EDIT
[metadata]
source = "Community"
contributors = ["$GITHUB_USERNAME"]
confidence = 0.6
last_updated = "{iso_timestamp}"
"#,
            manufacturer = self.manufacturer,
            model = self.model,
            year = self.year,
            module = self.module_type,
            module_lower = format!("{:?}", self.module_type).to_lowercase(),
            iso_timestamp = chrono::Utc::now().to_rfc3339(),
        )
    }
}
```

### 4.2 GitHub Integration

#### **Automated PR Creation**

```rust
pub struct GithubSubmitter {
    token: String,
    repo_owner: String,
    repo_name: String,
}

impl GithubSubmitter {
    pub async fn submit_contribution(
        &self,
        toml_path: &Path,
        message: &str,
    ) -> Result<PullRequest> {
        // 1. Read contribution file
        let content = std::fs::read_to_string(toml_path)?;
        let ecu: EcuPinout = toml::from_str(&content)?;

        // 2. Create branch name
        let branch_name = format!("contrib/{}", ecu.id);

        // 3. Use GitHub API to:
        //    - Fork repo (if not already forked)
        //    - Create branch
        //    - Upload file to correct path
        //    - Create pull request

        let file_path = format!(
            "crates/canary-data/data/manufacturers/{}/{}.toml",
            ecu.manufacturer_id,
            ecu.id
        );

        let pr = self.create_pull_request(
            &branch_name,
            &file_path,
            &content,
            message,
        ).await?;

        Ok(pr)
    }
}
```

#### **GitHub Actions Workflow**

```yaml
# .github/workflows/validate-contribution.yml
name: Validate Community Contribution

on:
  pull_request:
    paths:
      - 'crates/canary-data/data/manufacturers/**/*.toml'

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Validate TOML syntax
        run: |
          for file in $(git diff --name-only origin/main | grep '.toml$'); do
            echo "Validating $file..."
            cargo run --bin canary-validator -- validate-file "$file"
          done

      - name: Check for duplicate ECU IDs
        run: cargo run --bin canary-validator -- check-duplicates

      - name: Build library with new data
        run: cargo build --workspace

      - name: Run tests
        run: cargo test --workspace

      - name: Comment on PR
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '✅ Validation passed! Your contribution looks good. A maintainer will review shortly.'
            })
```

#### **PR Template**

```markdown
# Community ECU Contribution

## Contribution Details

**ECU:** {ecu_id}
**Manufacturer:** {manufacturer}
**Vehicle:** {model} {year}
**Module Type:** {module_type}

## Checklist

Please confirm the following:

- [ ] I have filled in all required fields marked with *
- [ ] Pin numbers are accurate and verified against service manual or ECU label
- [ ] Part numbers are correct (from ECU label)
- [ ] I have tested this pinout data (if applicable)
- [ ] I agree to license this contribution under CC-BY-SA-4.0

## Data Source

Where did you obtain this pinout information?

- [ ] Official OEM service manual
- [ ] ECU label/markings
- [ ] Community forum
- [ ] Personal reverse engineering
- [ ] Other (please specify): ___________

## Verification

Have you personally verified this pinout?

- [ ] Yes, I have tested this pinout on a vehicle
- [ ] Partially verified (specify which pins): ___________
- [ ] Not verified, data from documentation only

## Additional Notes

(Any additional context, warnings, or notes for reviewers)

---

**By submitting this PR, I certify that this information is accurate to the best of my knowledge and I have the right to contribute this data.**
```

### 4.3 Verification System

#### **Database Schema**

```sql
-- Track community verifications of ECU data
CREATE TABLE ecu_verifications (
    id BIGSERIAL PRIMARY KEY,
    ecu_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    verified BOOLEAN NOT NULL,  -- true = accurate, false = inaccurate
    notes TEXT,  -- Optional verification notes
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_verification UNIQUE(ecu_id, user_id)
);

CREATE INDEX idx_ecu_verifications_ecu ON ecu_verifications(ecu_id);
CREATE INDEX idx_ecu_verifications_user ON ecu_verifications(user_id);

-- Track community contributions
CREATE TABLE community_contributions (
    id BIGSERIAL PRIMARY KEY,
    ecu_id VARCHAR(255) NOT NULL UNIQUE,
    contributor_github VARCHAR(255),
    pr_number INTEGER,
    merged_at TIMESTAMP,
    verification_count INTEGER DEFAULT 0,
    confidence_score REAL DEFAULT 0.6
);

-- Contributor leaderboard
CREATE TABLE contributors (
    id BIGSERIAL PRIMARY KEY,
    github_username VARCHAR(255) NOT NULL UNIQUE,
    contributions_count INTEGER DEFAULT 0,
    verifications_count INTEGER DEFAULT 0,
    reputation_score REAL DEFAULT 0.0,
    joined_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

#### **Verification API**

```rust
pub struct VerificationSystem {
    db: &'static DatabaseConnection,
}

impl VerificationSystem {
    /// Add a verification (user confirms data is accurate)
    pub async fn add_verification(
        &self,
        ecu_id: &str,
        user_id: &str,
        verified: bool,
        notes: Option<String>,
    ) -> Result<()> {
        use sea_orm::*;

        let verification = ecu_verifications::ActiveModel {
            ecu_id: Set(ecu_id.to_string()),
            user_id: Set(user_id.to_string()),
            verified: Set(verified),
            notes: Set(notes),
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        verification.insert(self.db).await?;

        // Update confidence score
        self.update_confidence_score(ecu_id).await?;

        Ok(())
    }

    /// Get confidence score (0.0-1.0)
    pub async fn get_confidence_score(&self, ecu_id: &str) -> Result<f32> {
        use sea_orm::*;

        // Count positive vs negative verifications
        let total: i64 = ecu_verifications::Entity::find()
            .filter(ecu_verifications::Column::EcuId.eq(ecu_id))
            .count(self.db)
            .await?;

        if total == 0 {
            return Ok(0.6); // Default for community contributions
        }

        let positive: i64 = ecu_verifications::Entity::find()
            .filter(ecu_verifications::Column::EcuId.eq(ecu_id))
            .filter(ecu_verifications::Column::Verified.eq(true))
            .count(self.db)
            .await?;

        // Simple ratio, but could be weighted by user reputation
        let ratio = positive as f32 / total as f32;

        // Boost for multiple verifications
        let boost = if total >= 5 { 0.1 } else { 0.0 };

        Ok((ratio + boost).min(1.0))
    }

    /// Update confidence score in database
    async fn update_confidence_score(&self, ecu_id: &str) -> Result<()> {
        let score = self.get_confidence_score(ecu_id).await?;

        community_contributions::Entity::update_many()
            .col_expr(community_contributions::Column::ConfidenceScore, Expr::value(score))
            .filter(community_contributions::Column::EcuId.eq(ecu_id))
            .exec(self.db)
            .await?;

        Ok(())
    }
}
```

#### **Reputation System**

```rust
/// Calculate contributor reputation based on activity
pub fn calculate_reputation(
    contributions: u32,
    verifications: u32,
    positive_feedback: u32,
    negative_feedback: u32,
) -> f32 {
    let contribution_score = (contributions as f32 * 10.0).min(500.0);
    let verification_score = (verifications as f32 * 2.0).min(200.0);
    let feedback_ratio = if positive_feedback + negative_feedback > 0 {
        positive_feedback as f32 / (positive_feedback + negative_feedback) as f32
    } else {
        0.5
    };

    (contribution_score + verification_score) * feedback_ratio
}
```

**Leaderboard Display:**

```bash
$ canary contributors top
Top Contributors:
══════════════════════════════════════════════════════════

 1. user123          ⭐ 750 pts   (50 contributions, 75 verifications)
 2. automotive_guru  ⭐ 680 pts   (45 contributions, 80 verifications)
 3. ecu_expert       ⭐ 520 pts   (38 contributions, 60 verifications)
 4. vw_specialist    ⭐ 480 pts   (32 contributions, 70 verifications)
 5. gm_tech          ⭐ 450 pts   (30 contributions, 65 verifications)
```

### 4.4 Anti-Spam Measures

**Rate Limiting:**
- Max 10 contributions per day per user
- Max 50 verifications per day per user
- Exponential backoff on validation failures

**Quality Gates:**
- First 3 contributions from new users require manual review
- Automated spam detection (duplicate content, nonsense data)
- GitHub account age requirement (> 30 days)
- Minimum reputation to bypass manual review

**Automated Checks:**

```rust
pub fn is_spam(ecu: &EcuPinout) -> bool {
    // Check for obvious spam patterns

    // All pins have same signal name
    if ecu.connectors.iter().all(|c| {
        let first_name = &c.pins[0].signal_name;
        c.pins.iter().all(|p| &p.signal_name == first_name)
    }) {
        return true;
    }

    // Nonsense text
    let signal_names: String = ecu.connectors.iter()
        .flat_map(|c| c.pins.iter())
        .map(|p| p.signal_name.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    if contains_gibberish(&signal_names) {
        return true;
    }

    // Unrealistic pin counts
    if ecu.connectors.iter().any(|c| c.pins.len() > 300) {
        return true;
    }

    false
}

fn contains_gibberish(text: &str) -> bool {
    // Simple heuristic: high ratio of consonants
    let consonants = text.chars()
        .filter(|c| "bcdfghjklmnpqrstvwxyz".contains(c.to_lowercase().next().unwrap()))
        .count();
    let vowels = text.chars()
        .filter(|c| "aeiou".contains(c.to_lowercase().next().unwrap()))
        .count();

    if vowels == 0 {
        return true;
    }

    consonants as f32 / vowels as f32 > 5.0
}
```

### 4.5 Licensing & Attribution

**Data License:**
- Community contributions: CC-BY-SA-4.0 (share-alike)
- Scraped data: Respects source licenses
- OEM manuals: Fair use for educational purposes
- Attribution required for all sources

**TOML Metadata:**

```toml
[metadata]
source = "Community"
license = "CC-BY-SA-4.0"
contributors = ["github_user123", "automotive_expert"]
attribution = "Original data from Honda service manual, community verified"
documentation_urls = [
    "https://source-manual-link.com/page123"
]
```

**Attribution Display:**

```bash
$ canary ecu show vw_golf_mk7_2015_ecm

Data Sources:
  - Primary: ecu.design (scraped 2026-03-15)
  - Verified by: 8 community members
  - Contributors: user123, vw_specialist, ecu_guru
  - License: CC-BY-SA-4.0

Documentation:
  - https://ecu.design/vw-golf-mk7-ecm
  - Community thread: https://forum.example.com/...
```

---

## 5. Implementation Phases

### Phase 1: Foundation + MVP (Weeks 1-2)

#### **Week 1: Core Infrastructure**

**Milestone 1.1: Data Model Extension (Days 1-2)**

**Tasks:**
- Create `canary-models/src/embedded/ecu.rs`
  - Define `ModuleType` enum (15 variants)
  - Define `EcuPinout` struct with full metadata
  - Define `ConnectorSpec` and `PinMapping` structs
  - Define `SignalType` enum
  - Define `PowerSpec` and `MemorySpec` structs
- Create `canary-models/src/embedded/connector.rs`
  - Define `ConnectorDefinition` for reusable connectors
  - Define `ConnectorGender` and `MountingType` enums
- Create `canary-models/src/embedded/metadata.rs`
  - Define `DataMetadata` struct
  - Define `DataSource` enum
  - Implement `calculate_confidence()` function
- Update `canary-models/src/lib.rs`
  - Re-export all new types
  - Add to public API documentation
- Extend `DiagnosticCode` in `canary-models/src/embedded/dtc.rs`
  - Add `module_type`, `severity`, `causes`, `symptoms`, `diagnostic_steps` fields
  - Add `DtcSeverity` enum

**Deliverables:**
- ✅ All new types compile
- ✅ Rustdoc comments on all public types
- ✅ Unit tests for validation logic

**Milestone 1.2: Lazy Loading Infrastructure (Days 3-4)**

**Tasks:**
- Add `flate2` dependency to `canary-data/Cargo.toml`
- Create `canary-data/src/lazy.rs`
  - Implement `decompress_gzip()` function
  - Implement `LazyManufacturerData<T>` wrapper type
  - Create macro for generating lazy static loaders per manufacturer
- Update `canary-data/build.rs`
  - Add `compress_manufacturer_files()` function
  - Add `generate_manufacturer_index()` function
  - Add compression validation
- Create `canary-data/data/manufacturers/index.json`
  - Lists all manufacturers with metadata
  - Includes file paths and checksums

**Deliverables:**
- ✅ Decompression works correctly
- ✅ Lazy loading caches decompressed data
- ✅ Build script compresses TOML → .toml.gz
- ✅ Performance test: decompression < 100ms

**Milestone 1.3: Storage Reorganization (Days 5-7)**

**Tasks:**
- Restructure `canary-data/data/` directory:
  - Create `data/universal/` (move existing OBD-II, protocols)
  - Create `data/manufacturers/{vw,gm,ford,toyota,bmw,honda}/` directories
  - Create `data/connectors/` for connector library
- Update `canary-data/src/lib.rs` loaders:
  - Separate `UNIVERSAL_PINOUTS` (embedded) from manufacturer pinouts (lazy)
  - Implement per-manufacturer lazy loaders
  - Add `get_manufacturer_ecu()` function
- Update `canary-pinout/src/lib.rs` service:
  - Add `get_ecu_by_id()` function
  - Add `get_ecus_by_manufacturer()` function
  - Add `get_ecus_by_module_type()` function
  - Add `list_manufacturers()` function
- Create new `canary-ecu` crate (optional, or extend canary-pinout):
  - Service layer for ECU-specific operations
  - Filters by module type, manufacturer, vehicle

**Deliverables:**
- ✅ Directory structure reorganized
- ✅ All existing tests still pass
- ✅ New API functions work
- ✅ Documentation updated

#### **Week 2: MVP Data Collection**

**Milestone 1.4: Manual Data Curation (Days 1-7)**

**Target: 10-15 Critical ECUs**

**ECU Selection Strategy:**
- Geographic diversity: US (GM, Ford), Europe (VW, BMW), Asia (Toyota, Honda)
- Module diversity: ECM (5), TCM (2), BCM (2), ABS (1), universal (2)
- Popular models: High production volume vehicles
- Data availability: Well-documented ECUs with accessible service manuals

**Volkswagen Group (3 ECUs):**

1. **VW Golf Mk7 2015-2020 ECM (Bosch MED17.5)**
   - Source: ecu.design + VW service manual
   - 121-pin connector (T121)
   - CAN 2.0B + KWP2000
   - Part numbers: 06J 906 026 CQ, 0 261 S09 098

2. **Audi A4 B8 2009-2016 ECM (Bosch MED17.1)**
   - Source: Audi service manual
   - 2 connectors: 94-pin + 60-pin
   - CAN + K-Line
   - Part numbers: 8K0 907 115 C

3. **VW Passat B7 2012-2015 TCM (ZF 6HP)**
   - Source: ecu.design
   - Single 86-pin connector
   - CAN bus
   - Part numbers: 09G 927 750 FJ

**GM (2 ECUs):**

4. **Silverado 1500 2014-2018 ECM (Bosch E78)**
   - Source: GM service manual
   - 2 connectors: 80-pin + 32-pin
   - CAN + J1939 (heavy duty variant)
   - Part numbers: 12677884

5. **Corvette C6 2005-2013 BCM**
   - Source: Scribd PDF + community forums
   - 3 connectors
   - CAN + LIN
   - Part numbers: 15945613

**Ford (2 ECUs):**

6. **F-150 2015-2020 ECM (Continental EMS2204)**
   - Source: Ford service information
   - 104-pin connector
   - CAN + K-Line
   - Part numbers: FL3A-12A650-BUB

7. **Mustang 2015-2020 PCM (Bosch MG1CS011)**
   - Source: Mustang forums + service manual
   - 2 connectors: 104-pin + 58-pin
   - CAN bus
   - Part numbers: FR3A-12A650-BZB

**Toyota (2 ECUs):**

8. **Camry 2012-2017 ECM (Denso 89661-06D71)**
   - Source: Toyota repair manual
   - 2 connectors: 35-pin + 31-pin
   - CAN + K-Line
   - Part numbers: 89661-06D71

9. **RAV4 2013-2018 TCM (Denso 89530-42050)**
   - Source: ecu.design
   - Single 36-pin connector
   - CAN bus
   - Part numbers: 89530-42050

**BMW (2 ECUs):**

10. **E90 3-Series 2006-2011 ECM (Bosch DME MS45.1)**
    - Source: BMW TIS service documentation
    - 2 connectors: 88-pin + 60-pin
    - CAN + K-Line
    - Part numbers: 7 575 144

11. **F30 3-Series 2012-2018 ECM (Bosch MEVD17.2.9)**
    - Source: ecu.design
    - Single 134-pin connector
    - CAN + FlexRay
    - Part numbers: 8 604 545

**Universal Modules (3):**

12. **Bosch ABS 9.0 (Multi-Manufacturer)**
    - Source: Bosch technical documentation
    - 25-pin connector
    - CAN bus
    - Used by: VW, Audi, BMW, Mercedes

13. **Continental SRS Airbag Module**
    - Source: Service manuals
    - 2 connectors: 16-pin + 12-pin
    - CAN + dedicated crash sensor lines
    - Common across multiple brands

14. **J1939 ECM (Heavy-Duty Standard)**
    - Source: SAE J1939 specification
    - 9-pin Deutsch connector
    - J1939 CAN (250 kbps or 500 kbps)
    - Universal for trucks/buses

**Data Collection Process:**

For each ECU:
1. **Research:** Gather service manuals, pinout diagrams, community data
2. **Create TOML:** Use template generator or manual creation
3. **Populate Fields:**
   - Basic info: ID, module type, manufacturer
   - Part numbers from ECU labels or service manual
   - Vehicle models and years
   - Connector specifications (type, pin count, color)
   - Pin mappings (all pins with signal names, wire colors, voltages)
   - Power requirements (voltage ranges, current draw)
   - Protocols (CAN, K-Line, etc.)
   - Flash memory specs (if available for tuning)
4. **Validate:** Run through validator
5. **Test:** Build library, run CLI commands, verify lookups work
6. **Document:** Add to README examples

**Deliverables:**
- ✅ 10-15 ECU TOML files with complete data
- ✅ All files pass validation
- ✅ Compressed to .toml.gz (build.rs)
- ✅ Searchable via CLI
- ✅ Examples updated with new ECUs

**Milestone 1.5: CLI Enhancement (Days 1-3, parallel with data collection)**

**New CLI Commands:**

```bash
# ECU-specific lookup
canary ecu show vw_golf_mk7_2015_ecm
canary ecu list --manufacturer vw
canary ecu list --module ecm
canary ecu search "bosch med17"

# Module-specific queries
canary module ecm --manufacturer vw
canary module tcm --year 2015-2020

# Connector lookup
canary connector show bosch_ev1_4_121pin
canary connector list
```

**Implementation:**
- Extend `canary-cli/src/main.rs` with new subcommands
- Add `Commands::Ecu` enum variant
- Add `Commands::Module` enum variant
- Add `Commands::Connector` enum variant
- Implement handler functions
- Update help text and README

**Deliverables:**
- ✅ New CLI commands working
- ✅ Help text updated
- ✅ README examples using new commands
- ✅ CLI README updated

**Week 2 Final Deliverables:**

✅ **Infrastructure:**
- Extended data models in production
- Lazy loading system operational
- Tiered storage architecture implemented
- Binary size < 15MB

✅ **Data:**
- 10-15 ECUs with 100% accurate pinouts
- All ECUs validated and tested
- Organized by manufacturer

✅ **API:**
- New ECU/module/connector lookup functions
- CLI commands for all new features
- Backward compatibility maintained

✅ **Documentation:**
- API docs updated
- CLI usage examples
- Contribution guide started

✅ **Tests:**
- All existing tests passing
- New tests for lazy loading
- Integration tests for ECU lookup
- Performance benchmarks

---

### Phase 2: Automation (Weeks 1-3, Parallel with Phase 1)

#### **Week 1: Web Scrapers**

**Milestone 2.1: Scraper Crate Setup (Days 1-2)**

**Tasks:**
- Create `canary-scraper` workspace crate
- Add dependencies: reqwest, scraper, tokio, regex
- Create directory structure:
  - `src/lib.rs` - Public API
  - `src/scrapers/` - Scraper implementations
  - `src/parsers/` - PDF/table parsers
  - `src/validators/` - Data validators
  - `src/bin/canary_scraper.rs` - CLI tool
- Define `EcuScraper` trait
- Create `RateLimiter` utility
- Create `Cache` utility for HTTP responses

**Deliverables:**
- ✅ Crate compiles
- ✅ Basic CLI skeleton works
- ✅ Dependencies verified

**Milestone 2.2: ECU.Design Scraper (Days 3-5)**

**Tasks:**
- Implement `EcuDesignScraper` struct
- Implement `EcuScraper` trait
- HTML parsing with CSS selectors:
  - Extract ECU list page
  - Extract individual ECU detail pages
  - Parse pinout tables
- Map HTML data to `EcuPinout` struct
- Handle pagination
- Implement caching (disk-based)
- Rate limiting (1 req/2 sec)
- Error handling and retries
- robots.txt compliance check

**Challenges:**
- HTML structure may change (use multiple selectors as fallback)
- Missing data fields (graceful degradation)
- Different ECUs have different table formats

**Testing:**
- Scrape 1 manufacturer (VW) as test
- Validate all scraped ECUs
- Compare manual data vs scraped (for overlap)

**Deliverables:**
- ✅ ECU.Design scraper functional
- ✅ Can scrape single manufacturer
- ✅ Can scrape all manufacturers
- ✅ Data validates successfully
- ✅ Cached to avoid re-scraping

**Milestone 2.3: XTuning Scraper (Days 6-7)**

**Tasks:**
- Implement `XTuningScraper` struct
- Similar structure to ECU.Design but adapted for xtuning.vn
- Handle Vietnamese language text
- Image download for connector diagrams (optional)
- Integration with same caching/rate-limiting infrastructure

**Deliverables:**
- ✅ XTuning scraper functional
- ✅ Data validates
- ✅ Integrated with CLI tool

#### **Week 2: PDF Parsers**

**Milestone 2.4: PDF Table Extraction (Days 1-4)**

**Tasks:**
- Add `pdf-extract` dependency
- Implement `ScribdPdfParser` struct
- Table format detection:
  - Identify column headers (Pin, Signal, Notes, etc.)
  - Regex patterns for common table formats
  - Handle multi-page tables
- Extract pin data:
  - Pin number (integer)
  - Signal name (string)
  - Wire color (pattern: XX/YY)
  - Voltage (float with V suffix)
  - Notes (freeform text)
- Convert extracted data to `PinMapping` structs
- Confidence scoring (based on extraction quality)

**Challenges:**
- PDFs vary wildly in format
- Scanned PDFs need OCR
- Tables span multiple pages
- Inconsistent spacing and alignment

**Testing:**
- Curate 10 sample PDFs with different formats
- Measure accuracy (manually verify vs automatic extraction)
- Target: 80% accuracy for well-formatted PDFs

**Deliverables:**
- ✅ PDF parser works for common table formats
- ✅ Handles multi-page tables
- ✅ Confidence scoring implemented
- ✅ Integration tests with sample PDFs

**Milestone 2.5: OCR Fallback (Days 5-7, optional)**

**Tasks:**
- Add `tesseract-rs` dependency (optional)
- Implement `OcrPdfParser` struct
- Convert PDF pages to images
- Run Tesseract OCR
- Post-process OCR text (error correction)
- Parse OCR output with regex

**Note:** OCR is complex and may be deferred to Phase 3 if time is short.

**Deliverables:**
- ✅ OCR parser works (if implemented)
- ⏸️ Or marked as future enhancement

#### **Week 3: Validation & Merge Pipeline**

**Milestone 2.6: Data Validator (Days 1-2)**

**Tasks:**
- Implement `DataValidator` struct (as designed in Section 3.4)
- Required field checks
- Pin number validation (range, uniqueness)
- Voltage range validation (0-24V automotive range)
- Protocol reference validation
- Connector reference validation
- Confidence score calculation
- Error vs warning classification

**Deliverables:**
- ✅ Validator catches all error conditions
- ✅ Integration with scraper output
- ✅ CLI validation command works

**Milestone 2.7: Conflict Resolver (Days 3-4)**

**Tasks:**
- Implement `ConflictResolver` struct (as designed in Section 3.4)
- Source priority ranking
- Field-level merge logic
- Pin addition (union of all sources)
- Confidence boost for multiple sources
- Metadata tracking (all sources listed)

**Testing:**
- Create test cases with conflicting data
- Verify correct source priority
- Verify field-level merge works
- Verify confidence calculation

**Deliverables:**
- ✅ Conflict resolver merges data correctly
- ✅ Source attribution preserved
- ✅ Confidence scores accurate

**Milestone 2.8: CLI Tool Completion (Days 5-7)**

**Tasks:**
- Implement all CLI subcommands:
  - `scrape` - Run web scrapers
  - `parse-pdf` - Parse PDF documents
  - `validate` - Validate data files
  - `merge` - Merge from multiple sources
  - `compress` - Compress to .toml.gz
- Add progress bars (indicatif crate)
- Add logging (env_logger)
- Add configuration file support (optional)
- Write CLI usage documentation

**Deliverables:**
- ✅ All CLI commands functional
- ✅ Help text comprehensive
- ✅ Progress indicators work
- ✅ User-friendly error messages

**Phase 2 Final Deliverables:**

✅ **Scrapers:**
- ECU.Design scraper operational
- XTuning scraper operational
- Combined: 500+ ECUs scraped

✅ **Parsers:**
- PDF parser 80%+ accuracy
- Handles common table formats
- OCR fallback (if time permits)

✅ **Validation:**
- Comprehensive validator
- Conflict resolution system
- Data merge pipeline

✅ **CLI:**
- Full-featured data collection tool
- User-friendly interface
- Documentation complete

✅ **Testing:**
- Integration tests for all scrapers
- Validation test suite
- Real-world data collection test run

---

### Phase 3: Community & Scale (Weeks 4-7)

#### **Week 4: Community Tools**

**Milestone 3.1: Contribution CLI (Days 1-3)**

**Tasks:**
- Extend `canary-cli` with `contribute` subcommand
- Implement `ContributionTemplate` generator
- Implement TOML validator (reuse from scraper)
- Implement preview renderer
- Implement GitHub PR creation via API
  - OAuth flow for GitHub authentication
  - Fork repository if needed
  - Create branch
  - Upload file
  - Create PR with template

**Deliverables:**
- ✅ `canary contribute init` works
- ✅ `canary contribute validate` works
- ✅ `canary contribute preview` works
- ✅ `canary contribute submit` creates GitHub PR
- ✅ Documentation written

**Milestone 3.2: GitHub Integration (Days 4-7)**

**Tasks:**
- Create `.github/workflows/validate-contribution.yml`
- Implement validation GitHub Action
- Create PR template (`.github/PULL_REQUEST_TEMPLATE/contribution.md`)
- Create issue templates for:
  - ECU request
  - Data correction
  - Bug report
- Set up CODEOWNERS (maintainers review contributions)
- Create contribution guide (CONTRIBUTING.md)

**Deliverables:**
- ✅ GitHub Actions validate PRs automatically
- ✅ PR template guides contributors
- ✅ Issue templates streamline requests
- ✅ CONTRIBUTING.md comprehensive

#### **Week 5: Data Collection Blitz**

**Milestone 3.3: Automated Collection (Days 1-5)**

**Tasks:**
- Run ECU.Design scraper for all manufacturers
- Run XTuning scraper for all manufacturers
- Parse curated PDF collection (50-100 PDFs)
- Validate all scraped data
- Resolve conflicts across sources
- Manual spot-check of critical ECUs
- Compress and organize data

**Expected Output:**
- ~300-500 ECUs from ecu.design
- ~200-300 ECUs from xtuning.vn
- ~50-100 ECUs from PDFs
- Total: 550-900 ECUs (after deduplication)

**Quality Control:**
- All data passes automated validation
- Critical safety ECUs (ABS, SRS) manually reviewed
- High-value ECUs (popular models) verified
- Confidence scores calculated

**Deliverables:**
- ✅ 800-1000 ECUs in database
- ✅ All data validated
- ✅ Conflicts resolved
- ✅ Organized by manufacturer
- ✅ Compressed to .toml.gz

**Milestone 3.4: Integration & Testing (Days 6-7)**

**Tasks:**
- Rebuild library with full dataset
- Run all tests
- Benchmark performance:
  - Binary size
  - Startup time
  - Lazy loading latency
  - Memory usage
- Update examples with diverse ECUs
- Update CLI screenshots
- Performance tuning if needed

**Deliverables:**
- ✅ Library builds successfully
- ✅ All tests pass
- ✅ Performance meets targets
- ✅ Examples updated

#### **Week 6: Verification System**

**Milestone 3.5: Database Schema (Days 1-2)**

**Tasks:**
- Create migration for verification tables:
  - `ecu_verifications`
  - `community_contributions`
  - `contributors`
- Add indexes for performance
- Update `canary-models/src/dto/` with SeaORM entities
- Implement verification service in `canary-database`

**Deliverables:**
- ✅ Migration applies successfully
- ✅ Tables created with correct schema
- ✅ SeaORM entities generated

**Milestone 3.6: Verification API (Days 3-5)**

**Tasks:**
- Implement `VerificationSystem` struct (as designed in Section 4.3)
- Add verification endpoints (if web service)
- Implement confidence score calculation
- Implement reputation system
- Create contributor leaderboard query
- Add CLI commands:
  - `canary verify <ecu_id> --confirm`
  - `canary verify <ecu_id> --flag "Incorrect pin 23"`
  - `canary contributors top`

**Deliverables:**
- ✅ Verification system functional
- ✅ Confidence scores update correctly
- ✅ Leaderboard works
- ✅ CLI commands added

**Milestone 3.7: Anti-Spam (Days 6-7)**

**Tasks:**
- Implement spam detection (as designed in Section 4.4)
- Add rate limiting to contribution endpoints
- Implement manual review queue for new users
- Create moderation interface (admin CLI or web)
- GitHub account age requirement
- Automated checks in validation GitHub Action

**Deliverables:**
- ✅ Spam detection catches obvious spam
- ✅ Rate limiting works
- ✅ Manual review queue functional
- ✅ Moderation tools available

#### **Week 7: Documentation & Release**

**Milestone 3.8: Documentation (Days 1-4)**

**Tasks:**
- Write comprehensive CONTRIBUTING.md:
  - How to contribute ECU data
  - Data quality standards
  - TOML format guide
  - GitHub workflow
- Update README.md:
  - New features section
  - ECU lookup examples
  - Community section
- Write data quality guide (docs/data-quality.md)
- Write scraper documentation (docs/scraping.md)
- Update API documentation (rustdoc)
- Create tutorial videos (optional)
- Write blog post announcing expansion

**Deliverables:**
- ✅ CONTRIBUTING.md complete
- ✅ README.md updated
- ✅ All guides written
- ✅ Rustdoc published
- ✅ Blog post ready

**Milestone 3.9: Release (Days 5-7)**

**Tasks:**
- Update CHANGELOG.md for v0.2.0
- Bump version numbers
- Run final test suite
- Create git tag v0.2.0
- Publish crates to crates.io:
  - `canary-models` v0.2.0
  - `canary-data` v0.2.0
  - `canary-database` v0.2.0
  - `canary-pinout` v0.2.0
  - `canary-protocol` v0.2.0
  - `canary-dtc` v0.2.0
  - `canary-service-proc` v0.2.0
  - `canary-scraper` v0.1.0 (new)
  - `canary-core` v0.2.0
  - `canary-cli` v0.2.0
- Create GitHub release with:
  - Release notes
  - Binary downloads
  - Migration guide
- Announce on:
  - Reddit (r/rust, r/justrolledintotheshop)
  - Hacker News
  - Rust forum
  - Automotive forums
- Monitor for issues and feedback

**Deliverables:**
- ✅ v0.2.0 released to crates.io
- ✅ GitHub release created
- ✅ Announcements posted
- ✅ Community engagement started

**Phase 3 Final Deliverables:**

✅ **Data:**
- 800-1000 ECU pinouts
- Multiple manufacturers covered
- High-quality verified data

✅ **Community:**
- Contribution system operational
- Verification system live
- GitHub integration working
- Anti-spam measures effective

✅ **Documentation:**
- Comprehensive guides
- API docs updated
- Tutorial content
- Blog post published

✅ **Release:**
- v0.2.0 on crates.io
- Community engaged
- Feedback loop established

---

## 6. Testing & Quality Assurance

### 6.1 Testing Strategy

**Test Pyramid:**
```
           /\
          /  \  Integration Tests (10%)
         /____\
        /      \  Unit Tests (70%)
       /________\
      /          \ Validation Tests (20%)
     /____________\
```

#### **Unit Tests**

**Coverage Targets:**
- All new data models: 100%
- Lazy loading system: 100%
- Validators: 100%
- Scrapers: 80% (mocked HTTP)
- Parsers: 80% (sample files)

**Key Test Files:**

```rust
// canary-data/src/lazy.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_decompress_manufacturer_data() {
        let data = load_manufacturer("vw").unwrap();
        assert!(data.len() > 0);
        assert!(data.contains_key("vw_golf_mk7_2015_ecm"));
    }

    #[test]
    fn test_lazy_cache_performance() {
        // First load
        let start = Instant::now();
        let _ = load_manufacturer("vw").unwrap();
        let first_duration = start.elapsed();

        // Second load (should be cached)
        let start = Instant::now();
        let _ = load_manufacturer("vw").unwrap();
        let cached_duration = start.elapsed();

        // Cached access should be 10x+ faster
        assert!(cached_duration < first_duration / 10);
    }

    #[test]
    fn test_decompression_integrity() {
        // Verify decompressed data matches original
        let original = std::fs::read_to_string("test_data/sample.toml").unwrap();
        let compressed = compress_gzip(&original).unwrap();
        let decompressed = decompress_gzip(&compressed).unwrap();

        assert_eq!(original, decompressed);
    }
}

// canary-models/src/embedded/ecu.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_ecu_pinout_validation() {
        let ecu = EcuPinout {
            id: "test_ecu".to_string(),
            module_type: ModuleType::ECM,
            // ... populate fields
        };

        assert!(!ecu.id.is_empty());
        assert!(ecu.connectors.len() > 0);
        assert!(ecu.connectors[0].pins.len() > 0);
    }

    #[test]
    fn test_pin_number_uniqueness() {
        let connector = ConnectorSpec {
            pins: vec![
                PinMapping { pin_number: 1, /* ... */ },
                PinMapping { pin_number: 2, /* ... */ },
                PinMapping { pin_number: 1, /* ... */ }, // Duplicate!
            ],
            // ...
        };

        let pin_numbers: HashSet<_> = connector.pins.iter()
            .map(|p| p.pin_number)
            .collect();

        // This should fail if there are duplicates
        assert_ne!(pin_numbers.len(), connector.pins.len());
    }
}
```

#### **Integration Tests**

```rust
// tests/ecu_lookup.rs
#[tokio::test]
async fn test_lookup_vw_golf_ecm() {
    canary_core::initialize(None).await.unwrap();

    let ecus = EcuService::get_by_vehicle("vw", "golf", 2015).unwrap();
    assert!(ecus.len() > 0);

    let ecm = ecus.iter()
        .find(|e| e.module_type == ModuleType::ECM)
        .expect("Should find Golf ECM");

    assert_eq!(ecm.manufacturer_id, "vw");
    assert!(ecm.connectors.len() > 0);

    // Verify specific pins
    let connector = &ecm.connectors[0];
    let can_high = connector.pins.iter()
        .find(|p| p.signal_name.contains("CAN") && p.signal_name.contains("High"))
        .expect("Should have CAN High pin");

    assert_eq!(can_high.signal_type, SignalType::CAN);
}

#[test]
fn test_lazy_loading_performance() {
    // Should load manufacturer data in < 100ms
    let start = Instant::now();
    let _ = EcuService::get_by_manufacturer("vw").unwrap();
    let duration = start.elapsed();

    assert!(duration.as_millis() < 100,
            "Lazy loading took {}ms, expected < 100ms", duration.as_millis());
}

#[test]
fn test_all_manufacturers_load() {
    let manufacturers = ["vw", "gm", "ford", "toyota", "bmw", "honda"];

    for mfr in manufacturers {
        let ecus = EcuService::get_by_manufacturer(mfr);
        assert!(ecus.is_ok(), "Failed to load manufacturer: {}", mfr);
        assert!(ecus.unwrap().len() > 0, "No ECUs for manufacturer: {}", mfr);
    }
}
```

#### **Scraper Tests**

```rust
// canary-scraper/tests/integration.rs
#[tokio::test]
async fn test_ecu_design_scraper_single() {
    let scraper = EcuDesignScraper::new(PathBuf::from("test_cache"));

    // Use cached data to avoid hitting live site in tests
    let ecus = scraper.scrape_manufacturer("vw").await.unwrap();

    assert!(ecus.len() > 5, "Should find multiple VW ECUs");

    for ecu in &ecus {
        assert!(!ecu.id.is_empty());
        assert!(ecu.manufacturer_id == "vw");
        assert!(ecu.connectors.len() > 0);
    }
}

#[test]
fn test_pdf_parser_table_extraction() {
    let parser = ScribdPdfParser::new();
    let pins = parser.parse_pinout_table(
        Path::new("test_data/sample_ecu_pinout.pdf")
    ).unwrap();

    assert!(pins.len() > 20, "Should extract multiple pins");

    // Verify structure
    for pin in &pins {
        assert!(pin.pin_number > 0);
        assert!(!pin.signal_name.is_empty());
    }

    // Verify specific pins from known PDF
    let pin_1 = pins.iter().find(|p| p.pin_number == 1).unwrap();
    assert_eq!(pin_1.signal_name, "Battery Positive");
}

#[test]
fn test_validator_catches_errors() {
    let validator = DataValidator::new();

    let invalid_ecu = EcuPinout {
        id: "".to_string(), // Invalid: empty ID
        connectors: vec![], // Invalid: no connectors
        // ...
    };

    let result = validator.validate_ecu(&invalid_ecu);
    assert!(!result.is_valid());
    assert!(result.errors.len() >= 2); // Should catch both issues
}
```

#### **Validation Tests**

```rust
#[test]
fn test_validate_all_embedded_data() {
    let validator = DataValidator::new();

    // Load all TOML files from data directory
    let toml_files = glob("crates/canary-data/data/**/*.toml")
        .expect("Failed to read glob pattern");

    let mut errors = Vec::new();

    for entry in toml_files {
        let path = entry.unwrap();
        let content = std::fs::read_to_string(&path)
            .expect(&format!("Failed to read {:?}", path));

        // Parse and validate
        match toml::from_str::<EcuPinout>(&content) {
            Ok(ecu) => {
                let result = validator.validate_ecu(&ecu);
                if !result.is_valid() {
                    errors.push((path.clone(), result.errors));
                }
            }
            Err(e) => {
                errors.push((path.clone(), vec![format!("Parse error: {}", e)]));
            }
        }
    }

    if !errors.is_empty() {
        for (path, errs) in errors {
            eprintln!("Validation failed for {:?}:", path);
            for err in errs {
                eprintln!("  - {}", err);
            }
        }
        panic!("Data validation failed");
    }
}

#[test]
fn test_no_duplicate_ecu_ids() {
    let all_ecus = load_all_ecus_from_data_dir();

    let mut id_counts: HashMap<String, usize> = HashMap::new();
    for ecu in &all_ecus {
        *id_counts.entry(ecu.id.clone()).or_insert(0) += 1;
    }

    let duplicates: Vec<_> = id_counts.iter()
        .filter(|(_, &count)| count > 1)
        .collect();

    assert!(duplicates.is_empty(),
            "Found duplicate ECU IDs: {:?}", duplicates);
}
```

### 6.2 Quality Gates

#### **Pre-Commit Hooks**

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: validate-toml
        name: Validate TOML data files
        entry: cargo run --bin canary-validator -- validate-all
        language: system
        pass_filenames: false
        files: \.toml$

      - id: check-duplicates
        name: Check for duplicate ECU IDs
        entry: cargo run --bin canary-validator -- check-duplicates
        language: system
        pass_filenames: false

      - id: cargo-fmt
        name: Format Rust code
        entry: cargo fmt --all -- --check
        language: system
        pass_filenames: false

      - id: cargo-clippy
        name: Lint Rust code
        entry: cargo clippy --all -- -D warnings
        language: system
        pass_filenames: false
```

#### **CI/CD Pipeline**

```yaml
# .github/workflows/ci.yml
name: Continuous Integration

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --workspace --all-features

      - name: Validate all data
        run: cargo run --bin canary-validator -- validate-all

      - name: Check for duplicate IDs
        run: cargo run --bin canary-validator -- check-duplicates

      - name: Check binary size
        run: |
          cargo build --release
          SIZE=$(stat -c%s target/release/libcanary_core.so)
          MAX_SIZE=20000000  # 20MB limit
          if [ $SIZE -gt $MAX_SIZE ]; then
            echo "::error::Binary size $SIZE exceeds limit $MAX_SIZE"
            exit 1
          fi
          echo "Binary size: $SIZE bytes (limit: $MAX_SIZE)"

      - name: Run benchmarks
        run: cargo bench --bench lazy_loading -- --output-format bencher

  lint:
    name: Code Quality
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --all --all-features -- -D warnings

  security:
    name: Security Audit
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Run security audit
        uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### 6.3 Data Quality Standards

#### **Required Fields per ECU**

✅ **Mandatory:**
- Unique ID (no duplicates)
- Valid module type
- Manufacturer ID
- At least 1 connector
- At least 10 pins (realistic minimum for ECU)
- All pin numbers < 256
- No duplicate pin numbers per connector

✅ **Highly Recommended:**
- ECU manufacturer (Bosch, Continental, etc.)
- Part numbers (OEM codes)
- Vehicle models and years
- Wire colors (for at least power/ground pins)
- Protocol references (CAN, K-Line, etc.)
- Source attribution

✅ **Optional:**
- Voltage specifications
- Current ratings
- Flash memory specs
- Documentation URLs

#### **Confidence Scoring**

```rust
pub fn calculate_confidence(ecu: &EcuPinout) -> f32 {
    let mut score = 0.0;

    // Source quality (max 0.4)
    score += match ecu.metadata.source {
        DataSource::OemManual => 0.4,
        DataSource::Manual => 0.35,
        DataSource::EcuDesign | DataSource::XTuning => 0.25,
        DataSource::Scribd => 0.2,
        DataSource::Community => 0.15,
        DataSource::Merged => 0.3,
    };

    // Multiple sources (max 0.3)
    if ecu.metadata.sources.len() >= 3 {
        score += 0.3;
    } else if ecu.metadata.sources.len() == 2 {
        score += 0.15;
    }

    // Community verification (max 0.2)
    if ecu.metadata.verified_by >= 5 {
        score += 0.2;
    } else if ecu.metadata.verified_by >= 2 {
        score += 0.1;
    }

    // Data completeness (max 0.1)
    let completeness = calculate_completeness(ecu);
    score += completeness * 0.1;

    score.min(1.0)
}

fn calculate_completeness(ecu: &EcuPinout) -> f32 {
    let mut points = 0.0;
    let max_points = 10.0;

    // Has part numbers
    if !ecu.part_numbers.is_empty() { points += 1.0; }

    // Has ECU manufacturer
    if !ecu.ecu_manufacturer.is_empty() { points += 1.0; }

    // Has vehicle models
    if !ecu.vehicle_models.is_empty() { points += 1.0; }

    // Has power requirements
    if ecu.power_requirements.voltage_nominal > 0.0 { points += 1.0; }

    // Has protocols
    if !ecu.supported_protocols.is_empty() { points += 1.0; }

    // Has flash memory info
    if ecu.flash_memory.is_some() { points += 1.0; }

    // Pin data quality (4 points max)
    let total_pins: usize = ecu.connectors.iter().map(|c| c.pins.len()).sum();
    if total_pins > 0 {
        let pins_with_wire_color = ecu.connectors.iter()
            .flat_map(|c| &c.pins)
            .filter(|p| p.wire_color.is_some())
            .count();
        let pins_with_voltage = ecu.connectors.iter()
            .flat_map(|c| &c.pins)
            .filter(|p| p.voltage.is_some())
            .count();

        points += (pins_with_wire_color as f32 / total_pins as f32) * 2.0;
        points += (pins_with_voltage as f32 / total_pins as f32) * 2.0;
    }

    points / max_points
}
```

#### **Manual Review Criteria**

**Critical ECUs requiring human review:**
- Safety systems (ABS, SRS airbag) - verify thoroughly
- High confidence threshold needed (0.9+)
- First ECU from new manufacturer
- Community-flagged as incorrect
- Conflicting data from multiple sources

**Review Checklist:**
- [ ] Pin numbers sequential and logical
- [ ] Power/ground pins present and realistic
- [ ] CAN/communication pins properly identified
- [ ] Wire colors follow industry standards (if present)
- [ ] Part numbers verified against images or documentation
- [ ] No obviously missing critical pins
- [ ] Source documentation linked and accessible
- [ ] Confidence score justified

### 6.4 Error Handling

#### **Graceful Degradation**

```rust
/// Load ECU with fallback strategy
pub fn get_ecu(id: &str) -> Result<EcuPinout> {
    // Try compressed data first
    match load_from_compressed(id) {
        Ok(ecu) => Ok(ecu),
        Err(e) => {
            log::warn!("Failed to load compressed data for {}: {}", id, e);

            // Fallback to database
            if let Ok(db) = canary_database::get_connection() {
                load_from_database(db, id)
                    .or_else(|_| Err(CanaryError::EcuNotFound(id.to_string())))
            } else {
                Err(CanaryError::EcuNotFound(id.to_string()))
            }
        }
    }
}
```

#### **User-Friendly Error Messages**

```rust
#[derive(Error, Debug)]
pub enum CanaryError {
    #[error("ECU not found: {0}. Try updating the library or contribute this ECU at https://github.com/canary/canary/blob/main/CONTRIBUTING.md")]
    EcuNotFound(String),

    #[error("Data corrupted for manufacturer: {0}. Please reinstall or report issue at https://github.com/canary/canary/issues")]
    DataCorrupted(String),

    #[error("Manufacturer data not loaded yet. Call EcuService::preload(\"{0}\") to load before use.")]
    ManufacturerNotLoaded(String),

    #[error("Invalid ECU ID format: {0}. Expected format: manufacturer_model_year_module (e.g., vw_golf_2015_ecm)")]
    InvalidEcuId(String),

    #[error("Database not initialized. Call canary_core::initialize(Some(\"database_url\")) first.")]
    DatabaseNotInitialized,

    #[error("Network error while scraping: {0}. Check internet connection or try again later.")]
    NetworkError(String),

    #[error("PDF parsing failed: {0}. The PDF may be scanned (requires OCR) or use an unsupported format.")]
    PdfParseError(String),
}
```

#### **Logging Strategy**

```rust
// Use env_logger for configurable logging

// In library code
log::debug!("Loading manufacturer data: {}", manufacturer);
log::info!("Loaded {} ECUs for manufacturer {}", count, manufacturer);
log::warn!("Missing wire color data for pin {} in {}", pin_num, ecu_id);
log::error!("Failed to decompress data for {}: {}", manufacturer, error);

// Users configure via environment variable
// RUST_LOG=canary=debug cargo run
// RUST_LOG=canary=info,canary_scraper=debug cargo run
```

### 6.5 Performance Benchmarks

#### **Benchmark Suite**

```rust
// benches/lazy_loading.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_lazy_loading(c: &mut Criterion) {
    c.bench_function("load_vw_ecus_first_time", |b| {
        b.iter(|| {
            // Clear cache before each iteration
            canary_data::clear_cache();

            // First load (includes decompression)
            black_box(canary_data::load_manufacturer("vw").unwrap());
        });
    });

    c.bench_function("load_vw_ecus_cached", |b| {
        // Warm up cache
        let _ = canary_data::load_manufacturer("vw").unwrap();

        b.iter(|| {
            // Cached access
            black_box(canary_data::load_manufacturer("vw").unwrap());
        });
    });
}

fn benchmark_ecu_lookup(c: &mut Criterion) {
    c.bench_function("lookup_ecu_by_id", |b| {
        b.iter(|| {
            black_box(EcuService::get_by_id("vw_golf_mk7_2015_ecm").unwrap());
        });
    });

    c.bench_function("filter_ecus_by_module_type", |b| {
        b.iter(|| {
            let ecus = EcuService::get_by_manufacturer("vw").unwrap();
            black_box(ecus.into_iter()
                .filter(|e| e.module_type == ModuleType::ECM)
                .collect::<Vec<_>>());
        });
    });
}

criterion_group!(benches, benchmark_lazy_loading, benchmark_ecu_lookup);
criterion_main!(benches);
```

**Performance Targets:**

| Operation | Target | Rationale |
|-----------|--------|-----------|
| First manufacturer load | < 100ms | Decompression + parsing |
| Cached manufacturer load | < 1ms | HashMap lookup only |
| ECU lookup by ID | < 1ms | HashMap lookup |
| Filter by module type | < 5ms | In-memory iteration |
| Binary size (with all data) | < 20MB | Compressed data keeps it small |
| Memory usage (idle) | < 50MB | Only loaded manufacturers in memory |
| Startup time | < 200ms | No upfront decompression |

---

## 7. Success Metrics

### Phase 1 (Foundation + MVP)

**Technical Metrics:**
- ✅ Binary size < 15MB (with 10-15 ECUs)
- ✅ Lazy loading < 100ms per manufacturer
- ✅ All tests passing (35+ existing + 20+ new = 55+ total)
- ✅ Zero compiler errors or warnings

**Data Metrics:**
- ✅ 10-15 ECUs with 100% accurate pinouts
- ✅ 100% validation pass rate
- ✅ All critical fields populated
- ✅ Confidence scores >= 0.8

**API Metrics:**
- ✅ 100% backward compatibility
- ✅ New API functions documented
- ✅ CLI commands functional

### Phase 2 (Automation)

**Scraper Metrics:**
- ✅ ECU.Design: 300-500 ECUs collected
- ✅ XTuning: 200-300 ECUs collected
- ✅ PDF Parser: 80%+ accuracy on well-formatted PDFs
- ✅ Zero false positives in validation

**Quality Metrics:**
- ✅ 90%+ scraped data passes validation
- ✅ Conflict resolution preserves source attribution
- ✅ Confidence scores accurately reflect data quality

**Performance Metrics:**
- ✅ Scraping rate: 1 ECU every 2-5 seconds (rate-limited)
- ✅ PDF parsing: < 30 seconds per document
- ✅ Validation: < 1 second per ECU

### Phase 3 (Community & Scale)

**Data Scale Metrics:**
- ✅ 800-1000 total ECUs in database
- ✅ Coverage of 10+ major manufacturers
- ✅ All module types represented (ECM, TCM, BCM, ABS, etc.)
- ✅ 50+ unique vehicle models

**Community Metrics:**
- ✅ 10+ community contributions in first month
- ✅ 50+ ECU verifications
- ✅ 20+ active contributors
- ✅ < 5% spam/invalid contributions

**Quality Metrics:**
- ✅ Average confidence score >= 0.7
- ✅ Critical ECUs (ABS, SRS) >= 0.9 confidence
- ✅ Zero data integrity issues in production

**Engagement Metrics:**
- ✅ 100+ GitHub stars
- ✅ 500+ crate downloads/month
- ✅ Active community discussions
- ✅ Positive feedback on quality

---

## 8. Risk Mitigation

### Technical Risks

**Risk: Lazy loading performance issues**
- **Mitigation:** Benchmark early, optimize decompression, consider LZ4 if gzip too slow
- **Fallback:** Reduce compression level, or embed more data

**Risk: Binary size exceeds limits**
- **Mitigation:** Aggressive compression, consider external data downloads
- **Fallback:** Split data into optional feature flags

**Risk: Data format changes break compatibility**
- **Mitigation:** Versioned TOML schemas, migration tools
- **Fallback:** Support multiple schema versions

### Data Quality Risks

**Risk: Scraped data has high error rate**
- **Mitigation:** Manual spot-checks, community verification
- **Fallback:** Higher confidence threshold, manual-only for critical ECUs

**Risk: Conflicting data from multiple sources**
- **Mitigation:** Conflict resolver with source priority
- **Fallback:** Mark conflicts clearly, let users decide

**Risk: Community spam/vandalism**
- **Mitigation:** Anti-spam measures, moderation queue
- **Fallback:** Disable public contributions, manual review only

### Legal Risks

**Risk: Copyright issues with scraped data**
- **Mitigation:** Respect robots.txt, fair use for educational purposes, proper attribution
- **Fallback:** Remove contested data, rely on community contributions

**Risk: Manufacturer legal action**
- **Mitigation:** Educational/research use defense, no commercial exploitation
- **Fallback:** Limit to public domain / community data only

---

## 9. Future Enhancements

### Post-v0.2.0 Roadmap

**v0.3.0 - Enhanced Protocols**
- J1939 decoder implementation
- LIN bus decoder
- FlexRay support (basic)
- Real-time CAN streaming

**v0.4.0 - AI/ML Features**
- DTC prediction from sensor patterns
- Automatic pinout extraction from images (OCR + ML)
- Smart conflict resolution (ML-based)

**v0.5.0 - Web Platform**
- Web UI for browsing ECU database
- Online verification system
- Community forums
- API endpoints for third-party integration

**v0.6.0 - Commercial Features**
- Premium data (dealer-level access)
- Tuning file repository
- Diagnostic guided workflows
- Subscription model for advanced features

---

## 10. Conclusion

This design provides a comprehensive, scalable approach to expanding Canary with extensive automotive ECU and pinout data. The three-phase implementation strategy balances immediate value (MVP with 10-15 ECUs), automation efficiency (web scraping and PDF parsing), and long-term sustainability (community contributions and verification).

**Key Strengths:**
- ✅ Tiered storage optimizes binary size vs. data volume
- ✅ Lazy loading ensures fast startup and low memory usage
- ✅ Hybrid data collection (automated + community) maximizes coverage
- ✅ Comprehensive validation and quality control
- ✅ Backward compatibility maintained
- ✅ Clear success metrics and risk mitigation

**Expected Outcomes:**
- **Phase 1:** Production-ready foundation with 10-15 high-quality ECUs (2 weeks)
- **Phase 2:** Automated collection tools and 500+ ECUs (3 weeks)
- **Phase 3:** Community-driven platform with 800-1000 ECUs and ongoing growth (4 weeks)
- **Total:** v0.2.0 release in 7-9 weeks with comprehensive ECU coverage

This design is ready for implementation.
