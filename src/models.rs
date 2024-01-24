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
    pub collection_id: String,
    pub foreign_id: String,
    pub data_updated_at: String,
    pub label: String,
    pub casefile: bool,
    pub secret: bool,
    pub xref: Option<bool>,
    pub restricted: Option<bool>,
    pub id: String,
    pub writeable: bool,
    pub links: Links,
    pub shallow: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StatusResult {
    pub finished: u32,
    pub running: u32,
    pub pending: u32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub last_update: Option<String>,
    pub collection: Collection,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Status {
    pub results: Vec<StatusResult>,
    pub total: u32,
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
}
