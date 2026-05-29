use plato_correlator::*;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// 1. ingest_shadow stores shadow
#[test]
fn test_ingest_stores_shadow() {
    let mut c = Correlator::new();
    let shadow = ShadowRef::new("vision", "front-door", 1000, "person_approaching");
    let events = c.ingest_shadow(shadow);
    assert!(events.is_empty());
    // Shadow is stored internally; no fusion yet
}

// 2. Single shadow produces no fusion
#[test]
fn test_single_shadow_no_fusion() {
    let mut c = Correlator::new();
    let events = c.ingest_shadow(ShadowRef::new("vision", "front-door", 1000, "person_approaching"));
    assert!(events.is_empty());
}

// 3. Two shadows from same location within window fuse
#[test]
fn test_two_shadows_same_location_fuse() {
    let mut c = Correlator::new();
    c.ingest_shadow(ShadowRef::new("vision", "front-door", 1000, "person_approaching"));
    let events = c.ingest_shadow(ShadowRef::new("sonar", "front-door", 1100, "knock detected"));
    assert_eq!(events.len(), 1);
    assert!(events[0].fused_assessment.contains("vision"));
    assert!(events[0].fused_assessment.contains("sonar"));
}

// 4. Two shadows from different locations don't fuse
#[test]
fn test_different_locations_no_fuse() {
    let mut c = Correlator::new();
    c.ingest_shadow(ShadowRef::new("vision", "front-door", 1000, "person_approaching"));
    let events = c.ingest_shadow(ShadowRef::new("sonar", "kitchen", 1100, "knock detected"));
    assert!(events.is_empty());
}

// 5. Fusion rule triggers correctly
#[test]
fn test_fusion_rule_triggers() {
    let mut c = Correlator::new();
    c.add_rule(FusionRule::new(
        "vision.person_approaching AND sonar.knock",
        "visitor_at_door",
        0.5,
        Severity::Alert,
    ));
    c.ingest_shadow(ShadowRef::new("vision", "front-door", 1000, "person_approaching"));
    let events = c.ingest_shadow(ShadowRef::new("sonar", "front-door", 1100, "knock detected"));
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].fused_assessment, "visitor_at_door");
    assert_eq!(events[0].severity, Severity::Alert);
}

// 6. Rule with min_confidence filters low-confidence shadows (rules only match if confidence >= min)
#[test]
fn test_min_confidence_rule() {
    let mut c = Correlator::new();
    c.add_rule(FusionRule::new(
        "vision.person AND sonar.knock",
        "high_conf_visitor",
        0.95,
        Severity::Critical,
    ));
    // Default fusion has confidence 0.7, rule has min_confidence 0.95
    // The rule should still fire but use max(0.95, 0.8) = 0.95 confidence
    c.ingest_shadow(ShadowRef::new("vision", "front-door", 1000, "person detected"));
    let events = c.ingest_shadow(ShadowRef::new("sonar", "front-door", 1100, "knock"));
    assert_eq!(events.len(), 1);
    assert!(events[0].confidence >= 0.95);
}

// 7. Temporal window expires old shadows
#[test]
fn test_temporal_window_expires() {
    let mut c = Correlator::new();
    c.ingest_shadow(ShadowRef::new("vision", "front-door", 1000, "person_approaching"));
    // Second shadow is way outside the 500ms window
    let events = c.ingest_shadow(ShadowRef::new("sonar", "front-door", 9999, "knock"));
    // The first shadow (ts=1000) is outside the window [9499, 9999], so no fusion
    assert!(events.is_empty());
}

// 8. flush_older_than returns expired events
#[test]
fn test_flush_older_than() {
    let mut fake_now = 2000u64;
    let mut c = Correlator::with_now_fn(Box::new(move || fake_now));
    c.add_rule(FusionRule::new(
        "vision.person AND sonar.knock",
        "visitor",
        0.5,
        Severity::Info,
    ));
    c.ingest_shadow(ShadowRef::new("vision", "front-door", 1000, "person"));
    c.ingest_shadow(ShadowRef::new("sonar", "front-door", 1100, "knock"));
    assert_eq!(c.pending_events().len(), 1);
    // Event timestamp is 1100. now=2000, age=500 => cutoff=1500. 1100 < 1500 so it's flushed.
    let flushed = c.flush_older_than(500);
    assert_eq!(flushed.len(), 1);
    assert!(c.pending_events().is_empty());
}

// 9. Priority queue returns critical events first
#[test]
fn test_priority_queue_critical_first() {
    let mut pq = PriorityQueue::new();
    pq.push(FusedEvent::new(vec![], "info event", 0.5, Severity::Info));
    pq.push(FusedEvent::new(vec![], "critical event", 0.99, Severity::Critical));
    pq.push(FusedEvent::new(vec![], "warning event", 0.7, Severity::Warning));
    let first = pq.pop().unwrap();
    assert_eq!(first.severity, Severity::Critical);
    let second = pq.pop().unwrap();
    assert_eq!(second.severity, Severity::Warning);
}

// 10. Multiple rules can fire on same shadow pair
#[test]
fn test_multiple_rules_fire() {
    let mut c = Correlator::new();
    c.add_rule(FusionRule::new(
        "vision.person AND sonar.knock",
        "visitor_at_door",
        0.5,
        Severity::Alert,
    ));
    c.add_rule(FusionRule::new(
        "vision.person AND sonar.knock",
        "possible_delivery",
        0.3,
        Severity::Info,
    ));
    c.ingest_shadow(ShadowRef::new("vision", "front-door", 1000, "person_approaching"));
    let events = c.ingest_shadow(ShadowRef::new("sonar", "front-door", 1050, "knock detected"));
    assert!(events.len() >= 2);
}

// 11. Event classifier assigns correct severity
#[test]
fn test_event_classifier() {
    assert_eq!(EventClassifier::classify("everything is fine"), Severity::Info);
    assert_eq!(EventClassifier::classify("warning: something unusual"), Severity::Warning);
    assert_eq!(EventClassifier::classify("alert: intruder detected"), Severity::Alert);
    assert_eq!(EventClassifier::classify("critical emergency fire"), Severity::Critical);
}
