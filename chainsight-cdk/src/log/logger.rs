use super::types::{LogLevel, Logger};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::{
    cmp::{max, min},
    collections::HashMap,
};

const DAY_IN_NANOS: u64 = 86400 * 1_000_000_000;

thread_local! {
    static LOGS: std::cell::RefCell<HashMap<u64,Vec<String>>> = std::cell::RefCell::new(HashMap::new());
}

#[derive(Default, Clone)]
pub struct LoggerImpl {
    ctx: String,
}

#[derive(Debug, Default, candid::CandidType, Serialize)]
pub struct TailResponse {
    pub logs: Vec<String>,
    pub range: TailRange,
    pub next_from: Option<TailCursor>,
}

#[derive(Debug, Default, Serialize, candid::CandidType, Deserialize, Clone)]
pub struct TailRange {
    from: Option<TailCursor>,
    to: Option<TailCursor>,
}
#[cfg(test)]
impl TailRange {
    fn new(from: Option<TailCursor>, to: Option<TailCursor>) -> Self {
        Self { from, to }
    }
}

#[derive(Debug, Serialize, candid::CandidType, Deserialize, Clone, Eq, PartialEq)]
pub struct TailCursor {
    pub key: u64,
    pub idx: usize,
}

impl TailCursor {
    fn new(key: u64, idx: usize) -> Self {
        Self { key, idx }
    }
    fn older_than(&self, cursor: &TailCursor) -> bool {
        self.key < cursor.key || (self.key == cursor.key && self.idx < cursor.idx)
    }
    fn older_than_or_eq(&self, cursor: &TailCursor) -> bool {
        self.older_than(cursor) || self == cursor
    }

    const MIN: TailCursor = TailCursor { key: 0, idx: 0 };
    const MAX: TailCursor = TailCursor {
        key: u64::MAX,
        idx: usize::MAX,
    };
}
impl From<u64> for TailCursor {
    fn from(key: u64) -> Self {
        Self {
            key,
            idx: usize::MAX,
        }
    }
}

impl Logger for LoggerImpl {
    fn info(&self, s: &str) {
        self.log(&LogLevel::Info, s, ic_cdk::api::time());
    }

    fn err(&self, err: &Error) {
        self.log(
            &LogLevel::Error,
            &Self::format_err(err),
            ic_cdk::api::time(),
        );
    }

    fn err_with_msg(&self, err: &Error, msg: &str) {
        self.log(
            &LogLevel::Error,
            &Self::format_err_with_msg(err, msg),
            ic_cdk::api::time(),
        );
    }
}

impl LoggerImpl {
    pub fn new(ctx: Option<&str>) -> Self {
        if let Some(ctx) = ctx {
            Self {
                ctx: format!("[{}] ", ctx),
            }
        } else {
            Self::default()
        }
    }

    pub fn drain(&self, rows: usize) -> Vec<String> {
        let exported = self._drain(rows);
        self.info(format!("Drained {} logs.", exported.len()).as_str());
        exported
    }

    pub fn tail(&self, rows: usize, option: Option<TailRange>) -> TailResponse {
        let range = option.unwrap_or_default();
        let to = range.to.clone().unwrap_or(TailCursor::MIN);
        let from = range
            .from
            .as_ref()
            .map(|f| TailCursor::new(f.key, f.idx + 1))
            .unwrap_or(TailCursor::MAX);

        let mut res = Vec::new();
        let mut tailed: TailRange = TailRange::default();
        LOGS.with_borrow(|logs| {
            for key in Self::keys()
                .iter()
                .filter(|k| from.key >= **k && **k >= to.key)
                .rev()
            {
                if res.len() >= rows || tailed.to.as_ref().is_some_and(|t| t.older_than(&to)) {
                    break;
                }
                let logs = logs.get(key).unwrap();

                let idx_to = if from.key == *key {
                    min(from.idx, logs.len())
                } else {
                    logs.len()
                };

                let idx_from = if key == &to.key {
                    max(idx_to.saturating_sub(rows - res.len()), min(idx_to, to.idx))
                } else {
                    idx_to.saturating_sub(rows - res.len())
                };

                let mut logs = logs[idx_from..idx_to].to_vec();
                logs.extend(res.clone());
                res = logs;

                if tailed.from.is_none() {
                    tailed.from = Some(TailCursor::new(*key, idx_to - 1));
                }
                tailed.to = Some(TailCursor::new(*key, idx_from));
            }
        });
        TailResponse {
            logs: res,
            next_from: self.next(&tailed.to, &range.to),
            range: tailed,
        }
    }

