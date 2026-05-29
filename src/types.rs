use std::fmt;

/// A fusion rule for the rules engine.
#[derive(Debug, Clone)]
pub struct FusionRule {
    pub condition: String,
    pub output_event: String,
    pub min_confidence: f64,
    pub severity: Severity,
}

impl FusionRule {
    pub fn new(condition: &str, output_event: &str, min_confidence: f64, severity: Severity) -> Self {
        Self {
            condition: condition.to_string(),
            output_event: output_event.to_string(),
            min_confidence,
            severity,
        }
    }
}


/// Severity level for fused events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Info,
    Warning,
    Alert,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Alert => write!(f, "alert"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

/// Reference to a text shadow from any sense module.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowRef {
    pub source: String,
    pub location: String,
    pub timestamp: u64,
    pub text: String,
}

impl ShadowRef {
    pub fn new(source: &str, location: &str, timestamp: u64, text: &str) -> Self {
        Self {
            source: source.to_string(),
            location: location.to_string(),
            timestamp,
            text: text.to_string(),
        }
    }
}

/// A unified event fused from multiple sense-module shadows.
#[derive(Debug, Clone)]
pub struct FusedEvent {
    pub id: String,
    pub timestamp: u64,
    pub source_shadows: Vec<ShadowRef>,
    pub fused_assessment: String,
    pub confidence: f64,
    pub severity: Severity,
    pub suggested_action: Option<String>,
}

impl FusedEvent {
    pub fn new(
        source_shadows: Vec<ShadowRef>,
        fused_assessment: &str,
        confidence: f64,
        severity: Severity,
    ) -> Self {
        let timestamp = source_shadows
            .iter()
            .map(|s| s.timestamp)
            .max()
            .unwrap_or(0);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp,
            source_shadows,
            fused_assessment: fused_assessment.to_string(),
            confidence,
            severity,
            suggested_action: None,
        }
    }

    pub fn with_suggested_action(mut self, action: &str) -> Self {
        self.suggested_action = Some(action.to_string());
        self
    }
}
