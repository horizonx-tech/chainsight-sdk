use std::{cell::RefCell, collections::HashMap};

use candid::CandidType;

use crate::time::TimeStamper;
// 7 days
const DATA_POINT_RETENTION_SECONDS: u64 = 7 * 24 * 60 * 60;
// 1 hour
const ONE_DATA_POINT_SECONDS: u64 = 60 * 60;

#[derive(Clone, Debug, CandidType)]
pub struct Metric {
    pub metric_type: MetricType,
    pub value: f64,
}

#[derive(Clone, Debug, CandidType)]
pub enum MetricType {
    TimeMax,
    TimeMin,
    Count,
}

#[derive(Clone, Debug)]
pub struct TaskDuration {
    from_nanosec: u64,
    to_nanosec: u64,
}

impl TaskDuration {
    pub fn new(from_nanosec: u64, to_nanosec: u64) -> Self {
        Self {
            from_nanosec,
            to_nanosec,
        }
    }
}

#[derive(Clone, Debug, CandidType)]
pub struct DataPoint {
    pub metrics: Vec<Metric>,
    pub from_timestamp: u64,
}

impl Default for DataPoint {
    fn default() -> Self {
        Self {
            metrics: vec![
                Metric {
                    metric_type: MetricType::TimeMax,
                    value: 0.0,
                },
                Metric {
                    metric_type: MetricType::TimeMin,
                    value: f64::MAX,
                },
                Metric {
                    metric_type: MetricType::Count,
                    value: 0.0,
                },
            ],
            from_timestamp: TimeStamper::now_sec(),
        }
    }
}
struct MetricCollector {
    data: HashMap<MetricId, Vec<DataPoint>>,
}
type MetricId = String;

impl MetricCollector {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

thread_local! {
    static METRIC_COLLECTOR: RefCell<MetricCollector> = RefCell::new(MetricCollector::new());
}

#[ic_cdk::query]
fn metric_ids() -> Vec<String> {
    _metric_ids()
}

#[ic_cdk::query]
fn metrics(id: String, count: u8) -> Vec<DataPoint> {
    METRIC_COLLECTOR.with(|collector| {
        if let Some(data) = collector.borrow().data.get(&id) {
            return data.iter().rev().take(count as usize).cloned().collect();
        }
        vec![]
    })
}

#[ic_cdk::query]
fn metrics_between(id: String, from: u64, to: u64) -> Vec<DataPoint> {
    METRIC_COLLECTOR.with(|collector| {
        if let Some(data) = collector.borrow().data.get(&id) {
            return data
                .iter()
                .filter(|d| d.from_timestamp >= from && d.from_timestamp <= to)
                .cloned()
                .collect();
        }
        vec![]
    })
}

fn _metric_ids() -> Vec<MetricId> {
    METRIC_COLLECTOR.with(|collector| collector.borrow().data.keys().cloned().collect())
}

fn _enqueue(id: MetricId) {
    let now = TimeStamper::now_sec();
    METRIC_COLLECTOR.with(|collector| {
        let last = _last(id.clone());
        if last.is_none() {
            collector.borrow_mut().data.insert(id, vec![]);
            return;
        }
        let last = last.unwrap();
        if last.from_timestamp + ONE_DATA_POINT_SECONDS < now {
            collector.borrow_mut().data.insert(id, vec![]);
        }
    });
}

fn _insert(id: MetricId, data_point: DataPoint) {
    METRIC_COLLECTOR.with(|collector| {
        let mut collector = collector.borrow_mut();
        if let Some(data) = collector.data.get_mut(&id) {
            data.push(data_point);
        } else {
            collector.data.insert(id, vec![data_point]);
        }
    });
}

fn _dequeue(id: MetricId) {
    METRIC_COLLECTOR.with(|collector| {
        let mut collector = collector.borrow_mut();
        if let Some(data) = collector.data.get_mut(&id) {
            data.remove(0);
        }
    });
}

fn _clean() {
    let ids = _metric_ids();
    let now = TimeStamper::now_sec();

    for id in ids {
        let first = _first(id.clone());
        if first.is_none() {
            continue;
        }
        let first = first.unwrap();
        if first.from_timestamp + DATA_POINT_RETENTION_SECONDS < now {
            _dequeue(id);
        }
    }
}

fn _first(id: MetricId) -> Option<DataPoint> {
    METRIC_COLLECTOR.with(|collector| {
        if let Some(data) = collector.borrow().data.get(&id) {
            return data.first().cloned();
        }
        None
    })
}

fn _last(id: MetricId) -> Option<DataPoint> {
    METRIC_COLLECTOR.with(|collector| {
        if let Some(data) = collector.borrow().data.get(&id) {
            return data.last().cloned();
        }
        None
    })
}
pub fn metric(id: MetricId, duration: TaskDuration) {
    _clean();
    _enqueue(id.clone());
    let mut last = _last(id.clone()).unwrap_or_default();
    let task_dur = (duration.to_nanosec - duration.from_nanosec) as f64;
    let new_metrics = last
        .metrics
        .iter()
        .map(|m| {
            let mut m = m.clone();
            match m.metric_type {
                MetricType::TimeMax => {
                    m.value = m.value.max(task_dur);
                }
                MetricType::TimeMin => {
                    m.value = m.value.min(task_dur);
                }
                MetricType::Count => {
                    m.value += 1.0;
                }
            }
            m
        })
        .collect();
    last.metrics = new_metrics;
    _insert(id, last);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_metric() {
        let id = "test".to_string();
        let duration = TaskDuration::new(0, 100);
        metric(id.clone(), duration);
        let data = _last(id.clone()).unwrap();
        assert_eq!(data.metrics.len(), 3);
        assert_eq!(data.metrics[0].value, 100.0);
        assert_eq!(data.metrics[1].value, 100.0);
        assert_eq!(data.metrics[2].value, 1.0);
        metric(id.clone(), TaskDuration::new(0, 200));
        let data = _last(id.clone()).unwrap();
        assert_eq!(data.metrics.len(), 3);
        assert_eq!(data.metrics[0].value, 200.0);
        assert_eq!(data.metrics[1].value, 100.0);
        assert_eq!(data.metrics[2].value, 2.0);
    }
}