    pub fn sweep(&self, retention_days: u8) {
        let until = (ic_cdk::api::time() / DAY_IN_NANOS - retention_days as u64) * DAY_IN_NANOS;
        self._sweep(until);
        self.info(format!("Sweeped logs before {}.", Self::format_timestamp(until)).as_str());
    }

    fn next(&self, cursor: &Option<TailCursor>, to: &Option<TailCursor>) -> Option<TailCursor> {
        if cursor.is_none() {
            return None;
        }
        let cursor = cursor.as_ref().unwrap();
        if to.as_ref().is_some_and(|t| cursor.older_than_or_eq(t)) {
            return None;
        }
        if cursor.idx > 0 {
            return Some(TailCursor::new(cursor.key, cursor.idx - 1));
        }

        let key = Self::keys().into_iter().find(|k| cursor.key > *k)?;
        LOGS.with_borrow(|logs| {
            if let Some(logs) = logs.get(&key) {
                return Some(TailCursor::new(key, logs.len().saturating_sub(1)));
            };
            None
        })
    }

    fn _drain(&self, rows: usize) -> Vec<String> {
        let keys = Self::keys();
        let mut drained = Vec::new();
        for key in keys.iter() {
            if drained.len() >= rows {
                break;
            }
            LOGS.with_borrow_mut(|logs| {
                let logs = logs.get_mut(key).unwrap();
                let to_drain = min(rows - drained.len(), logs.len());
                drained.extend(logs.drain(..to_drain));
            });
        }
        drained
    }

    fn _sweep(&self, until: u64) {
        let keys = Self::keys();
        let key = Self::key(until);
        LOGS.with_borrow_mut(|logs| {
            keys.iter().filter(|k| **k < key).for_each(|k| {
                logs.remove(k);
            });
        });
    }

    fn key(ts: u64) -> u64 {
        ts / (DAY_IN_NANOS)
    }

    fn keys() -> Vec<u64> {
        let mut keys = LOGS.with_borrow(|logs| logs.keys().cloned().collect::<Vec<u64>>());
        keys.sort();
        keys
    }

    fn log(&self, level: &LogLevel, s: &str, ts: u64) {
        let key = Self::key(ts);
        LOGS.with_borrow_mut(|logs| {
            if logs.get(&key).is_none() {
                logs.insert(key, Vec::new());
            }
            let log = logs.get_mut(&key).unwrap();
            log.push(self.format_log(level, s, ts));
        });
    }

    fn format_log(&self, level: &LogLevel, s: &str, ts: u64) -> String {
        format!(
            "[{}]: [{}] {}{}",
            Self::format_timestamp(ts),
            level,
            self.ctx,
            s
        )
    }

    fn format_timestamp(ts: u64) -> String {
        let datetime = time::OffsetDateTime::from_unix_timestamp_nanos(ts.into());
        if datetime.is_err() {
            return format!("{}", ts);
        }
        let datetime = datetime.unwrap();
        let time = datetime.time();
        format!(
            "{} {:02}:{:02}:{:02}.{:09} UTC",
            datetime.date(),
            time.hour(),
            time.minute(),
            time.second(),
            time.nanosecond()
        )
    }

    fn format_err(err: &Error) -> String {
        let bt = err.backtrace();
        if bt.status() == std::backtrace::BacktraceStatus::Captured {
            format!("{}\nstack backtrace:\n{}", err, err.backtrace())
        } else {
            format!("{}", err)
        }
    }

