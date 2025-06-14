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
    use ged_io::types::event::HasEvents;
    use ged_io::Gedcom;

    #[test]
    fn parses_basic_gedcom() {
        let simple_ged: String = read_relative("./tests/fixtures/simple.ged");
        // let simple_ged: String = read_relative("./tests/fixtures/allged.ged");
        assert!(simple_ged.len() > 0);

        let mut doc = Gedcom::new(simple_ged.chars());
        let data = doc.parse_document();
        assert_eq!(data.individuals.len(), 3);
        assert_eq!(data.families.len(), 1);
        assert_eq!(data.submitters.len(), 1);

        let header = data.header.unwrap();

        // header
        assert_eq!(header.encoding.unwrap().value.unwrap().as_str(), "ASCII");
        assert_eq!(header.submitter_tag.unwrap().as_str(), "@SUBMITTER@");
        assert_eq!(header.gedcom.unwrap().version.unwrap(), "5.5");

        // names
        assert_eq!(
            data.individuals[0]
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
            data.submitters[0]
                .address
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "Submitters address\naddress continued here"
        );

        // events
        let events = data.families[0].events();
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
        assert!(simple_ged.len() > 0);

        let mut doc = Gedcom::new(simple_ged.chars());
        let data = doc.parse_document();
        assert_eq!(data.individuals.len(), 538);
        assert_eq!(data.families.len(), 278);
        // assert_eq!(data.submitters.len(), 0);

        let header = data.header.unwrap();

        // header
        assert_eq!(header.encoding.unwrap().value.unwrap().as_str(), "UTF-8");
        // assert_eq!(header.submitter_tag.unwrap().as_str(), "@SUBMITTER@");
        assert_eq!(header.gedcom.unwrap().version.unwrap(), "5.5.1");

        // names
        assert_eq!(
            data.individuals[0]
                .name
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "George /Washington/"
        );

        // events
        let events = data.families[0].events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.to_string(), "Marriage");
        assert_eq!(
            events[0].date.as_ref().unwrap().value.as_ref().unwrap(),
            "6 MAR 1730"
        );
    }
}
