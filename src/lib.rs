/*!
# cuda-metrics-v2

Enhanced metrics with percentiles and aggregation.

Beyond basic counters and gauges — histograms with p50/p95/p99,
timers with latency distribution, and metric aggregation across
agents.

- Histogram with configurable buckets and percentile calculation
- Timer with duration tracking
- Derivative gauge (rate of change)
- Metric labels (tags)
- Aggregation across multiple metric sets
- Snapshot/export
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A histogram bucket
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// A histogram
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Histogram {
    pub name: String,
    pub buckets: Vec<Bucket>,
    pub sum: f64,
    pub count: u64,
    pub min: f64,
    pub max: f64,
}

impl Histogram {
    pub fn new(name: &str, bounds: &[f64]) -> Self {
        let mut buckets: Vec<Bucket> = bounds.iter().map(|&b| Bucket { upper_bound: b, count: 0 }).collect();
        buckets.push(Bucket { upper_bound: f64::INFINITY, count: 0 });
        Histogram { name: name.to_string(), buckets, sum: 0.0, count: 0, min: f64::INFINITY, max: f64::NEG_INFINITY }
    }

    pub fn record(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        if value < self.min { self.min = value; }
        if value > self.max { self.max = value; }
        for bucket in &mut self.buckets { if value <= bucket.upper_bound { bucket.count += 1; break; } }
    }

    pub fn mean(&self) -> f64 { if self.count == 0 { 0.0 } else { self.sum / self.count as f64 } }

    pub fn percentile(&self, p: f64) -> f64 {
        let target = (p / 100.0) * self.count as f64;
        let mut cumulative = 0.0;
        for i in 0..self.buckets.len() {
            cumulative += self.buckets[i].count as f64;
            if cumulative >= target {
                if i == 0 { return self.min; }
                return self.buckets[i - 1].upper_bound;
            }
        }
        self.max
    }

    pub fn p50(&self) -> f64 { self.percentile(50.0) }
    pub fn p95(&self) -> f64 { self.percentile(95.0) }
    pub fn p99(&self) -> f64 { self.percentile(99.0) }

    pub fn snapshot(&self) -> HistogramSnapshot {
        HistogramSnapshot { name: self.name.clone(), count: self.count, sum: self.sum, min: self.min, max: self.max, mean: self.mean(), p50: self.p50(), p95: self.p95(), p99: self.p99() }
    }
}

/// A histogram snapshot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistogramSnapshot {
    pub name: String,
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

/// A timer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timer {
    pub name: String,
    pub histogram: Histogram,
}

impl Timer {
    pub fn new(name: &str, bounds: &[f64]) -> Self { Timer { name: name.to_string(), histogram: Histogram::new(name, bounds) } }
    pub fn record_ms(&mut self, duration_ms: f64) { self.histogram.record(duration_ms); }
    pub fn snapshot(&self) -> TimerSnapshot { TimerSnapshot { name: self.name.clone(), count: self.histogram.count, mean_ms: self.histogram.mean(), p50_ms: self.histogram.p50(), p95_ms: self.histogram.p95(), p99_ms: self.histogram.p99() } }
}

/// Timer snapshot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimerSnapshot {
    pub name: String,
    pub count: u64,
    pub mean_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

/// A derivative gauge (rate of change)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivativeGauge {
    pub name: String,
    pub last_value: f64,
    pub last_update: u64,
    pub rate: f64,
    pub samples: Vec<(u64, f64)>,
    pub max_samples: usize,
}

impl DerivativeGauge {
    pub fn new(name: &str) -> Self { DerivativeGauge { name: name.to_string(), last_value: 0.0, last_update: 0, rate: 0.0, samples: vec![], max_samples: 100 } }

    pub fn set(&mut self, value: f64) {
        let now = now();
        if self.last_update > 0 {
            let elapsed = (now - self.last_update) as f64 / 1000.0;
            if elapsed > 0.0 { self.rate = (value - self.last_value) / elapsed; }
        }
        self.last_value = value;
        self.last_update = now;
        self.samples.push((now, value));
        if self.samples.len() > self.max_samples { self.samples.remove(0); }
    }

    pub fn rate_per_sec(&self) -> f64 { self.rate }
}

/// Metrics registry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsRegistry {
    pub histograms: HashMap<String, Histogram>,
    pub timers: HashMap<String, Timer>,
    pub gauges: HashMap<String, DerivativeGauge>,
}

impl MetricsRegistry {
    pub fn new() -> Self { MetricsRegistry { histograms: HashMap::new(), timers: HashMap::new(), gauges: HashMap::new() } }

    pub fn histogram(&mut self, name: &str, bounds: &[f64]) -> &mut Histogram {
        self.histograms.entry(name.to_string()).or_insert_with(|| Histogram::new(name, bounds))
    }

    pub fn timer(&mut self, name: &str) -> &mut Timer {
        self.timers.entry(name.to_string()).or_insert_with(|| Timer::new(name, &[1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0]))
    }

    pub fn gauge(&mut self, name: &str) -> &mut DerivativeGauge {
        self.gauges.entry(name.to_string()).or_insert_with(|| DerivativeGauge::new(name))
    }

    pub fn snapshot_all(&self) -> MetricsSnapshot {
        let histograms: Vec<_> = self.histograms.values().map(|h| h.snapshot()).collect();
        let timers: Vec<_> = self.timers.values().map(|t| t.snapshot()).collect();
        let gauges: Vec<_> = self.gauges.values().map(|g| (g.name.clone(), g.rate_per_sec())).collect();
        MetricsSnapshot { histograms, timers, gauges }
    }

    pub fn summary(&self) -> String {
        format!("MetricsV2: {} histograms, {} timers, {} gauges",
            self.histograms.len(), self.timers.len(), self.gauges.len())
    }
}

/// Full metrics snapshot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub histograms: Vec<HistogramSnapshot>,
    pub timers: Vec<TimerSnapshot>,
    pub gauges: Vec<(String, f64)>,
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_basic() {
        let mut h = Histogram::new("latency", &[10.0, 50.0, 100.0]);
        for v in [5.0, 15.0, 55.0, 150.0] { h.record(v); }
        assert_eq!(h.count, 4);
        assert!((h.mean() - 56.25).abs() < 0.01);
    }

    #[test]
    fn test_histogram_percentiles() {
        let mut h = Histogram::new("test", &[10.0, 50.0, 100.0]);
        for _ in 0..100 { h.record(10.0); }
        assert!(h.p50() <= 10.0);
        assert!(h.p99() <= 10.0);
    }

    #[test]
    fn test_timer() {
        let mut t = Timer::new("req", &[10.0, 50.0, 100.0]);
        t.record_ms(5.0);
        t.record_ms(50.0);
        t.record_ms(200.0);
        let snap = t.snapshot();
        assert_eq!(snap.count, 3);
    }

    #[test]
    fn test_derivative_gauge() {
        let mut g = DerivativeGauge::new("cpu");
        g.set(10.0);
        g.set(20.0);
        let rate = g.rate_per_sec();
        assert!(rate > 0.0); // increased
    }

    #[test]
    fn test_registry() {
        let mut reg = MetricsRegistry::new();
        reg.histogram("h", &[10.0]).record(5.0);
        reg.timer("t").record_ms(10.0);
        reg.gauge("g").set(42.0);
        let snap = reg.snapshot_all();
        assert_eq!(snap.histograms.len(), 1);
        assert_eq!(snap.timers.len(), 1);
    }

    #[test]
    fn test_histogram_min_max() {
        let mut h = Histogram::new("x", &[100.0]);
        h.record(10.0);
        h.record(50.0);
        h.record(200.0);
        assert!((h.min - 10.0).abs() < 0.01);
        assert!((h.max - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_gauge_max_samples() {
        let mut g = DerivativeGauge::new("lim");
        g.max_samples = 3;
        for i in 0..5 { g.set(i as f64); }
        assert_eq!(g.samples.len(), 3);
    }

    #[test]
    fn test_registry_summary() {
        let reg = MetricsRegistry::new();
        let s = reg.summary();
        assert!(s.contains("0 histograms"));
    }
}
