use super::*;
use arc_swap::ArcSwap;
use itertools::Itertools;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicBool, Arc};

#[derive(Debug, Serialize, Deserialize)]
struct CapabilityEntry {
    cap: ClientCapability,
    values: RwLock<Vec<String>>,
    available: AtomicBool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityRepository {
    supported_caps: Vec<CapabilityEntry>,
    all_caps_301: ArcSwap<String>,
    all_caps_302: ArcSwap<String>,
}

impl CapabilityRepository {
    pub fn new() -> Self {
        let mut supported_caps = Vec::new();

        for &(cap, values) in ClientCapability::all().iter() {
            supported_caps.push(CapabilityEntry {
                cap,
                values: RwLock::new(values.iter().map(|v| v.to_string()).collect()),
                available: AtomicBool::new(cap.is_default()),
            });
        }

        let ret = Self {
            supported_caps,
            all_caps_301: ArcSwap::from_pointee(String::new()),
            all_caps_302: ArcSwap::from_pointee(String::new()),
        };

        ret.update_supported_lists();

        ret
    }

    fn update_supported_lists(&self) {
        let all_caps_301 = self
            .supported_caps
            .iter()
            .filter(|e| e.available.load(Ordering::Relaxed))
            .map(CapabilityEntry::token_301)
            .join(" ");

        let all_caps_302 = self
            .supported_caps
            .iter()
            .filter(|e| e.available.load(Ordering::Relaxed))
            .map(CapabilityEntry::token_302)
            .join(" ");

        self.all_caps_301.store(Arc::new(all_caps_301));
        self.all_caps_302.store(Arc::new(all_caps_302));
    }

    pub fn supported_caps_301(&self) -> Arc<String> {
        self.all_caps_301.load_full()
    }

    pub fn supported_caps_302(&self) -> Arc<String> {
        self.all_caps_302.load_full()
    }

    pub fn find(&self, name: &str) -> Option<ClientCapability> {
        self.supported_caps
            .iter()
            .filter(|e| e.available.load(Ordering::Relaxed))
            .find(|e| e.name() == name)
            .map(|e| e.cap)
    }
    /*
        pub fn enable(&self, cap: ClientCapability)
        {
            for entry in &self.supported_caps
            {
                if entry.cap == cap
                {
                    entry.available.store(true, Ordering::Relaxed);
                }
            }
            self.update_supported_lists();
        }

        pub fn disable(&self, cap: ClientCapability)
        {
            for entry in &self.supported_caps
            {
                if entry.cap == cap
                {
                    entry.available.store(false, Ordering::Relaxed);
                    entry.values.write().clear();
                }
            }
            self.update_supported_lists();
        }
    */
    pub fn enable_with_values(&self, cap: ClientCapability, values: &Vec<String>) {
        for entry in &self.supported_caps {
            if entry.cap == cap {
                entry.available.store(true, Ordering::Relaxed);
                std::mem::swap(entry.values.write().as_mut(), &mut values.clone())
            }
        }
        self.update_supported_lists();
    }
}

impl CapabilityEntry {
    fn token_301(&self) -> String {
        self.cap.name().to_owned()
    }

    fn token_302(&self) -> String {
        let values = self.values.read();

        if values.is_empty() {
            self.cap.name().to_owned()
        } else {
            format!("{}={}", self.cap.name(), values.iter().join(","))
        }
    }

    fn name(&self) -> &str {
        self.cap.name()
    }
}