    fn format_err_with_msg(err: &Error, msg: &str) -> String {
        format!("{} err: {}", msg, Self::format_err(err))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_format() {
        let ts = i64::MAX as u64;
        assert_eq!(
            LoggerImpl::default().format_log(&LogLevel::Info, "test", ts),
            "[2262-04-11 23:47:16.854775807 UTC]: [INFO] test"
        );
    }

    #[test]
    fn test_format_with_ctx() {
        let ts = i64::MAX as u64;
        assert_eq!(
            LoggerImpl::new(Some("Test")).format_log(&LogLevel::Info, "test", ts),
            "[2262-04-11 23:47:16.854775807 UTC]: [INFO] [Test] test"
        );
    }

    #[test]
    fn test_format_err() {
        let err = anyhow::anyhow!("test error");
        assert!(LoggerImpl::format_err(&err).contains("test error"));
    }

    #[test]
    fn test_format_err_with_msg() {
        let err = anyhow::anyhow!("test error");
        assert!(LoggerImpl::format_err_with_msg(&err, "msg.").contains("msg. err: test error"));
    }

    #[test]
    fn test_log() {
        let logger = LoggerImpl::default();
        let ts = i64::MAX as u64;
        logger.log(&LogLevel::Error, "test", ts);
        LOGS.with(|log| {
            assert_eq!(
                log.borrow()
                    .get(&LoggerImpl::key(ts))
                    .unwrap()
                    .last()
                    .unwrap(),
                "[2262-04-11 23:47:16.854775807 UTC]: [ERROR] test"
            );
        });
    }

    #[test]
    fn test_drain() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "test", 1);
        logger.log(&LogLevel::Info, "test", 2);
        logger.log(&LogLevel::Info, "test", DAY_IN_NANOS);
        logger.log(&LogLevel::Info, "test", DAY_IN_NANOS + 1);
        logger.log(&LogLevel::Info, "test", DAY_IN_NANOS + 2);

