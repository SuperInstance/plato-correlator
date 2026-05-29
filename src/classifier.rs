use crate::types::{FusedEvent, Severity};

/// Classifies fused events by severity.
#[derive(Debug, Default)]
pub struct EventClassifier;

impl EventClassifier {
    pub fn new() -> Self {
        Self
    }

    /// Classify based on keywords in the assessment text.
    pub fn classify(assessment: &str) -> Severity {
        let lower = assessment.to_lowercase();
        if lower.contains("critical") || lower.contains("emergency") || lower.contains("fire") {
            Severity::Critical
        } else if lower.contains("alert") || lower.contains("urgent") || lower.contains("intruder") {
            Severity::Alert
        } else if lower.contains("warning") || lower.contains("caution") || lower.contains("unusual") {
            Severity::Warning
        } else {
            Severity::Info
        }
    }

    /// Re-classify an existing event.
    pub fn reclassify(event: &mut FusedEvent) {
        event.severity = Self::classify(&event.fused_assessment);
    }
}
