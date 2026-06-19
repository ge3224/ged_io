#[cfg(test)]
pub mod util {
    use std::path::PathBuf;
    pub fn read_relative(path: &str) -> String {
        let path_buf: PathBuf = PathBuf::from(path);
        let absolute_path: PathBuf = std::fs::canonicalize(path_buf).unwrap();
        std::fs::read_to_string(absolute_path).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::util::read_relative;
    use ged_io::Gedcom;

    #[test]
    fn parses_basic_gedcom() {
        let simple_ged: String = read_relative("./tests/fixtures/simple.ged");
        // let simple_ged: String = read_relative("./tests/fixtures/allged.ged");
        assert!(!simple_ged.is_empty());

        let mut doc = Gedcom::new(simple_ged.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        assert_eq!(data.count_individual(), 3);
        assert_eq!(data.count_family(), 1);
        assert_eq!(data.count_submitter(), 1);

        let header = data.header.as_ref().unwrap();

        // header
        assert_eq!(
            header
                .encoding
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap()
                .as_str(),
            "ASCII"
        );
        assert_eq!(
            header.submitter_tag.as_ref().unwrap().as_str(),
            "@SUBMITTER@"
        );
        assert_eq!(
            header.gedcom.as_ref().unwrap().version.as_ref().unwrap(),
            "5.5"
        );

        // names
        assert_eq!(
            data.find_individual("@FATHER@")
                .unwrap()
                .name
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "/Father/"
        );

        // addresses
        assert_eq!(
            data.find_submitter("@SUBMITTER@")
                .unwrap()
                .address
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "Submitters address\naddress continued here"
        );

        // events
        let events = data.find_family("@FAMILY@").unwrap().events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.to_string(), "Marriage");
        assert_eq!(
            events[0].date.as_ref().unwrap().value.as_ref().unwrap(),
            "1 APR 1950"
        );
    }

    #[test]
    fn parses_basic_washington_doc() {
        let simple_ged: String = read_relative("./tests/fixtures/washington.ged");
        assert!(!simple_ged.is_empty());

        let mut doc = Gedcom::new(simple_ged.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        assert_eq!(data.count_individual(), 538);
        assert_eq!(data.count_family(), 278);
        // assert_eq!(data.submitter_count(), 0);

        let header = data.header.as_ref().unwrap();

        // header
        assert_eq!(
            header
                .encoding
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap()
                .as_str(),
            "UTF-8"
        );
        // assert_eq!(header.submitter_tag.as_ref().unwrap().as_str(), "@SUBMITTER@");
        assert_eq!(
            header.gedcom.as_ref().unwrap().version.as_ref().unwrap(),
            "5.5.1"
        );

        // names
        assert_eq!(
            data.find_individual("@I1@")
                .unwrap()
                .name
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "George /Washington/"
        );

        // events
        let events = data.find_family("@F1@").unwrap().events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.to_string(), "Marriage");
        assert_eq!(
            events[0].date.as_ref().unwrap().value.as_ref().unwrap(),
            "6 MAR 1730"
        );
    }
}
