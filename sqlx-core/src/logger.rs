use log::Level;
use std::time::{Duration, Instant};

const SLOW_QUERY_THRESHOLD: Duration = Duration::from_secs(1);

pub(crate) struct QueryLogger<'q> {
    sql: &'q str,
    rows: usize,
    start: Instant,
}

impl<'q> QueryLogger<'q> {
    pub(crate) fn new(sql: &'q str) -> Self {
        Self {
            sql,
            rows: 0,
            start: Instant::now(),
        }
    }

    pub(crate) fn increment_rows(&mut self) {
        self.rows += 1;
    }

    pub(crate) fn finish(&self) {
        let elapsed = self.start.elapsed();

        let lvl = if elapsed >= SLOW_QUERY_THRESHOLD {
            Level::Warn
        } else {
            Level::Info
        };

        if lvl <= log::STATIC_MAX_LEVEL && lvl <= log::max_level() {
            let mut summary = parse_query_summary(&self.sql);

            let sql = if summary != self.sql {
                summary.push_str(" …");
                format!(
                    "\n\n{}\n",
                    sqlformat::format(
                        &self.sql,
                        &sqlformat::QueryParams::None,
                        sqlformat::FormatOptions::default()
                    )
                )
            } else {
                String::new()
            };

            let rows = self.rows;

            log::logger().log(
                &log::Record::builder()
                    .args(format_args!(
                        "{}; rows: {}, elapsed: {:.3?}{}",
                        summary, rows, elapsed, sql
                    ))
                    .level(lvl)
                    .module_path_static(Some("sqlx::query"))
                    .build(),
            );
        }
    }
}

impl<'q> Drop for QueryLogger<'q> {
    fn drop(&mut self) {
        self.finish();
    }
}

fn parse_query_summary(sql: &str) -> String {
    // For now, just take the first 4 words
    sql.split_whitespace()
        .take(4)
        .collect::<Vec<&str>>()
        .join(" ")
}
