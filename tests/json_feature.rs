mod common;

#[cfg(test)]
#[cfg(feature = "json")]
mod json_feature_tests {
    use crate::common::util::read_relative;
    use ged_io::Gedcom;

    #[test]
    fn test_serializes_document_without_xref_registry() {
        let gedcom_content: String = read_relative("./tests/fixtures/simple.ged");
        let mut parser = Gedcom::new(gedcom_content.chars()).unwrap();
        let data = parser.parse_data().unwrap();

        let json = serde_json::to_string_pretty(&data).unwrap();

        assert!(json.contains("\"individuals\""));
        assert!(json.contains("\"families\""));
        assert!(json.contains("@FATHER@"));
        assert!(!json.contains("xrefs"));
    }

    #[test]
    fn test_serde_entire_gedcom_tree() {
        let gedcom_content: String = read_relative("./tests/fixtures/simple.ged");
        let mut parser = Gedcom::new(gedcom_content.chars()).unwrap();
        let data = parser.parse_data().unwrap();

        // Verify header can be serialized
        let header_json = serde_json::to_string_pretty(&data.header).unwrap();
        assert!(header_json.contains("gedcom"));
        assert!(header_json.contains("5.5"));

        // Verify families can be serialized
        let families: Vec<_> = data.iter_families().collect();
        let families_json = serde_json::to_string_pretty(&families).unwrap();
        assert!(families_json.contains("@FAMILY@"));
        assert!(families_json.contains("@FATHER@"));
        assert!(families_json.contains("@MOTHER@"));
        assert!(families_json.contains("@CHILD@"));
        assert!(families_json.contains("Marriage"));
        assert!(families_json.contains("1 APR 1950"));
        assert!(families_json.contains("marriage place"));

        // Verify individuals can be serialized
        let individuals: Vec<_> = data.iter_individuals().collect();
        let individuals_json = serde_json::to_string_pretty(&individuals).unwrap();
        assert!(individuals_json.contains("@FATHER@"));
        assert!(individuals_json.contains("/Father/"));
        assert!(individuals_json.contains("Male"));
        assert!(individuals_json.contains("Birth"));
        assert!(individuals_json.contains("1 JAN 1899"));
        assert!(individuals_json.contains("birth place"));
    }
}
