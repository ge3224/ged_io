mod common;

#[cfg(test)]
#[cfg(feature = "json")]
mod json_feature_tests {
    use crate::common::util::read_relative;
    use ged_io::{gedcom::Gedcom, types::individual::Name};
    use serde_json;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn serde_simple_gedcom_data() {
        let name = Name {
            value: Some("Gregor Johann /Mendel/".into()),
            given: Some("Gregor Johann".into()),
            surname: Some("Mendel".into()),
            prefix: None,
            surname_prefix: None,
            suffix: None,
            note: None,
            source: Vec::new(),
        };

        assert_tokens(
            &name,
            &[
                Token::Struct {
                    name: "Name",
                    len: 8,
                },
                Token::Str("value"),
                Token::Some,
                Token::String("Gregor Johann /Mendel/"),
                Token::Str("given"),
                Token::Some,
                Token::String("Gregor Johann"),
                Token::Str("surname"),
                Token::Some,
                Token::String("Mendel"),
                Token::Str("prefix"),
                Token::None,
                Token::Str("surname_prefix"),
                Token::None,
                Token::Str("note"),
                Token::None,
                Token::Str("suffix"),
                Token::None,
                Token::Str("source"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn serde_entire_gedcom_tree() {
        let gedcom_content: String = read_relative("./tests/fixtures/simple.ged");
        let mut parser = Gedcom::new(gedcom_content.chars());
        let data = parser.parse();

        assert_eq!(
            serde_json::to_string_pretty(&data.header).unwrap(),
            "{\n  \"gedcom\": {\n    \"version\": \"5.5\",\n    \"form\": \"Lineage-Linked\"\n  },\n  \"encoding\": {\n    \"value\": \"ASCII\",\n    \"version\": null\n  },\n  \"source\": {\n    \"value\": \"ID_OF_CREATING_FILE\",\n    \"version\": null,\n    \"name\": null,\n    \"corporation\": null,\n    \"data\": null\n  },\n  \"destination\": null,\n  \"date\": null,\n  \"submitter_tag\": \"@SUBMITTER@\",\n  \"submission_tag\": null,\n  \"copyright\": null,\n  \"language\": null,\n  \"filename\": null,\n  \"note\": null,\n  \"place\": null,\n  \"custom_data\": []\n}"
        );

        assert_eq!(
            serde_json::to_string_pretty(&data.families).unwrap(),
            "[\n  {\n    \"xref\": \"@FAMILY@\",\n    \"individual1\": \"@FATHER@\",\n    \"individual2\": \"@MOTHER@\",\n    \"family_event\": [],\n    \"children\": [\n      \"@CHILD@\"\n    ],\n    \"num_children\": null,\n    \"change_date\": null,\n    \"events\": [\n      {\n        \"event\": \"Marriage\",\n        \"value\": null,\n        \"date\": {\n          \"value\": \"1 APR 1950\",\n          \"time\": null\n        },\n        \"place\": \"marriage place\",\n        \"note\": null,\n        \"family_link\": null,\n        \"family_event_details\": [],\n        \"event_type\": null,\n        \"citations\": [],\n        \"multimedia\": []\n      }\n    ],\n    \"sources\": [],\n    \"multimedia\": [],\n    \"notes\": [],\n    \"custom_data\": []\n  }\n]"
        );

        assert_eq!(
            serde_json::to_string_pretty(&data.individuals).unwrap(),
            "[\n  {\n    \"xref\": \"@FATHER@\",\n    \"name\": {\n      \"value\": \"/Father/\",\n      \"given\": null,\n      \"surname\": null,\n      \"prefix\": null,\n      \"surname_prefix\": null,\n      \"note\": null,\n      \"suffix\": null,\n      \"source\": []\n    },\n    \"sex\": {\n      \"value\": \"Male\",\n      \"fact\": null,\n      \"sources\": [],\n      \"custom_data\": []\n    },\n    \"families\": [\n      {\n        \"xref\": \"@FAMILY@\",\n        \"family_link_type\": \"Spouse\",\n        \"pedigree_linkage_type\": null,\n        \"child_linkage_status\": null,\n        \"adopted_by\": null,\n        \"note\": null,\n        \"custom_data\": []\n      }\n    ],\n    \"attributes\": [],\n    \"source\": [],\n    \"events\": [\n      {\n        \"event\": \"Birth\",\n        \"value\": null,\n        \"date\": {\n          \"value\": \"1 JAN 1899\",\n          \"time\": null\n        },\n        \"place\": \"birth place\",\n        \"note\": null,\n        \"family_link\": null,\n        \"family_event_details\": [],\n        \"event_type\": null,\n        \"citations\": [],\n        \"multimedia\": []\n      },\n      {\n        \"event\": \"Death\",\n        \"value\": null,\n        \"date\": {\n          \"value\": \"31 DEC 1990\",\n          \"time\": null\n        },\n        \"place\": \"death place\",\n        \"note\": null,\n        \"family_link\": null,\n        \"family_event_details\": [],\n        \"event_type\": null,\n        \"citations\": [],\n        \"multimedia\": []\n      }\n    ],\n    \"multimedia\": [],\n    \"last_updated\": null,\n    \"note\": null,\n    \"change_date\": null,\n    \"custom_data\": []\n  },\n  {\n    \"xref\": \"@MOTHER@\",\n    \"name\": {\n      \"value\": \"/Mother/\",\n      \"given\": null,\n      \"surname\": null,\n      \"prefix\": null,\n      \"surname_prefix\": null,\n      \"note\": null,\n      \"suffix\": null,\n      \"source\": []\n    },\n    \"sex\": {\n      \"value\": \"Female\",\n      \"fact\": null,\n      \"sources\": [],\n      \"custom_data\": []\n    },\n    \"families\": [\n      {\n        \"xref\": \"@FAMILY@\",\n        \"family_link_type\": \"Spouse\",\n        \"pedigree_linkage_type\": null,\n        \"child_linkage_status\": null,\n        \"adopted_by\": null,\n        \"note\": null,\n        \"custom_data\": []\n      }\n    ],\n    \"attributes\": [],\n    \"source\": [],\n    \"events\": [\n      {\n        \"event\": \"Birth\",\n        \"value\": null,\n        \"date\": {\n          \"value\": \"1 JAN 1899\",\n          \"time\": null\n        },\n        \"place\": \"birth place\",\n        \"note\": null,\n        \"family_link\": null,\n        \"family_event_details\": [],\n        \"event_type\": null,\n        \"citations\": [],\n        \"multimedia\": []\n      },\n      {\n        \"event\": \"Death\",\n        \"value\": null,\n        \"date\": {\n          \"value\": \"31 DEC 1990\",\n          \"time\": null\n        },\n        \"place\": \"death place\",\n        \"note\": null,\n        \"family_link\": null,\n        \"family_event_details\": [],\n        \"event_type\": null,\n        \"citations\": [],\n        \"multimedia\": []\n      }\n    ],\n    \"multimedia\": [],\n    \"last_updated\": null,\n    \"note\": null,\n    \"change_date\": null,\n    \"custom_data\": []\n  },\n  {\n    \"xref\": \"@CHILD@\",\n    \"name\": {\n      \"value\": \"/Child/\",\n      \"given\": null,\n      \"surname\": null,\n      \"prefix\": null,\n      \"surname_prefix\": null,\n      \"note\": null,\n      \"suffix\": null,\n      \"source\": []\n    },\n    \"sex\": null,\n    \"families\": [\n      {\n        \"xref\": \"@FAMILY@\",\n        \"family_link_type\": \"Child\",\n        \"pedigree_linkage_type\": null,\n        \"child_linkage_status\": null,\n        \"adopted_by\": null,\n        \"note\": null,\n        \"custom_data\": []\n      }\n    ],\n    \"attributes\": [],\n    \"source\": [],\n    \"events\": [\n      {\n        \"event\": \"Birth\",\n        \"value\": null,\n        \"date\": {\n          \"value\": \"31 JUL 1950\",\n          \"time\": null\n        },\n        \"place\": \"birth place\",\n        \"note\": null,\n        \"family_link\": null,\n        \"family_event_details\": [],\n        \"event_type\": null,\n        \"citations\": [],\n        \"multimedia\": []\n      },\n      {\n        \"event\": \"Death\",\n        \"value\": null,\n        \"date\": {\n          \"value\": \"29 FEB 2000\",\n          \"time\": null\n        },\n        \"place\": \"death place\",\n        \"note\": null,\n        \"family_link\": null,\n        \"family_event_details\": [],\n        \"event_type\": null,\n        \"citations\": [],\n        \"multimedia\": []\n      }\n    ],\n    \"multimedia\": [],\n    \"last_updated\": null,\n    \"note\": null,\n    \"change_date\": null,\n    \"custom_data\": []\n  }\n]"
        );

        // let json_data = serde_json::to_string_pretty(&data.individuals).unwrap();
        // panic!("{:?}", json_data);
    }
}
