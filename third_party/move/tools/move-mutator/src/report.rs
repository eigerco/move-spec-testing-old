use serde::Serialize;
use serde_json;
use std::io::Write;

/// The `Report` struct represents a report of mutations.
/// It contains a vector of `ReportEntry` instances.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// The vector of `ReportEntry` instances.
    mutations: Vec<ReportEntry>,
}

impl Report {
    /// Creates a new `Report` instance.
    pub fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    /// Adds a new `ReportEntry` to the report.
    pub fn add_entry(&mut self, entry: ReportEntry) {
        self.mutations.push(entry);
    }

    /// Saves the `Report` as a JSON file.
    pub fn save_to_json_file(&self, path: &str) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, &self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Saves the `Report` as a text file.
    pub fn save_to_text_file(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        for entry in &self.mutations {
            writeln!(file, "File: {}", entry.file)?;
            writeln!(file, "Original file: {}", entry.original_file)?;
            writeln!(file, "Modifications:")?;
            for modification in &entry.modifications {
                writeln!(file, "  Operator: {}", modification.operator_name)?;
                writeln!(file, "  Old value: {}", modification.old_value)?;
                writeln!(file, "  New value: {}", modification.new_value)?;
                writeln!(file, "  Changed place: {}-{}", modification.changed_place.start, modification.changed_place.end)?;
            }
            writeln!(file, "Diff:")?;
            writeln!(file, "{}", entry.diff)?;
            writeln!(file, "----------------------------------------")?;
        }
        Ok(())
    }

    /// Converts the `Report` to a JSON string.
    #[cfg(test)]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self)
    }
}

/// The `Range` struct represents a range with a start and end.
/// It is used to represent the location of a mutation inside the source file.
#[derive(Debug, Clone, Serialize)]
pub struct Range {
    /// The start of the range.
    start: usize,
    /// The end of the range.
    end: usize,
}

impl Range {
    /// Creates a new `Range` instance.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// The `Modification` struct represents a modification that was applied to a file.
/// It contains the location of the modification, the name of the mutation operator, the old value and the new value.
/// It is used to represent a single modification inside a `ReportEntry`.
#[derive(Debug, Clone, Serialize)]
pub struct Modification {
    /// The location of the modification.
    changed_place: Range,
    /// The name of the mutation operator.
    operator_name: String,
    /// The old operator value.
    old_value: String,
    /// The new operator value.
    new_value: String,
}

impl Modification {
    /// Creates a new `Modification` instance.
    pub fn new(
        changed_place: Range,
        operator_name: String,
        old_value: String,
        new_value: String,
    ) -> Self {
        Self {
            changed_place,
            operator_name,
            old_value,
            new_value,
        }
    }
}

/// The `ReportEntry` struct represents an entry in a report.
/// It contains information about a mutation that was applied to a file.
#[derive(Debug, Clone, Serialize)]
pub struct ReportEntry {
    /// The path to the mutated file.
    file: String,
    /// The path to the original file.
    original_file: String,
    /// The modifications that were applied to the file.
    modifications: Vec<Modification>,
    /// The diff between the original and mutated file.
    diff: String,
}

impl ReportEntry {
    /// Creates a new `ReportEntry` instance.
    pub fn new(file: String, original_file: String) -> Self {
        Self {
            file,
            original_file,
            modifications: vec![],
            diff: String::new(),
        }
    }

    /// Adds a `Modification` to the `ReportEntry`.
    pub fn add_modification(&mut self, modification: Modification) {
        self.modifications.push(modification);
    }

    /// Generate diff (patch) between the original and mutated source.
    /// The diff is stored in the `ReportEntry`.
    pub fn generate_diff(&mut self, original_source: &str, mutated_source: &str) {
        let patch = diffy::create_patch(original_source, mutated_source);
        self.diff = patch.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_report() {
        let mut report = Report::new();
        assert_eq!(report.to_json().unwrap(), "{\n  \"mutations\": []\n}");

        let range = Range::new(0, 10);
        let modification = Modification::new(
            range,
            "operator".to_string(),
            "old".to_string(),
            "new".to_string(),
        );
        let mut report_entry = ReportEntry::new("file".to_string(), "original_file".to_string());
        report_entry.add_modification(modification);
        report_entry.generate_diff("diff\n", "\n");

        report.add_entry(report_entry.clone());
        assert_eq!(
            report.to_json().unwrap(),
            "{\n  \"mutations\": [\n    {\n      \"file\": \"file\",\n      \"original_file\": \"original_file\",\n      \"modifications\": [\n        {\n          \"changed_place\": {\n            \"start\": 0,\n            \"end\": 10\n          },\n          \"operator_name\": \"operator\",\n          \"old_value\": \"old\",\n          \"new_value\": \"new\"\n        }\n      ],\n      \"diff\": \"--- original\\n+++ modified\\n@@ -1 +1 @@\\n-diff\\n+\\n\"\n    }\n  ]\n}"
        );
    }

    #[test]
    fn test_range() {
        let range = Range::new(0, 10);
        assert_eq!(
            serde_json::to_string(&range).unwrap(),
            "{\"start\":0,\"end\":10}"
        );
    }

    #[test]
    fn test_modification() {
        let range = Range::new(0, 10);
        let modification = Modification::new(
            range,
            "operator".to_string(),
            "old".to_string(),
            "new".to_string(),
        );
        assert_eq!(serde_json::to_string(&modification).unwrap(), "{\"changed_place\":{\"start\":0,\"end\":10},\"operator_name\":\"operator\",\"old_value\":\"old\",\"new_value\":\"new\"}");
    }

    #[test]
    fn saves_report_as_text_file_successfully() {
        let mut report = Report::new();
        let range = Range::new(0, 10);
        let modification = Modification::new(
            range,
            "operator".to_string(),
            "old".to_string(),
            "new".to_string(),
        );
        let mut report_entry = ReportEntry::new("file".to_string(), "original_file".to_string());
        report_entry.add_modification(modification);
        report.add_entry(report_entry);

        let path = "test_report.txt";
        report.save_to_text_file(path).unwrap();

        let mut file = fs::File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert!(contents.contains("File: file"));
        assert!(contents.contains("Original file: original_file"));
        assert!(contents.contains("Modifications:"));
        assert!(contents.contains("Operator: operator"));
        assert!(contents.contains("Old value: old"));
        assert!(contents.contains("New value: new"));
        assert!(contents.contains("Changed place: 0-10"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn fails_to_save_report_to_non_existent_directory() {
        let report = Report::new();
        let path = "non_existent_directory/test_report.txt";
        report.save_to_text_file(path).unwrap();
    }
}
