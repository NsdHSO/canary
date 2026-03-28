pub mod manufacturer;
pub mod pinout;
pub mod protocol;
pub mod dtc;
pub mod service_proc;
pub mod ecu;
pub mod connector;
pub mod metadata;

pub use manufacturer::{Manufacturer, Brand, VehicleModel};
pub use pinout::{ConnectorPinout, PinMapping};
pub use protocol::{ProtocolSpec, FrameFormat, CanFrame, KLineFrame};
pub use dtc::{DiagnosticCode, DtcSystem, ManufacturerDtcInfo};
pub use service_proc::{ServiceProcedure, ProcedureCategory, ProcedureStep};
pub use ecu::{ModuleType, SignalType, PinMapping as EcuPinMapping, ConnectorSpec, PowerSpec, MemorySpec, EcuPinout};
pub use connector::{UniversalConnector, StandardPinMapping, ConnectorDefinition, ConnectorGender, MountingType};
pub use metadata::{DataMetadata, DataSource, default_license};
