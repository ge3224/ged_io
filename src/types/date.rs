pub mod change_date;

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Date encompasses a number of date formats, e.g. approximated, period, phrase and range.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Date {
    pub value: Option<String>,
    pub time: Option<String>,
}

impl Date {
    /// Creates a new `Date` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Date, GedcomError> {
        let mut date = Date::default();
        date.parse(tokenizer, level)?;
        Ok(date)
    }

    /// datetime returns Date and Date.time in a single string.
    ///
    /// # Panics
    ///
    /// Panics when encountering a None value
    #[must_use]
    pub fn datetime(&self) -> Option<String> {
        match &self.time {
            Some(time) => {
                let mut dt = String::new();
                dt.push_str(self.value.as_ref().unwrap().as_str());
                dt.push(' ');
                dt.push_str(time);
                Some(dt)
            }
            None => None,
        }
    }
}

impl Parser for Date {
    /// parse handles the DATE tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        self.value = Some(tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "TIME" => self.time = Some(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::InvalidToken {
                        line: tokenizer.line,
                        token: format!("{:?}", tokenizer.current_token),
                    });
                }
            }
            Ok(())
        };
        parse_subset(tokenizer, level, handle_subset)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_date_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 DATE 2 Oct 2019\n\
            2 TIME 0:00:00\n\
            0 @I1@ INDI\n\
            1 NAME Ancestor\n\
            1 BIRT\n\
            2 DATE BEF 1828\n\
            1 RESI\n\
            2 PLAC 100 Broadway, New York, NY 10005\n\
            2 DATE from 1900 to 1905\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let head_date = data.data.header.unwrap().date.unwrap();
        assert_eq!(head_date.value.unwrap(), "2 Oct 2019");

        let birt_date = data.data.individuals[0].events[0].date.as_ref().unwrap();
        assert_eq!(birt_date.value.as_ref().unwrap(), "BEF 1828");

        let resi_date = data.data.individuals[0].events[1].date.as_ref().unwrap();
        assert_eq!(resi_date.value.as_ref().unwrap(), "from 1900 to 1905");
    }

    #[test]
    fn test_parse_change_date_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 @MEDIA1@ OBJE\n\
            1 FILE /home/user/media/file_name.bmp\n\
            1 CHAN\n\
            2 DATE 1 APR 1998\n\
            3 TIME 12:34:56.789\n\
            2 NOTE A note\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let gedcom_data = doc.parse_data().unwrap();
        assert_eq!(gedcom_data.data.multimedia.len(), 1);

        let object = &gedcom_data.data.multimedia[0];

        let chan = object.change_date.as_ref().unwrap();
        let date = chan.date.as_ref().unwrap();
        assert_eq!(date.value.as_ref().unwrap(), "1 APR 1998");
        assert_eq!(date.time.as_ref().unwrap(), "12:34:56.789");

        let chan_note = chan.note.as_ref().unwrap();
        assert_eq!(chan_note.value.as_ref().unwrap(), "A note");
    }
}
