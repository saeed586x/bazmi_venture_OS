//! Observability - monitoring, logging, and auditing for the kernel

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Observability - monitoring, logging, and auditing for the kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observability {
    /// Metrics collection
    metrics: Arc<Mutex<MetricsStore>>,
    /// Logging configuration
    log_config: LogConfig,
    /// Audit trail
    audit_trail: Arc<Mutex<AuditTrail>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub log_level: LogLevel,
    pub log_format: LogFormat,
    pub destinations: Vec<LogDestination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Text,
    Structured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogDestination {
    Stdout,
    File { path: String },
    Remote { url: String, api_key: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStore {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, Histogram>,
    pub summaries: HashMap<String, Summary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub buckets: Vec<HistogramBucket>,
    pub sum: f64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub quantiles: Vec<Quantile>,
    pub sum: f64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantile {
    pub quantile: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub events: Vec<AuditEvent>,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: EventType,
    pub actor: String,
    pub target: String,
    pub action: String,
    pub details: Option<serde_json::Value>,
    pub outcome: EventOutcome,
    pub severity: EventSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    PlanCreated,
    PlanExecuted,
    DecisionMade,
    ValidationPerformed,
    VerificationRun,
    RiskAssessed,
    ToolExecuted,
    SystemEvent,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventOutcome {
    Success,
    Failure,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Observability {
    /// Create a new observability instance
    pub fn new(config: LogConfig) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(MetricsStore {
                counters: HashMap::new(),
                gauges: HashMap::new(),
                histograms: HashMap::new(),
                summaries: HashMap::new(),
            })),
            log_config: config,
            audit_trail: Arc::new(Mutex::new(AuditTrail {
                events: Vec::new(),
                retention_days: 365, // Keep audit events for 1 year
            })),
        }
    }
    
    /// Increment a counter metric
    pub fn increment_counter(&self, name: &str, value: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        let counter = metrics.counters.entry(name.to_string()).or_insert(0);
        *counter += value;
    }
    
    /// Set a gauge metric
    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.gauges.insert(name.to_string(), value);
    }
    
    /// Record a histogram observation
    pub fn observe_histogram(&self, name: &str, value: f64, buckets: &[f64]) {
        let mut metrics = self.metrics.lock().unwrap();
        let histogram = metrics.histograms.entry(name.to_string()).or_insert_with(|| {
            Histogram {
                buckets: buckets.iter().map(|&upper_bound| HistogramBucket {
                    upper_bound,
                    count: 0,
                }).collect(),
                sum: 0.0,
                count: 0,
            }
        });
        
        histogram.sum += value;
        histogram.count += 1;
        
        // Find appropriate bucket and increment
        for bucket in &mut histogram.buckets {
            if value <= bucket.upper_bound {
                bucket.count += 1;
                break;
            }
        }
    }
    
    /// Record a summary observation
    pub fn observe_summary(&self, name: &str, value: f64, quantiles: &[f64]) {
        let mut metrics = self.metrics.lock().unwrap();
        let summary = metrics.summaries.entry(name.to_string()).or_insert_with(|| {
            Summary {
                quantiles: quantiles.iter().map(|&q| Quantile {
                    quantile: q,
                    value: 0.0,
                }).collect(),
                sum: 0.0,
                count: 0,
            }
        });
        
        summary.sum += value;
        summary.count += 1;
        
        // Update quantile values (simplified)
        for quantile in &mut summary.quantiles {
            // In a real implementation, this would maintain proper quantiles
            if summary.count == 1 || value > quantile.value {
                quantile.value = value;
            }
        }
    }
    
    /// Log a message
    pub fn log(&self, level: LogLevel, message: &str, details: Option<serde_json::Value>) {
        // Check if log level is enabled
        if self.is_log_level_enabled(&level) {
            let log_entry = LogEntry {
                timestamp: chrono::Utc::now(),
                level,
                message: message.to_string(),
                details,
            };
            
            self.write_log_entry(&log_entry);
        }
    }
    
    /// Check if a log level is enabled
    fn is_log_level_enabled(&self, level: &LogLevel) -> bool {
        match (&self.log_config.log_level, level) {
            (LogLevel::Trace, _) => true,
            (LogLevel::Debug, LogLevel::Trace) => false,
            (LogLevel::Debug, _) => true,
            (LogLevel::Info, LogLevel::Trace | LogLevel::Debug) => false,
            (LogLevel::Info, _) => true,
            (LogLevel::Warn, LogLevel::Trace | LogLevel::Debug | LogLevel::Info) => false,
            (LogLevel::Warn, _) => true,
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Error, _) => false,
        }
    }
    
    /// Write a log entry to configured destinations
    fn write_log_entry(&self, entry: &LogEntry) {
        // Format the log entry
        let formatted = match self.log_config.log_format {
            LogFormat::Json => serde_json::to_string(entry).unwrap_or_else(|_| format!("{:?}", entry)),
            LogFormat::Text => format!("[{}] {:?} - {}", entry.timestamp, entry.level, entry.message),
            LogFormat::Structured => format!("{:?}|{}|{}|{:?}", entry.timestamp, entry.level, entry.message, entry.details),
        };
        
        // Write to destinations
        for destination in &self.log_config.destinations {
            match destination {
                LogDestination::Stdout => {
                    println!("{}", formatted);
                }
                LogDestination::File { path } => {
                    // In a real implementation, write to file
                    // For now, just print to stdout
                    println!("[FILE: {}] {}", path, formatted);
                }
                LogDestination::Remote { url, .. } => {
                    // In a real implementation, send to remote endpoint
                    // For now, just print to stdout
                    println!("[REMOTE: {}] {}", url, formatted);
                }
            }
        }
    }
    
    /// Record an audit event
    pub fn record_audit_event(
        &self,
        event_type: EventType,
        actor: String,
        target: String,
        action: String,
        details: Option<serde_json::Value>,
        outcome: EventOutcome,
        severity: EventSeverity,
    ) {
        let event = AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type,
            actor,
            target,
            action,
            details,
            outcome,
            severity,
        };
        
        let mut audit_trail = self.audit_trail.lock().unwrap();
        audit_trail.events.push(event);
        
        // Clean up old events based on retention policy
        self.cleanup_old_events(&mut audit_trail);
    }
    
    /// Clean up old audit events based on retention policy
    fn cleanup_old_events(&self, audit_trail: &mut AuditTrail) {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(audit_trail.retention_days as i64);
        audit_trail.events.retain(|event| event.timestamp > cutoff_date);
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> MetricsStore {
        self.metrics.lock().unwrap().clone()
    }
    
    /// Get audit trail
    pub fn get_audit_trail(&self) -> AuditTrail {
        self.audit_trail.lock().unwrap().clone()
    }
    
    /// Get audit events by type
    pub fn get_events_by_type(&self, event_type: &EventType) -> Vec<AuditEvent> {
        let audit_trail = self.audit_trail.lock().unwrap();
        audit_trail.events.iter()
            .filter(|event| matches!(&event.event_type, et if et == event_type))
            .cloned()
            .collect()
    }
    
    /// Get audit events by actor
    pub fn get_events_by_actor(&self, actor: &str) -> Vec<AuditEvent> {
        let audit_trail = self.audit_trail.lock().unwrap();
        audit_trail.events.iter()
            .filter(|event| event.actor == actor)
            .cloned()
            .collect()
    }
    
    /// Get audit events by time range
    pub fn get_events_by_time_range(&self, start: &chrono::DateTime<chrono::Utc>, end: &chrono::DateTime<chrono::Utc>) -> Vec<AuditEvent> {
        let audit_trail = self.audit_trail.lock().unwrap();
        audit_trail.events.iter()
            .filter(|event| &event.timestamp >= start && &event.timestamp <= end)
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl Default for Observability {
    fn default() -> Self {
        Self::new(LogConfig {
            log_level: LogLevel::Info,
            log_format: LogFormat::Json,
            destinations: vec![LogDestination::Stdout],
        })
    }
}

// Convenience macros for logging
impl Observability {
    /// Log a trace message
    pub fn trace(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Trace, message, details);
    }
    
    /// Log a debug message
    pub fn debug(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Debug, message, details);
    }
    
    /// Log an info message
    pub fn info(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Info, message, details);
    }
    
    /// Log a warning message
    pub fn warn(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Warn, message, details);
    }
    
    /// Log an error message
    pub fn error(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Error, message, details);
    }
}