        let logs = logger._drain(3);
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0], "[1970-01-01 00:00:00.000000001 UTC]: [INFO] test");
        assert_eq!(logs[2], "[1970-01-02 00:00:00.000000000 UTC]: [INFO] test");
        LOGS.with_borrow(|logs| {
            let logs_1 = logs.get(&LoggerImpl::key(0)).unwrap();
            assert_eq!(logs_1.len(), 0);
            let logs_2 = logs.get(&LoggerImpl::key(DAY_IN_NANOS)).unwrap();
            assert_eq!(logs_2.len(), 2);
            assert_eq!(
                logs_2[0],
                "[1970-01-02 00:00:00.000000001 UTC]: [INFO] test"
            );
            assert_eq!(
                logs_2[1],
                "[1970-01-02 00:00:00.000000002 UTC]: [INFO] test"
            );
        });
    }

    #[test]
    fn test_drain_overflow() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "test", 1);
        logger.log(&LogLevel::Info, "test", 2);

        let logs = logger._drain(3);
        assert_eq!(logs.len(), 2);
        LOGS.with_borrow(|logs| {
            assert_eq!(logs.len(), 1);
        });
    }

    #[test]
    fn test_tail() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "test", 1);
        logger.log(&LogLevel::Info, "test", 2);
        logger.log(&LogLevel::Info, "test", 3);
        logger.log(&LogLevel::Info, "test", 4);
        logger.log(&LogLevel::Info, "test", 5);
        logger.log(&LogLevel::Info, "test", DAY_IN_NANOS);

        let res = logger.tail(usize::MAX, None);
        assert_eq!(res.logs.len(), 6);
        assert_eq!(
            res.logs[0],
            "[1970-01-01 00:00:00.000000001 UTC]: [INFO] test"
        );
        assert_eq!(
            res.logs[5],
            "[1970-01-02 00:00:00.000000000 UTC]: [INFO] test"
        );
        assert_eq!(res.next_from, None);
        assert_eq!(res.range.from, Some(TailCursor::new(1, 0)));
        assert_eq!(res.range.to, Some(TailCursor::new(0, 0)));
    }

    #[test]
    fn test_tail_with_cursor() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "t", 1);
        logger.log(&LogLevel::Info, "t", 2);
        logger.log(&LogLevel::Info, "t", 3);
        logger.log(&LogLevel::Info, "t", 4);
        logger.log(&LogLevel::Info, "t", 5);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 1);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 2);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 3);

        let res = logger.tail(3, None);
        assert_eq!(res.logs.len(), 3);
        assert_eq!(res.logs[0], "[1970-01-02 00:00:00.000000001 UTC]: [INFO] t");
        assert_eq!(res.logs[2], "[1970-01-02 00:00:00.000000003 UTC]: [INFO] t");
        assert_eq!(res.next_from.clone().unwrap(), TailCursor::new(1, 0));

        let res = logger.tail(3, Some(TailRange::new(res.next_from, None)));
        assert_eq!(res.logs.len(), 3);
        assert_eq!(res.logs[0], "[1970-01-01 00:00:00.000000004 UTC]: [INFO] t");
        assert_eq!(res.logs[2], "[1970-01-02 00:00:00.000000000 UTC]: [INFO] t");
        assert_eq!(res.next_from.clone().unwrap(), TailCursor::new(0, 2));

        let res = logger.tail(4, Some(TailRange::new(res.next_from, None)));
        assert_eq!(res.logs.len(), 3);
        assert_eq!(res.logs[0], "[1970-01-01 00:00:00.000000001 UTC]: [INFO] t");
        assert_eq!(res.logs[2], "[1970-01-01 00:00:00.000000003 UTC]: [INFO] t");
        assert_eq!(res.next_from, None);
    }

    #[test]
    fn test_tail_with_cursor_idempotency() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "t", 1);
        logger.log(&LogLevel::Info, "t", 2);
        logger.log(&LogLevel::Info, "t", 3);
        logger.log(&LogLevel::Info, "t", 4);
        logger.log(&LogLevel::Info, "t", 5);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 1);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 2);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 3);

        let res = logger.tail(3, None);
        assert_eq!(res.logs.len(), 3);
        assert_eq!(res.logs[0], "[1970-01-02 00:00:00.000000001 UTC]: [INFO] t");
        assert_eq!(res.logs[2], "[1970-01-02 00:00:00.000000003 UTC]: [INFO] t");
        assert_eq!(res.next_from.clone().unwrap(), TailCursor::new(1, 0));

        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 4);
        let res = logger.tail(3, Some(TailRange::new(res.next_from, None)));
        assert_eq!(res.logs.len(), 3);
        assert_eq!(res.logs[0], "[1970-01-01 00:00:00.000000004 UTC]: [INFO] t");
        assert_eq!(res.logs[2], "[1970-01-02 00:00:00.000000000 UTC]: [INFO] t");
        assert_eq!(res.next_from.clone().unwrap(), TailCursor::new(0, 2));

        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS * 2);
        let res = logger.tail(4, Some(TailRange::new(res.next_from, None)));
        assert_eq!(res.logs.len(), 3);
        assert_eq!(res.logs[0], "[1970-01-01 00:00:00.000000001 UTC]: [INFO] t");
        assert_eq!(res.logs[2], "[1970-01-01 00:00:00.000000003 UTC]: [INFO] t");
        assert_eq!(res.next_from, None);
    }

    #[test]
    fn test_tail_with_range() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "t", 1);
        logger.log(&LogLevel::Info, "t", 2);
        logger.log(&LogLevel::Info, "t", 3);
        logger.log(&LogLevel::Info, "t", 4);
        logger.log(&LogLevel::Info, "t", 5);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 1);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 2);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS + 3);

        let to = TailCursor::new(0, 2);
        let res = logger.tail(3, None);
        assert_eq!(res.logs.len(), 3);
        assert_eq!(res.logs[0], "[1970-01-02 00:00:00.000000001 UTC]: [INFO] t");
        assert_eq!(res.logs[2], "[1970-01-02 00:00:00.000000003 UTC]: [INFO] t");
        assert_eq!(res.next_from.clone(), Some(TailCursor::new(1, 0)));
        assert_eq!(res.range.from, Some(TailCursor::new(1, 3)));
        assert_eq!(res.range.to, Some(TailCursor::new(1, 1)));

        let res = logger.tail(1, Some(TailRange::new(res.next_from, Some(to.clone()))));
        assert_eq!(res.logs.len(), 1);
        assert_eq!(res.logs[0], "[1970-01-02 00:00:00.000000000 UTC]: [INFO] t");
        assert_eq!(res.next_from.clone(), Some(TailCursor::new(0, 4)));
        assert_eq!(res.range.from, Some(TailCursor::new(1, 0)));
        assert_eq!(res.range.to, Some(TailCursor::new(1, 0)));

        let res = logger.tail(4, Some(TailRange::new(res.next_from, Some(to.clone()))));
        assert_eq!(res.logs.len(), 3);
        assert_eq!(res.logs[0], "[1970-01-01 00:00:00.000000003 UTC]: [INFO] t");
        assert_eq!(res.logs[2], "[1970-01-01 00:00:00.000000005 UTC]: [INFO] t");
        assert_eq!(res.next_from, None);
        assert_eq!(res.range.from, Some(TailCursor::new(0, 4)));
        assert_eq!(res.range.to, Some(TailCursor::new(0, 2)));
    }

    #[test]
    fn test_tail_range_overflow_from() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS * 2);

        let res = logger.tail(3, Some(TailRange::new(Some(TailCursor::new(2, 2)), None)));
        assert_eq!(res.logs.len(), 2);
        assert_eq!(res.next_from, None);
        assert_eq!(res.range.from, Some(TailCursor::new(2, 0)));
        assert_eq!(res.range.to, Some(TailCursor::new(1, 0)));

        let res = logger.tail(3, Some(TailRange::new(Some(TailCursor::new(3, 0)), None)));
        assert_eq!(res.logs.len(), 2);
        assert_eq!(res.next_from, None);
        assert_eq!(res.range.from, Some(TailCursor::new(2, 0)));
        assert_eq!(res.range.to, Some(TailCursor::new(1, 0)));
    }

    #[test]
    fn test_tail_range_overflow_to() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS * 2);

        let to: TailCursor = TailCursor::new(3, 10);
        let res = logger.tail(3, Some(TailRange::new(None, Some(to))));
        assert_eq!(res.logs.len(), 0);
        assert_eq!(res.next_from, None);

        let to = TailCursor::new(1, 10);
        let res = logger.tail(3, Some(TailRange::new(None, Some(to))));
        assert_eq!(res.logs.len(), 1);
        assert_eq!(res.logs[0], "[1970-01-03 00:00:00.000000000 UTC]: [INFO] t");
        assert_eq!(res.next_from, None);

        let to = TailCursor::new(0, 0);
        let res = logger.tail(3, Some(TailRange::new(None, Some(to))));
        assert_eq!(res.logs.len(), 2);
        assert_eq!(res.logs[0], "[1970-01-02 00:00:00.000000000 UTC]: [INFO] t");
        assert_eq!(res.logs[1], "[1970-01-03 00:00:00.000000000 UTC]: [INFO] t");
        assert_eq!(res.next_from, None);
    }

    #[test]
    fn test_tail_invalid_range() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS);
        logger.log(&LogLevel::Info, "t", DAY_IN_NANOS * 2);

        let from = Some(TailCursor::new(4, 9));
        let to = Some(TailCursor::new(4, 10));
        let res = logger.tail(3, Some(TailRange::new(from, to)));
        assert_eq!(res.logs.len(), 0);
        assert_eq!(res.next_from, None);
        assert_eq!(res.range.from, None);
        assert_eq!(res.range.to, None);

        let from = Some(TailCursor::new(4, 10));
        let to = Some(TailCursor::new(4, 10));
        let res = logger.tail(3, Some(TailRange::new(from, to)));
        assert_eq!(res.logs.len(), 0);
        assert_eq!(res.next_from, None);
        assert_eq!(res.range.from, None);
        assert_eq!(res.range.to, None);
    }

    #[test]
    fn test_tail_no_logs() {
        let logger = LoggerImpl::default();
        let res = logger.tail(1, None);
        assert_eq!(res.logs.len(), 0);
        assert_eq!(res.next_from, None);
        assert_eq!(res.range.from, None);
        assert_eq!(res.range.to, None);
    }

    #[test]
    fn test_sweep() {
        let logger = LoggerImpl::default();
        logger.log(&LogLevel::Info, "test", 1);
        logger.log(&LogLevel::Info, "test", 2);
        logger.log(&LogLevel::Info, "test", 3);
        logger.log(&LogLevel::Info, "test", 4);
        logger.log(&LogLevel::Info, "test", 5);
        logger.log(&LogLevel::Info, "test", DAY_IN_NANOS);

        logger._sweep(DAY_IN_NANOS);
        let logs = logger.tail(usize::MAX, None).logs;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], "[1970-01-02 00:00:00.000000000 UTC]: [INFO] test");
    }
}
