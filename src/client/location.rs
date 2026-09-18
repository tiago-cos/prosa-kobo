use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

/// A position inside an EPUB, as Prosa spells it: `OEBPS/chapter-001.xhtml#0/2/t1:44`.
///
/// The fragment is an element path from `<body>`, optionally ending in a text
/// run (`t1`) and a character offset. A location without an offset names a
/// whole element, which is how a page holding only an image is addressed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProsaLocation {
    pub source: String,
    pub path: String,
    pub offset: Option<u32>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProsaLocationError {
    MissingFragment,
}

impl ProsaLocation {
    pub fn new(source: &str, path: &str, offset: Option<u32>) -> Self {
        Self {
            source: source.to_owned(),
            path: path.to_owned(),
            offset,
        }
    }

    /// Everything after the `#`, which is what the Kobo device carries as the
    /// opaque value of a bookmark.
    pub fn fragment(&self) -> String {
        match self.offset {
            Some(offset) => format!("{}:{offset}", self.path),
            None => self.path.clone(),
        }
    }
}

impl FromStr for ProsaLocation {
    type Err = ProsaLocationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (source, fragment) = value.split_once('#').ok_or(ProsaLocationError::MissingFragment)?;

        // Only a trailing `:<digits>` is an offset; a path never contains a colon.
        let (path, offset) = match fragment.rsplit_once(':') {
            Some((path, offset)) => match offset.parse::<u32>() {
                Ok(offset) => (path, Some(offset)),
                Err(_) => (fragment, None),
            },
            None => (fragment, None),
        };

        Ok(Self::new(source, path, offset))
    }
}

impl Display for ProsaLocation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}#{}", self.source, self.path)?;

        match self.offset {
            Some(offset) => write!(formatter, ":{offset}"),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_character_location() {
        let location: ProsaLocation = "OEBPS/chapter-001.xhtml#0/2/t1:44"
            .parse()
            .expect("Failed to parse location");

        assert_eq!(
            location,
            ProsaLocation::new("OEBPS/chapter-001.xhtml", "0/2/t1", Some(44))
        );
    }

    #[test]
    fn parses_an_element_location() {
        let location: ProsaLocation = "OEBPS/cover.xhtml#0/0".parse().expect("Failed to parse location");

        assert_eq!(location, ProsaLocation::new("OEBPS/cover.xhtml", "0/0", None));
    }

    #[test]
    fn parses_a_location_with_an_empty_element_path() {
        let location: ProsaLocation = "chapter.xhtml#t0:5".parse().expect("Failed to parse location");

        assert_eq!(location, ProsaLocation::new("chapter.xhtml", "t0", Some(5)));
    }

    #[test]
    fn rejects_a_location_without_a_fragment() {
        let location = "OEBPS/chapter-001.xhtml".parse::<ProsaLocation>();

        assert_eq!(location, Err(ProsaLocationError::MissingFragment));
    }

    #[test]
    fn round_trips_through_display() {
        for raw in ["OEBPS/chapter-001.xhtml#0/2/t1:44", "OEBPS/cover.xhtml#0/0"] {
            let location: ProsaLocation = raw.parse().expect("Failed to parse location");

            assert_eq!(location.to_string(), raw);
        }
    }
}
