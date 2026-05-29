use crate::types::{FusedEvent, FusionRule, ShadowRef};
use crate::priority::PriorityQueue;
use crate::classifier::EventClassifier;

const TEMPORAL_WINDOW_MS: u64 = 500;

/// The cross-sense fusion engine.
pub struct Correlator {
    shadows: Vec<ShadowRef>,
    rules: Vec<FusionRule>,
    pending: Vec<FusedEvent>,
    priority_queue: PriorityQueue,
    now_fn: Box<dyn Fn() -> u64>,
}

impl Correlator {
    pub fn new() -> Self {
        Self {
            shadows: Vec::new(),
            rules: Vec::new(),
            pending: Vec::new(),
            priority_queue: PriorityQueue::new(),
            now_fn: Box::new(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            }),
        }
    }

    pub fn with_now_fn(now_fn: Box<dyn Fn() -> u64>) -> Self {
        Self {
            shadows: Vec::new(),
            rules: Vec::new(),
            pending: Vec::new(),
            priority_queue: PriorityQueue::new(),
            now_fn,
        }
    }

    pub fn ingest_shadow(&mut self, shadow: ShadowRef) -> Vec<FusedEvent> {
        self.shadows.push(shadow);
        self.tick()
    }

    pub fn add_rule(&mut self, rule: FusionRule) {
        self.rules.push(rule);
    }

    pub fn pending_events(&self) -> Vec<&FusedEvent> {
        self.pending.iter().collect()
    }

    pub fn flush_older_than(&mut self, age_ms: u64) -> Vec<FusedEvent> {
        let now = (self.now_fn)();
        let cutoff = now.saturating_sub(age_ms);
        let (old, recent) = self.pending.drain(..).partition(|e| e.timestamp < cutoff);
        self.pending = recent;
        old
    }

    /// Process temporal windows using the latest shadow timestamp as reference.
    pub fn tick(&mut self) -> Vec<FusedEvent> {
        let mut new_events = Vec::new();

        if self.shadows.len() < 2 {
            return new_events;
        }

        // Use the newest shadow's timestamp as "now" for windowing
        let now = self.shadows.iter().map(|s| s.timestamp).max().unwrap_or(0);
        let window_start = now.saturating_sub(TEMPORAL_WINDOW_MS);

        // Remove expired shadows (older than window relative to newest)
        self.shadows.retain(|s| s.timestamp >= window_start);

        if self.shadows.len() < 2 {
            return new_events;
        }

        // Group shadows by location
        let mut groups: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
        for (i, s) in self.shadows.iter().enumerate() {
            groups.entry(s.location.clone()).or_default().push(i);
        }

        let mut fused_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for (_location, indices) in &groups {
            if indices.len() < 2 {
                continue;
            }
            for i in 0..indices.len() {
                for j in (i + 1)..indices.len() {
                    let ai = indices[i];
                    let aj = indices[j];
                    if fused_indices.contains(&ai) || fused_indices.contains(&aj) {
                        continue;
                    }
                    let a = &self.shadows[ai];
                    let b = &self.shadows[aj];
                    if a.source == b.source {
                        continue;
                    }
                    let time_diff = if a.timestamp > b.timestamp {
                        a.timestamp - b.timestamp
                    } else {
                        b.timestamp - a.timestamp
                    };
                    if time_diff <= TEMPORAL_WINDOW_MS {
                        fused_indices.insert(ai);
                        fused_indices.insert(aj);

                        let shadows = vec![a.clone(), b.clone()];
                        let shadow_refs: Vec<(&str, &str)> = shadows
                            .iter()
                            .map(|s| (s.source.as_str(), s.text.as_str()))
                            .collect();

                        let mut rule_matched = false;
                        for rule in &self.rules {
                            if rule.matches(&shadow_refs) {
                                let confidence = rule.min_confidence.max(0.8);
                                let event = FusedEvent::new(
                                    shadows.clone(),
                                    &rule.output_event,
                                    confidence,
                                    rule.severity,
                                )
                                .with_suggested_action(&format!("Action for: {}", rule.output_event));
                                self.pending.push(event.clone());
                                self.priority_queue.push(event.clone());
                                new_events.push(event);
                                rule_matched = true;
                            }
                        }

                        if !rule_matched {
                            let assessment = format!(
                                "Fused event from {} and {} at {}",
                                a.source, b.source, a.location
                            );
                            let confidence = 0.7;
                            let severity = EventClassifier::classify(&assessment);
                            let event = FusedEvent::new(shadows, &assessment, confidence, severity);
                            self.pending.push(event.clone());
                            self.priority_queue.push(event.clone());
                            new_events.push(event);
                        }
                    }
                }
            }
        }

        let fused = fused_indices;
        let mut new_shadows = Vec::new();
        for (i, s) in self.shadows.drain(..).enumerate() {
            if !fused.contains(&i) {
                new_shadows.push(s);
            }
        }
        self.shadows = new_shadows;

        new_events
    }

    pub fn priority_queue(&mut self) -> &mut PriorityQueue {
        &mut self.priority_queue
    }
}
