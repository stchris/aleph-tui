use std::fmt::Display;

use serde::Deserialize;

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
    pub languages: Vec<String>,
    pub name: String,
    pub collection_id: String,
    pub data_updated_at: String,
    pub label: String,
    pub casefile: bool,
    pub secret: bool,
    pub xref: Option<bool>,
    pub restricted: Option<bool>,
    pub contains_ai: Option<bool>,
    pub taggable: bool,
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

#[derive(Clone, Debug, Deserialize)]
pub struct Stats {
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
    pub remaining_time: String,
    pub took: String,
    pub total: u32,
    pub active: u32,
    pub finished: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StatusResult {
    #[serde(flatten)]
    pub stats: Stats,
    pub batches: Vec<StatusBatch>,
    pub collection: Collection,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StatusBatch {
    #[serde(flatten)]
    pub stats: Stats,
    pub queues: Vec<StatusQueue>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StatusQueue {
    #[serde(flatten)]
    pub stats: Stats,
    pub tasks: Vec<Stats>,
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
        let _: Status = serde_json::from_str(&test).unwrap();
    }

    #[test]
    fn test_status_400_deserialization() {
        let test = read_to_string("testdata/results400.json").unwrap();
        let status: Status = serde_json::from_str(&test).unwrap();
        // let stage = status.results[0].stages.as_ref().unwrap();
        // if let StageOrStages::Stages(stages) = stage {
        //     assert!(stages[0].stage == "exportsearch")
        // } else {
        //     panic!("Unexpected stage")
        // }
    }

    #[test]
    fn test_deserialization_no_collection() {
        let test: String = read_to_string("testdata/export.json").unwrap();
        let status: Status = serde_json::from_str(&test).unwrap();
        // assert!(status.results[0].collection.is_none());
    }

    #[test]
    fn test_metadata_deserialization() {
        let test = read_to_string("testdata/metadata.json").unwrap();
        let meta: Metadata = serde_json::from_str(&test).unwrap();
        assert!(meta.status == "ok");
        assert!(!meta.maintenance);
        assert!(meta.app.title.unwrap() == "OCCRP Aleph");
        assert!(meta.app.version.unwrap() == "3.15.5");
        assert!(meta.app.ftm_version.unwrap() == "3.5.8");
    }

    #[test]
    fn test_openaleph() {
        let test = read_to_string("testdata/openaleph.json").unwrap();
        let status: Status = serde_json::from_str(&test).unwrap();
        assert!(status.results[0].batches[0].queues[0].name == "analyze");
        panic!("NAH")
    }
}
