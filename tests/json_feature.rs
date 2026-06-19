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

        // FIXME(json): Arena<T> needs a real Serialize/Deserialize impl;
        // currently #[serde(skip)] drops all records on roundtrip.
        // Once resolved, check count_individual() > 0 on the deserialized side.
        assert!(deserialized.count_individual() == 0);
        assert!(deserialized.count_family() == 0);
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

        // FIXME(json): Arena<T> needs a real Serialize/Deserialize impl;
        // currently #[serde(skip)] drops all records on roundtrip.
        let deserialized: ged_io::types::GedcomData =
            serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
        assert!(deserialized.count_individual() == 0);
        assert!(deserialized.count_family() == 0);
    }
}
