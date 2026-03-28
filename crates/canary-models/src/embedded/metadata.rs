use serde::{Deserialize, Serialize};

/// Data provenance and quality metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataMetadata {
    /// Source of the data
    pub source: DataSource,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Date added (YYYY-MM-DD)
    pub date_added: String,
    /// Last verified date (YYYY-MM-DD)
    pub last_verified: Option<String>,
    /// Last modified date (YYYY-MM-DD)
    pub last_modified: Option<String>,
    /// Version number
    pub version: String,
    /// Contributors (usernames or identifiers)
    pub contributors: Vec<String>,
    /// Data license
    pub license: String,
    /// References to documentation or sources
    pub references: Vec<String>,
    /// Notes about data quality or limitations
    pub notes: Option<String>,
}

/// Source of ECU pinout data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSource {
    /// Official manufacturer documentation
    OfficialDocumentation,
    /// Reverse engineered from physical ECU
    ReverseEngineered,
    /// Community contributed data
    CommunityContributed,
    /// Third-party service manuals
    ServiceManual,
    /// Verified by testing with physical hardware
    HardwareTested,
    /// Verified by multiple independent sources
    MultiSourceVerified,
    /// Extracted from diagnostic tools
    DiagnosticTool,
}

impl DataMetadata {
    /// Calculate confidence score based on source and verification
    pub fn calculate_confidence(source: &DataSource, verified: bool, num_sources: usize) -> f32 {
        let base_confidence = match source {
            DataSource::OfficialDocumentation => 0.95,
            DataSource::HardwareTested => 0.90,
            DataSource::MultiSourceVerified => 0.85,
            DataSource::ServiceManual => 0.80,
            DataSource::DiagnosticTool => 0.75,
            DataSource::ReverseEngineered => 0.70,
            DataSource::CommunityContributed => 0.60,
        };

        let verification_bonus = if verified { 0.05 } else { 0.0 };
        let source_bonus = (num_sources.saturating_sub(1) as f32) * 0.03;

        (base_confidence + verification_bonus + source_bonus).min(1.0)
    }
}

/// Default license for automotive data
pub fn default_license() -> String {
    "CC-BY-SA-4.0".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_metadata_creation() {
        let metadata = DataMetadata {
            source: DataSource::OfficialDocumentation,
            confidence: 0.95,
            date_added: "2026-03-26".to_string(),
            last_verified: Some("2026-03-26".to_string()),
            last_modified: Some("2026-03-26".to_string()),
            version: "1.0.0".to_string(),
            contributors: vec!["engineer1".to_string()],
            license: default_license(),
            references: vec!["VW Service Manual".to_string()],
            notes: Some("From VW service manual".to_string()),
        };

        assert_eq!(metadata.confidence, 0.95);
        assert_eq!(metadata.source, DataSource::OfficialDocumentation);
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.license, "CC-BY-SA-4.0");
    }

    #[test]
    fn test_data_source_variants() {
        let sources = vec![
            DataSource::OfficialDocumentation,
            DataSource::ReverseEngineered,
            DataSource::CommunityContributed,
            DataSource::ServiceManual,
            DataSource::HardwareTested,
            DataSource::MultiSourceVerified,
            DataSource::DiagnosticTool,
        ];

        assert_eq!(sources.len(), 7);
    }

    #[test]
    fn test_calculate_confidence() {
        let conf1 = DataMetadata::calculate_confidence(&DataSource::OfficialDocumentation, false, 1);
        assert_eq!(conf1, 0.95);

        let conf2 = DataMetadata::calculate_confidence(&DataSource::OfficialDocumentation, true, 1);
        assert_eq!(conf2, 1.0);

        let conf3 = DataMetadata::calculate_confidence(&DataSource::CommunityContributed, false, 3);
        assert_eq!(conf3, 0.66);
    }

    #[test]
    fn test_default_license() {
        assert_eq!(default_license(), "CC-BY-SA-4.0");
    }
}
