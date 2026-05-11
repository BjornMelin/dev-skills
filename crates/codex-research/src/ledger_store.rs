use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub(crate) enum LedgerRecord {
    #[serde(rename = "source")]
    Source(SourceRecord),
    #[serde(rename = "claim")]
    Claim(ClaimRecord),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SourceRecord {
    pub(crate) id: String,
    pub(crate) provider: String,
    pub(crate) url: String,
    pub(crate) title: Option<String>,
    pub(crate) route: Option<String>,
    pub(crate) fetched_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ClaimRecord {
    pub(crate) id: String,
    pub(crate) text: String,
    pub(crate) confidence: f32,
    pub(crate) sources: Vec<String>,
    pub(crate) note: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

pub(crate) fn append_ledger_record(path: &Path, record: &LedgerRecord) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, record)?;
    file.write_all(b"\n")?;
    Ok(())
}

pub(crate) fn read_ledger_records(path: &Path) -> Result<Vec<LedgerRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(serde_json::from_str(&line)?);
    }
    Ok(records)
}
