mod common;

#[cfg(test)]
#[cfg(feature = "json")]
mod json_feature_tests {
    use crate::common::util::read_relative;
    use ged_io::Gedcom;

    #[test]
    fn serde_simple_gedcom_data() {
        // Parse a simple GEDCOM file
        let gedcom_content: String = read_relative("./tests/fixtures/simple.ged");
        let mut parser = Gedcom::new(gedcom_content.chars()).unwrap();
        let data = parser.parse_data().unwrap();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&data).unwrap();

        // Deserialize back
        let deserialized: ged_io::types::GedcomData = serde_json::from_str(&json).unwrap();

        // Verify key data is preserved
        assert_eq!(data.individuals.len(), deserialized.individuals.len());
        assert_eq!(data.families.len(), deserialized.families.len());

        // Check individual names are preserved
        if !data.individuals.is_empty() {
            let original_name = &data.individuals[0].name;
            let deser_name = &deserialized.individuals[0].name;
            assert_eq!(
                original_name.as_ref().map(|n| n.value.clone()),
                deser_name.as_ref().map(|n| n.value.clone())
            );
        }
    }

    #[test]
    fn serde_entire_gedcom_tree() {
        let gedcom_content: String = read_relative("./tests/fixtures/simple.ged");
        let mut parser = Gedcom::new(gedcom_content.chars()).unwrap();
        let data = parser.parse_data().unwrap();

        // Verify header can be serialized
        let header_json = serde_json::to_string_pretty(&data.header).unwrap();
        assert!(header_json.contains("gedcom"));
        assert!(header_json.contains("5.5"));

        // Verify families can be serialized
        let families_json = serde_json::to_string_pretty(&data.families).unwrap();
        assert!(families_json.contains("@FAMILY@"));
        assert!(families_json.contains("@FATHER@"));
        assert!(families_json.contains("@MOTHER@"));
        assert!(families_json.contains("@CHILD@"));
        assert!(families_json.contains("Marriage"));
        assert!(families_json.contains("1 APR 1950"));
        assert!(families_json.contains("marriage place"));

        // Verify individuals can be serialized
        let individuals_json = serde_json::to_string_pretty(&data.individuals).unwrap();
        assert!(individuals_json.contains("@FATHER@"));
        assert!(individuals_json.contains("/Father/"));
        assert!(individuals_json.contains("Male"));
        assert!(individuals_json.contains("Birth"));
        assert!(individuals_json.contains("1 JAN 1899"));
        assert!(individuals_json.contains("birth place"));

        // Test roundtrip - deserialize and verify
        let deserialized: ged_io::types::GedcomData =
            serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        assert_eq!(data.individuals.len(), deserialized.individuals.len());
        assert_eq!(data.families.len(), deserialized.families.len());
    }
}
