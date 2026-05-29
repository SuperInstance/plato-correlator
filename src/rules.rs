use crate::types::FusionRule;

/// Simple pattern-based rules engine.
impl FusionRule {
    /// Check if a rule's condition pattern matches a combination of shadow texts.
    /// Condition format: "source1.pattern1 AND source2.pattern2"
    pub fn matches(&self, shadow_texts: &[(&str, &str)]) -> bool {
        let parts: Vec<&str> = self.condition.split(" AND ").collect();
        for part in parts {
            let trimmed = part.trim();
            let matched = shadow_texts
                .iter()
                .any(|(source, text)| {
                    let prefix = format!("{}.", source);
                    if let Some(rest) = trimmed.strip_prefix(&prefix) {
                        text.contains(rest)
                    } else {
                        text.contains(trimmed)
                    }
                });
            if !matched {
                return false;
            }
        }
        true
    }
}
