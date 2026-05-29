use std::collections::BinaryHeap;
use std::cmp::Ordering;
use crate::types::FusedEvent;

/// Wrapper for priority-based ordering (critical first).
#[derive(Debug, Clone)]
struct PriorityEvent {
    event: FusedEvent,
}

impl PartialEq for PriorityEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.severity == other.event.severity
    }
}

impl Eq for PriorityEvent {}

impl PartialOrd for PriorityEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.event.severity.cmp(&other.event.severity)
    }
}

/// Priority queue for urgent alerts.
#[derive(Debug, Default)]
pub struct PriorityQueue {
    heap: BinaryHeap<PriorityEvent>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: FusedEvent) {
        self.heap.push(PriorityEvent { event });
    }

    pub fn pop(&mut self) -> Option<FusedEvent> {
        self.heap.pop().map(|pe| pe.event)
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn peek(&self) -> Option<&FusedEvent> {
        self.heap.peek().map(|pe| &pe.event)
    }

    pub fn drain_all(&mut self) -> Vec<FusedEvent> {
        let mut events = Vec::with_capacity(self.heap.len());
        while let Some(e) = self.pop() {
            events.push(e);
        }
        events
    }
}
