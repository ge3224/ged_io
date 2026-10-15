use crate::{
    types::{header::Header, submission::Submission, submitter::Submitter, GedcomData},
    xref::{AnyHandle, Xref},
    GedcomError,
};

impl GedcomData {
    /// Points the header at a submitter record.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if the xref does not resolve to a
    /// submitter, [`GedcomError::AlreadyLinked`] if the header already points
    /// at one.
    pub fn link_header_and_submitter(
        &mut self,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let xref = submitter.into();

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&xref) else {
            return Err(GedcomError::XrefNotFound {
                xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        };

        let Some(h) = &mut self.header else {
            unreachable!("header is not optional")
        };

        if let Some(x) = &h.submitter_tag {
            return Err(GedcomError::AlreadyLinked {
                from_xref: x.as_str().to_owned(),
                to_xref: xref,
                link_type: Submitter::RECORD_TYPE.to_string(),
            });
        }

        h.submitter_tag = Some(self.xrefs.pointer(&xref));

        Ok(())
    }

    /// Decouples a submitter record from the document header.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if the provided xref does not
    /// resolve to a record, or [`GedcomError::NotLinked`] if the header holds
    /// no association pointing to the given submitter.
    pub fn unlink_header_and_submitter(
        &mut self,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let xref = submitter.into();

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&xref) else {
            return Err(GedcomError::XrefNotFound {
                xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        };

        let Some(h) = &mut self.header else {
            unreachable!("header is not optional")
        };

        if !h.submitter_tag.as_ref().is_some_and(|p| *p == *xref) {
            let from_xref = match &h.submitter_tag {
                Some(x) => x.as_str().to_owned(),
                None => "None".to_string(),
            };

            return Err(GedcomError::NotLinked {
                from_xref,
                to_xref: xref,
                link_type: Submitter::RECORD_TYPE.to_string(),
            });
        }

        h.submitter_tag = None;

        Ok(())
    }

    /// Points the header at a submission record.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if the xref does not resolve to a
    /// submission, [`GedcomError::AlreadyLinked`] if the header already points
    /// at one.
    pub fn link_header_and_submission(
        &mut self,
        submission: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let xref = submission.into();

        let Some(AnyHandle::Submission(_)) = self.xrefs.handle(&xref) else {
            return Err(GedcomError::XrefNotFound {
                xref,
                record_type: Submission::RECORD_TYPE.to_string(),
            });
        };

        let Some(h) = &mut self.header else {
            unreachable!("header is not optional")
        };

        if let Some(x) = &h.submission_tag {
            return Err(GedcomError::AlreadyLinked {
                from_xref: x.as_str().to_owned(),
                to_xref: xref,
                link_type: Submission::RECORD_TYPE.to_string(),
            });
        }

        h.submission_tag = Some(self.xrefs.pointer(&xref));

        Ok(())
    }
    /// Decouples a submission record from the document header.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if the provided xref does not
    /// resolve to a record, or [`GedcomError::NotLinked`] if the header holds
    /// no association pointing to the given submission.
    pub fn unlink_header_and_submission(
        &mut self,
        submission: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let xref = submission.into();

        let Some(AnyHandle::Submission(_)) = self.xrefs.handle(&xref) else {
            return Err(GedcomError::XrefNotFound {
                xref,
                record_type: Submission::RECORD_TYPE.to_string(),
            });
        };

        let Some(h) = &mut self.header else {
            unreachable!("header is not optional")
        };

        if !h.submission_tag.as_ref().is_some_and(|p| *p == *xref) {
            let from_xref = match &h.submission_tag {
                Some(x) => x.as_str().to_owned(),
                None => "None".to_string(),
            };

            return Err(GedcomError::NotLinked {
                from_xref,
                to_xref: xref,
                link_type: Submission::RECORD_TYPE.to_string(),
            });
        }

        h.submission_tag = None;

        Ok(())
    }

    pub(crate) fn set_header(&mut self, mut header: Header) {
        header.pointers_mut(&mut |p| self.xrefs.adopt(p));
        self.header = Some(header);
    }
}
