pub mod adopted;
pub mod child_link;
pub mod pedigree;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        custom::UserDefinedTag,
        individual::family_link::{
            adopted::AdoptedByWhichParent, child_link::ChildLinkStatus, pedigree::Pedigree,
        },
        note::Note,
        Xref,
    },
    GedcomError,
};

/// `FamilyLinkType` is a code used to indicates whether a family link is a pointer to a family
/// where this person is a child (FAMC tag), or it is pointer to a family where this person is a
/// spouse or parent (FAMS tag). See GEDCOM 5.5 spec, page 26.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub enum FamilyLinkType {
    Spouse,
    Child,
}

impl std::fmt::Display for FamilyLinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// `FamilyLink` indicates the normal lineage links through the use of pointers from the individual
/// to a family through either the FAMC tag or the FAMS tag. The FAMC tag provides a pointer to a
/// family where this person is a child. The FAMS tag provides a pointer to a family where this
/// person is a spouse or parent. See GEDCOM 5.5 spec, page 26.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct FamilyLink {
    pub xref: Xref,
    pub family_link_type: FamilyLinkType,
    pub pedigree_linkage_type: Option<Pedigree>,
    pub child_linkage_status: Option<ChildLinkStatus>,
    pub adopted_by: Option<AdoptedByWhichParent>,
    pub note: Option<Note>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl FamilyLink {
    /// Creates a new `FamilyLink` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8, tag: &str) -> Result<FamilyLink, GedcomError> {
        let xref = tokenizer.take_line_value()?;
        let link_type = match tag {
            "FAMC" => FamilyLinkType::Child,
            "FAMS" => FamilyLinkType::Spouse,
            _ => {
                return Err(GedcomError::ParseError {
                    line: tokenizer.line,
                    message: format!("Unhandled FamilyLinkType Tag: {tag}"),
                })
            }
        };
        let mut family_link = FamilyLink {
            xref,
            family_link_type: link_type,
            pedigree_linkage_type: None,
            child_linkage_status: None,
            adopted_by: None,
            note: None,
            custom_data: Vec::new(),
        };
        family_link.parse(tokenizer, level)?;
        Ok(family_link)
    }

    /// Sets the pedigree linkage type.
    ///
    /// # Errors
    ///
    /// This function will return an error if the pedigree text is unrecognized.
    pub fn set_pedigree(&mut self, pedigree_text: &str, line: u32) -> Result<(), GedcomError> {
        self.pedigree_linkage_type = match pedigree_text.to_lowercase().as_str() {
            "adopted" => Some(Pedigree::Adopted),
            "birth" => Some(Pedigree::Birth),
            "foster" => Some(Pedigree::Foster),
            "sealing" => Some(Pedigree::Sealing),
            _ => {
                return Err(GedcomError::ParseError {
                    line,
                    message: format!("Unrecognized FamilyLink.pedigree code: {pedigree_text}"),
                })
            }
        };
        Ok(())
    }

    /// Sets the child linkage status.
    ///
    /// # Errors
    ///
    /// This function will return an error if the status text is unrecognized.
    pub fn set_child_linkage_status(
        &mut self,
        status_text: &str,
        line: u32,
    ) -> Result<(), GedcomError> {
        self.child_linkage_status = match status_text.to_lowercase().as_str() {
            "challenged" => Some(ChildLinkStatus::Challenged),
            "disproven" => Some(ChildLinkStatus::Disproven),
            "proven" => Some(ChildLinkStatus::Proven),
            _ => {
                return Err(GedcomError::ParseError {
                    line,
                    message: format!(
                        "Unrecognized FamilyLink.child_linkage_status code: {status_text}"
                    ),
                })
            }
        };
        Ok(())
    }

    /// Sets the adopted by which parent.
    ///
    /// # Errors
    ///
    /// This function will return an error if the adopted by text is unrecognized.
    pub fn set_adopted_by_which_parent(
        &mut self,
        adopted_by_text: &str,
        line: u32,
    ) -> Result<(), GedcomError> {
        self.adopted_by = match adopted_by_text.to_lowercase().as_str() {
            "husb" => Some(AdoptedByWhichParent::Husband),
            "wife" => Some(AdoptedByWhichParent::Wife),
            "both" => Some(AdoptedByWhichParent::Both),
            _ => {
                return Err(GedcomError::ParseError {
                    line,
                    message: format!("Unrecognized FamilyLink.adopted_by code: {adopted_by_text}"),
                })
            }
        };
        Ok(())
    }

    #[must_use]
    pub fn child_linkage_status(&self) -> Option<&ChildLinkStatus> {
        self.child_linkage_status.as_ref()
    }
}

impl Parser for FamilyLink {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "PEDI" => {
                    self.set_pedigree(tokenizer.take_line_value()?.as_str(), tokenizer.line)?;
                }
                "STAT" => self.set_child_linkage_status(
                    tokenizer.take_line_value()?.as_str(),
                    tokenizer.line,
                )?,
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "ADOP" => self.set_adopted_by_which_parent(
                    tokenizer.take_line_value()?.as_str(),
                    tokenizer.line,
                )?,
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled FamilyLink Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
