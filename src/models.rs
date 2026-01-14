use std::fmt::Display;

use chrono::Duration;
use serde::{Deserialize, Deserializer};

mod duration_serde {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            None => Ok(None),
            Some(s) => {
                let duration = iso8601_duration::Duration::parse(&s)
                    .map_err(|e| serde::de::Error::custom(format!("Failed to parse duration: {:?}", e)))?;

                // Convert ISO8601 Duration to chrono::Duration
                let mut total_seconds = 0i64;

                // Handle date components (approximate conversions)
                total_seconds += duration.year as i64 * 365 * 24 * 3600;
                total_seconds += duration.month as i64 * 30 * 24 * 3600;
                total_seconds += duration.day as i64 * 24 * 3600;

                // Handle time components
                total_seconds += duration.hour as i64 * 3600;
                total_seconds += duration.minute as i64 * 60;

                // The second field is a f32 that includes fractional seconds
                let seconds_floor = duration.second.floor() as i64;
                let nanos = ((duration.second - seconds_floor as f32) * 1_000_000_000.0) as i64;
                total_seconds += seconds_floor;

                Ok(Some(
                    Duration::seconds(total_seconds) + Duration::nanoseconds(nanos)
                ))
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Links {
    #[serde(alias = "self")]
    pub self_: String,
    pub xref_export: String,
    pub reconcile: String,
    pub ui: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Collection {
    pub created_at: String,
    pub updated_at: String,
    pub category: String,
    pub frequency: String,
    pub name: String,
    pub collection_id: String,
    pub foreign_id: String,
    pub data_updated_at: String,
    pub label: String,
    pub casefile: bool,
    pub secret: bool,
    pub xref: Option<bool>,
    pub restricted: Option<bool>,
    pub contains_ai: Option<bool>,
    pub taggable: Option<bool>,
    pub id: String,
    pub writeable: bool,
    pub links: Links,
    pub shallow: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Stage {
    pub job_id: String,
    pub stage: String,
    pub finished: u32,
    pub running: u32,
    pub pending: u32,
}

impl Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<10} finished: {:<7} running {:<7} pending {:<7}",
            self.stage, self.finished, self.running, self.pending
        )
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum StageOrStages {
    Stage(Stage),
    Stages(Vec<Stage>),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Task {
    pub todo: u32,
    pub doing: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub aborted: u32,
    pub aborting: u32,
    pub cancelled: u32,
    pub min_ts: Option<String>,
    pub max_ts: Option<String>,
    pub name: String,
    #[serde(deserialize_with = "duration_serde::deserialize")]
    pub remaining_time: Option<Duration>,
    #[serde(deserialize_with = "duration_serde::deserialize")]
    pub took: Option<Duration>,
    pub total: u32,
    pub active: u32,
    pub finished: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Queue {
    pub todo: u32,
    pub doing: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub aborted: u32,
    pub aborting: u32,
    pub cancelled: u32,
    pub min_ts: Option<String>,
    pub max_ts: Option<String>,
    pub name: String,
    pub tasks: Vec<Task>,
    #[serde(deserialize_with = "duration_serde::deserialize")]
    pub remaining_time: Option<Duration>,
    #[serde(deserialize_with = "duration_serde::deserialize")]
    pub took: Option<Duration>,
    pub total: u32,
    pub active: u32,
    pub finished: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Batch {
    pub todo: u32,
    pub doing: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub aborted: u32,
    pub aborting: u32,
    pub cancelled: u32,
    pub min_ts: Option<String>,
    pub max_ts: Option<String>,
    pub name: String,
    pub queues: Vec<Queue>,
    #[serde(deserialize_with = "duration_serde::deserialize")]
    pub remaining_time: Option<Duration>,
    #[serde(deserialize_with = "duration_serde::deserialize")]
    pub took: Option<Duration>,
    pub total: u32,
    pub active: u32,
    pub finished: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StatusResult {
    pub todo: u32,
    pub doing: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub aborted: u32,
    pub aborting: u32,
    pub cancelled: u32,
    pub min_ts: Option<String>,
    pub max_ts: Option<String>,
    pub name: String,
    #[serde(deserialize_with = "duration_serde::deserialize")]
    pub remaining_time: Option<Duration>,
    #[serde(deserialize_with = "duration_serde::deserialize")]
    pub took: Option<Duration>,
    pub total: u32,
    pub active: u32,
    pub finished: u32,
    pub collection: Option<Collection>,
    #[serde(default)]
    pub batches: Vec<Batch>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Status {
    pub results: Vec<StatusResult>,
    pub total: u32,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct MetadataApp {
    pub title: Option<String>,
    pub version: Option<String>,
    pub ftm_version: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Metadata {
    pub status: String,
    pub maintenance: bool,
    pub app: MetadataApp,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn test_status_deserialization() {
        let test = read_to_string("testdata/results.json").unwrap();
        let status: Status = serde_json::from_str(&test).unwrap();

        // Test Status-level fields
        assert_eq!(status.total, 1);
        assert_eq!(status.results.len(), 1);

        let result = &status.results[0];

        // Test StatusResult counter fields
        assert_eq!(result.todo, 6);
        assert_eq!(result.doing, 2);
        assert_eq!(result.succeeded, 2);
        assert_eq!(result.failed, 15);
        assert_eq!(result.aborted, 0);
        assert_eq!(result.aborting, 0);
        assert_eq!(result.cancelled, 0);

        // Test timestamps
        assert_eq!(
            result.min_ts.as_ref().unwrap(),
            "2026-01-13T22:49:04.662300Z"
        );
        assert_eq!(
            result.max_ts.as_ref().unwrap(),
            "2026-01-13T22:52:45.168337Z"
        );

        // Test name and time fields
        assert_eq!(result.name, "collection_1");
        // Verify durations are parsed correctly
        assert!(result.remaining_time.is_some());
        let remaining = result.remaining_time.as_ref().unwrap();
        assert_eq!(remaining.num_seconds(), 77); // 1 minute 17 seconds

        assert!(result.took.is_some());
        let took = result.took.as_ref().unwrap();
        assert_eq!(took.num_seconds(), 220); // 3 minutes 40 seconds

        // Test aggregate counts
        assert_eq!(result.total, 25);
        assert_eq!(result.active, 8);
        assert_eq!(result.finished, 17);

        // Test batches array
        assert_eq!(result.batches.len(), 1);
        let batch = &result.batches[0];
        assert_eq!(batch.name, "4:cfab4858-fe55-4412-bc8a-18bee2087522");
        assert_eq!(batch.total, 25);
        assert_eq!(batch.active, 8);
        assert_eq!(batch.finished, 17);

        // Test queues within batch
        assert_eq!(batch.queues.len(), 3);

        // Test first queue (analyze)
        let analyze_queue = &batch.queues[0];
        assert_eq!(analyze_queue.name, "analyze");
        assert_eq!(analyze_queue.doing, 1);
        assert_eq!(analyze_queue.total, 1);
        assert_eq!(analyze_queue.tasks.len(), 1);
        assert_eq!(analyze_queue.tasks[0].name, "ftm_analyze.tasks.analyze");
        assert!(analyze_queue.tasks[0].took.is_some());
        assert_eq!(analyze_queue.tasks[0].took.as_ref().unwrap().num_seconds(), 1);

        // Test second queue (ingest)
        let ingest_queue = &batch.queues[1];
        assert_eq!(ingest_queue.name, "ingest");
        assert_eq!(ingest_queue.todo, 6);
        assert_eq!(ingest_queue.doing, 1);
        assert_eq!(ingest_queue.succeeded, 1);
        assert_eq!(ingest_queue.failed, 15);
        assert_eq!(ingest_queue.total, 23);
        assert_eq!(ingest_queue.tasks.len(), 1);
        assert_eq!(ingest_queue.tasks[0].name, "ingestors.tasks.ingest");
        assert!(ingest_queue.tasks[0].remaining_time.is_some());
        assert_eq!(
            ingest_queue.tasks[0].remaining_time.as_ref().unwrap().num_seconds(),
            82 // 1 minute 22 seconds
        );

        // Test third queue (openaleph)
        let openaleph_queue = &batch.queues[2];
        assert_eq!(openaleph_queue.name, "openaleph");
        assert_eq!(openaleph_queue.succeeded, 1);
        assert_eq!(openaleph_queue.finished, 1);
        assert_eq!(openaleph_queue.tasks.len(), 1);
        assert_eq!(
            openaleph_queue.tasks[0].name,
            "aleph.procrastinate.tasks.index_entities"
        );

        // Test Collection fields
        let collection = result.collection.as_ref().unwrap();
        assert_eq!(collection.created_at, "2026-01-13T22:48:18.414784");
        assert_eq!(collection.updated_at, "2026-01-13T22:48:18.983032");
        assert_eq!(collection.category, "casefile");
        assert_eq!(collection.frequency, "unknown");
        assert_eq!(collection.name, "a507b8a3ae424a64aefebb295211b4cf");
        assert_eq!(collection.collection_id, "1");
        assert_eq!(collection.foreign_id, "a507b8a3ae424a64aefebb295211b4cf");
        assert_eq!(collection.data_updated_at, "2026-01-13T22:52:38.431089");
        assert_eq!(collection.label, "dumpster");
        assert!(collection.casefile);
        assert!(collection.secret);
        assert_eq!(collection.xref, Some(false));
        assert_eq!(collection.restricted, Some(false));
        assert_eq!(collection.contains_ai, Some(false));
        assert_eq!(collection.taggable, Some(false));
        assert_eq!(collection.id, "1");
        assert!(collection.writeable);
        assert!(collection.shallow);

        // Test Links
        assert_eq!(
            collection.links.self_,
            "http://localhost:8080/api/2/collections/1"
        );
        assert_eq!(
            collection.links.xref_export,
            "http://localhost:8080/api/2/collections/1/xref.xlsx?_authz=%3CAuthz(4)%3E"
        );
        assert_eq!(
            collection.links.reconcile,
            "http://localhost:8080/api/2/collections/1/reconcile"
        );
        assert_eq!(collection.links.ui, "http://localhost:8080/datasets/1");
    }

    #[test]
    #[ignore]
    fn test_deserialization_no_collection() {
        let test: String = read_to_string("testdata/export.json").unwrap();
        let status: Status = serde_json::from_str(&test).unwrap();
        assert!(status.results[0].collection.is_none());
    }

    #[test]
    fn test_metadata_deserialization() {
        let test = read_to_string("testdata/metadata.json").unwrap();
        let meta: Metadata = serde_json::from_str(&test).unwrap();
        assert!(meta.status == "ok");
        assert!(!meta.maintenance);
        assert!(meta.app.title.unwrap() == "OpenAleph");
        assert!(meta.app.version.unwrap() == "5.1.0-rc9");
        assert!(meta.app.ftm_version.unwrap() == "4.3.3");
    }
}
