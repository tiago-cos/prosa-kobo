use crate::epub::{Book, Chapter, Node, element, image, text};
use image::{ImageFormat, Rgb, RgbImage};
use serde_json::{Value, json};
use std::io::Cursor;

pub const DEVICE_HIGHLIGHT: &str = "amber lamp swung twice";
pub const DEVICE_NOTE: &str = "brass telescope";
pub const DEVICE_ONE_CHARACTER: &str = "Q";
pub const DEVICE_PARAGRAPH_END: &str = "lantern room went dark";
pub const DEVICE_ACROSS_PARAGRAPHS: &str = "cormorants sleep.\n[2.11] Dawn arrived";
pub const DEVICE_ACROSS_FORMATTING: &str = "very last lantern";
pub const DEVICE_ACCENTED: &str = "naïve façade";

pub const PROSA_HIGHLIGHT: &str = "silver nets";
pub const PROSA_NOTE: &str = "keeper's logbook";
pub const PROSA_ONE_CHARACTER: &str = "Z";
pub const PROSA_PARAGRAPH_END: &str = "fog rolled in";
pub const PROSA_ACROSS_PARAGRAPHS: &str = "the pier.\n[5.10] Thunder";
pub const PROSA_BESIDE_UNTRANSLATABLE: &str = "final ferry";

pub const MID_PARAGRAPH: &str = "ferry horn sounded";

pub const PLATE_CHAPTER: usize = 2;
pub const BLANK_CHAPTER: usize = 3;

pub const AWKWARD_TITLE: &str = r#"Ωmega — "quoted" \ back\slash & café"#;

/// Every book's own file names a publisher Prosa does not, so a step can tell
/// which of the two the device shows.
pub const FILE_PUBLISHER: &str = "The File’s Own Press";

const WORDS: [&str; 48] = [
    "morning", "window", "paper", "river", "garden", "small", "old", "road", "light", "stone", "gentle",
    "table", "letter", "bridge", "green", "slow", "cold", "field", "house", "rain", "evening", "long",
    "path", "wind", "bread", "chair", "open", "north", "hill", "voice", "clock", "yellow", "wall", "door",
    "narrow", "warm", "shadow", "market", "cloud", "boat", "distant", "kettle", "street", "winter", "summer",
    "a", "the", "and",
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fixture {
    Lighthouse,
    Orchard,
    OrchardRevised,
    Tidepool,
    Bare,
}

#[derive(Clone, Copy)]
pub enum Cover {
    Blue,
    Orange,
    Green,
    Purple,
}

impl Fixture {
    pub fn book(self) -> Book {
        let mut book = match self {
            Fixture::Lighthouse => lighthouse(),
            Fixture::Orchard => orchard(false),
            Fixture::OrchardRevised => orchard(true),
            Fixture::Tidepool => simple(
                "Tidepool Notes",
                "Marina Shore",
                "7d1c8a52-3b4e-4f60-8a91-0c2d3e4f5a6b",
                &["Low Tide", "High Tide"],
            ),
            Fixture::Bare => simple(
                "Bare Book",
                "Nobody",
                "7d1c8a52-3b4e-4f60-8a91-0c2d3e4f5a6c",
                &["Only", "Also"],
            ),
        };
        book.cover = self.cover().map(Cover::png);

        book
    }

    pub fn title(self) -> &'static str {
        match self {
            Fixture::Lighthouse => "The Lighthouse Keeper",
            Fixture::Orchard | Fixture::OrchardRevised => "An Orchard Year",
            Fixture::Tidepool => "Tidepool Notes",
            Fixture::Bare => "Bare Book",
        }
    }

    pub fn metadata(self) -> Option<Value> {
        let (subtitle, author, series_number, isbn) = match self {
            Fixture::Lighthouse => ("A device test", "Ada Beacon", 1.0, "9780000000001"),
            Fixture::Orchard | Fixture::OrchardRevised => {
                ("Twelve months of fruit", "Bram Cider", 2.0, "9780000000002")
            }
            Fixture::Tidepool => ("Notes from the shore", "Marina Shore", 3.0, "9780000000003"),
            Fixture::Bare => return None,
        };

        Some(json!({
            "title": self.title(),
            "subtitle": subtitle,
            "description": format!("Description one of {}.", self.title()),
            "publisher": "Prosa Test Press",
            "publication_date": 981_158_400_000_i64,
            "isbn": isbn,
            "contributors": [{ "name": author, "role": "Author" }],
            "genres": ["Testing"],
            "series": { "title": "Device Tests", "number": series_number },
            "page_count": 42,
            "language": "en",
        }))
    }

    pub fn cover(self) -> Option<Cover> {
        match self {
            Fixture::Lighthouse => Some(Cover::Blue),
            Fixture::Orchard | Fixture::OrchardRevised => Some(Cover::Green),
            Fixture::Tidepool => Some(Cover::Purple),
            Fixture::Bare => None,
        }
    }
}

/// Each cover differs in shape and in lightness as well as in colour, so they
/// stay apart on a greyscale screen.
impl Cover {
    pub fn describe(self) -> &'static str {
        match self {
            Cover::Blue => "dark blue with a large white circle",
            Cover::Orange => "pale orange with a black triangle",
            Cover::Green => "green with a white square",
            Cover::Purple => "dark purple with white horizontal stripes",
        }
    }

    pub fn png(self) -> Vec<u8> {
        let (background, shape) = match self {
            Cover::Blue => (Rgb([25, 45, 120]), Rgb([255, 255, 255])),
            Cover::Orange => (Rgb([250, 190, 90]), Rgb([0, 0, 0])),
            Cover::Green => (Rgb([70, 140, 80]), Rgb([255, 255, 255])),
            Cover::Purple => (Rgb([90, 40, 110]), Rgb([255, 255, 255])),
        };

        png(&RgbImage::from_fn(600, 900, |x, y| {
            let (x, y) = (i64::from(x), i64::from(y));
            let inside = match self {
                Cover::Blue => (x - 300).pow(2) + (y - 450).pow(2) < 200 * 200,
                Cover::Orange => (200..700).contains(&y) && (x - 300).abs() * 500 <= (y - 200) * 220,
                Cover::Green => (150..450).contains(&x) && (300..600).contains(&y),
                Cover::Purple => (150..750).contains(&y) && (y / 75) % 2 == 0,
            };

            if inside { shape } else { background }
        }))
    }
}

fn png(image: &RgbImage) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, ImageFormat::Png)
        .expect("Failed to encode a PNG");
    bytes.into_inner()
}

fn plate() -> Vec<u8> {
    png(&RgbImage::from_fn(800, 1200, |x, y| {
        if (i64::from(x) - 400).pow(2) + (i64::from(y) - 600).pow(2) < 250 * 250 {
            Rgb([0, 0, 0])
        } else {
            let shade = u8::try_from(y * 255 / 1200).unwrap_or(u8::MAX);
            Rgb([shade, shade, shade])
        }
    }))
}

/// A deterministic word salad, so a book is the same on every run and no
/// sentence of it can collide with a phrase a step looks for.
struct Filler(u64);

impl Filler {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn sentence(&mut self) -> String {
        let length = 7 + self.next() % 7;
        let words: Vec<&str> = (0..length)
            .map(|_| WORDS[usize::try_from(self.next() % WORDS.len() as u64).unwrap_or_default()])
            .collect();
        let sentence = words.join(" ");
        let mut characters = sentence.chars();
        let first = characters
            .next()
            .map(|c| c.to_uppercase().collect::<String>())
            .unwrap_or_default();

        format!("{first}{}.", characters.as_str())
    }

    fn sentences(&mut self, count: u64) -> String {
        (0..count).map(|_| self.sentence()).collect::<Vec<_>>().join(" ")
    }
}

fn paragraph(chapter: usize, number: usize, before: &str, after: &str) -> Node {
    let mut filler = Filler(0x9E37_79B9_7F4A_7C15 ^ (chapter as u64 * 1000 + number as u64));
    let count = 5 + filler.next() % 4;
    let mut content = format!("[{chapter}.{number}] ");
    if !before.is_empty() {
        content.push_str(before);
        content.push(' ');
    }
    content.push_str(&filler.sentences(count));
    if !after.is_empty() {
        content.push(' ');
        content.push_str(after);
    }

    element("p", vec![text(&content)])
}

fn chapter(number: usize, title: &str, paragraphs: usize, special: &[(usize, &str, &str)]) -> Chapter {
    chapter_with(number, title, paragraphs, special, &[])
}

fn chapter_with(
    number: usize,
    title: &str,
    paragraphs: usize,
    special: &[(usize, &str, &str)],
    replaced: &[(usize, Node)],
) -> Chapter {
    let mut body = vec![element("h1", vec![text(&format!("{number}. {title}"))])];

    for index in 1..=paragraphs {
        if let Some((_, node)) = replaced.iter().find(|(at, _)| *at == index) {
            body.push(node.clone());
            continue;
        }
        let (before, after) = special
            .iter()
            .find(|(at, _, _)| *at == index)
            .map_or(("", ""), |(_, before, after)| (*before, *after));
        body.push(paragraph(number, index, before, after));
    }

    Chapter::new(&format!("chapter{number:02}.xhtml"), title, body)
}

fn lighthouse() -> Book {
    let keeper = chapter_with(
        2,
        "The Keeper",
        12,
        &[
            (3, "The amber lamp swung twice above the harbour wall.", ""),
            (5, "", "Salt had crusted every hinge of the brass telescope."),
            (7, "She wrote a single letter, Q, on the misted glass.", ""),
            (9, "", "The keeper waited until the lantern room went dark"),
            (10, "", "The boat drifted home to where the cormorants sleep."),
            (11, "Dawn arrived without a sound.", ""),
        ],
        &[(
            12,
            element(
                "p",
                vec![
                    text("[2.12] It was the "),
                    element("em", vec![text("very")]),
                    text(" last lantern on the coast, and nobody lit it again."),
                ],
            ),
        )],
    );

    let storms = chapter(
        5,
        "Storms",
        12,
        &[
            (3, "The café’s naïve façade faced the sea.", ""),
            (5, "Gulls argued over the silver nets.", ""),
            (6, "The keeper's logbook listed every storm.", ""),
            (7, "The buoy was marked Z in red paint.", ""),
            (8, "", "Nobody spoke, and the fog rolled in"),
            (9, "", "The skiff was tied beneath the pier."),
            (10, "Thunder followed the lightning.", ""),
            (12, "The final ferry left at noon.", ""),
        ],
    );

    Book {
        title: Fixture::Lighthouse.title().to_owned(),
        author: "Ada Beacon".to_owned(),
        identifier: "7d1c8a52-3b4e-4f60-8a91-0c2d3e4f5a61".to_owned(),
        chapters: vec![
            chapter(1, "Arrival", 10, &[]),
            keeper,
            Chapter::new(
                "chapter03.xhtml",
                "Plate",
                vec![element("div", vec![image("images/plate.png")])],
            ),
            Chapter::new(
                "chapter04.xhtml",
                "Blank",
                vec![element("hr", Vec::new()), element("div", vec![text("   ")])],
            ),
            storms,
            chapter(
                6,
                "Departure",
                8,
                &[(4, "", "The ferry horn sounded across the bay.")],
            ),
        ],
        images: vec![("plate.png".to_owned(), plate())],
        publisher: Some(FILE_PUBLISHER.to_owned()),
        cover: None,
    }
}

fn orchard(revised: bool) -> Book {
    let opening = if revised {
        "This is the revised edition."
    } else {
        ""
    };
    let mut chapters = vec![
        chapter(1, "Blossom", 8, &[(1, opening, "")]),
        chapter(2, "Harvest", 8, &[]),
        chapter(3, "Cider", 8, &[]),
    ];
    if revised {
        chapters.push(chapter(
            4,
            "Afterword",
            3,
            &[(1, "This afterword exists only in the revised edition.", "")],
        ));
    }

    Book {
        title: Fixture::Orchard.title().to_owned(),
        author: "Bram Cider".to_owned(),
        identifier: "7d1c8a52-3b4e-4f60-8a91-0c2d3e4f5a62".to_owned(),
        chapters,
        images: Vec::new(),
        publisher: Some(FILE_PUBLISHER.to_owned()),
        cover: None,
    }
}

fn simple(title: &str, author: &str, identifier: &str, chapters: &[&str]) -> Book {
    Book {
        title: title.to_owned(),
        author: author.to_owned(),
        identifier: identifier.to_owned(),
        chapters: chapters
            .iter()
            .enumerate()
            .map(|(index, name)| chapter(index + 1, name, 6, &[]))
            .collect(),
        images: Vec::new(),
        publisher: Some(FILE_PUBLISHER.to_owned()),
        cover: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kepub_rs::{Converter, epub_to_kobo_location, kobo_to_epub_location, validate_epub_location};

    const PHRASES: [&str; 14] = [
        DEVICE_HIGHLIGHT,
        DEVICE_NOTE,
        DEVICE_ONE_CHARACTER,
        DEVICE_PARAGRAPH_END,
        DEVICE_ACROSS_PARAGRAPHS,
        DEVICE_ACROSS_FORMATTING,
        DEVICE_ACCENTED,
        PROSA_HIGHLIGHT,
        PROSA_NOTE,
        PROSA_ONE_CHARACTER,
        PROSA_PARAGRAPH_END,
        PROSA_ACROSS_PARAGRAPHS,
        PROSA_BESIDE_UNTRANSLATABLE,
        MID_PARAGRAPH,
    ];

    fn kepub(epub: &[u8]) -> Vec<u8> {
        let mut kepub = Cursor::new(Vec::new());
        Converter::default()
            .convert(Cursor::new(epub), &mut kepub)
            .expect("The generated book should convert");
        kepub.into_inner()
    }

    fn round_trip(epub: &[u8], kepub: &[u8], location: &str) {
        validate_epub_location(Cursor::new(epub), location)
            .unwrap_or_else(|e| panic!("{location} is not a location in the book: {e}"));
        let kobo = epub_to_kobo_location(Cursor::new(kepub), location)
            .unwrap_or_else(|e| panic!("{location} has no Kobo position: {e}"));
        let back = kobo_to_epub_location(Cursor::new(kepub), &kobo).expect("A Kobo position translates back");

        assert_eq!(back, location, "{location} came back from {kobo} as {back}");
    }

    #[test]
    fn every_phrase_is_found_once_and_covers_itself() {
        let book = Fixture::Lighthouse.book();

        for phrase in PHRASES {
            let span = book.locate(phrase);
            assert_eq!(book.text_between(&span.start, &span.end).as_deref(), Ok(phrase));
        }
    }

    #[test]
    fn every_phrase_survives_the_trip_to_the_kobo_and_back() {
        let book = Fixture::Lighthouse.book();
        let epub = book.to_epub();
        let kepub = kepub(&epub);

        for phrase in PHRASES {
            let span = book.locate(phrase);
            round_trip(&epub, &kepub, &span.start);
            round_trip(&epub, &kepub, &span.end);
        }
    }

    #[test]
    fn a_phrase_that_ends_its_paragraph_ends_at_the_run_length() {
        let book = Fixture::Lighthouse.book();
        let end = book.position_after(DEVICE_PARAGRAPH_END);
        let next = book.position("[2.10]");

        assert!(end.ends_with(&format!(
                ":{}",
                book.text_between(&book.position("[2.9]"), &end)
                    .map(|t| t.chars().count())
                    .unwrap_or_default()
            )));
        assert_eq!(book.text_between(&end, &next).as_deref(), Ok("\n"));
    }

    #[test]
    fn the_image_page_is_a_kobo_position_and_the_blank_page_is_not() {
        let book = Fixture::Lighthouse.book();
        let epub = book.to_epub();
        let kepub = kepub(&epub);

        round_trip(&epub, &kepub, &book.element_location(PLATE_CHAPTER, &[0, 0]));

        let rule = book.element_location(BLANK_CHAPTER, &[0]);
        let whitespace = book.text_location(BLANK_CHAPTER, &[1], 3);
        for location in [rule, whitespace] {
            validate_epub_location(Cursor::new(&epub), &location).expect("A real location");
            assert!(epub_to_kobo_location(Cursor::new(&kepub), &location).is_err());
        }
    }

    #[test]
    fn paragraphs_are_found_by_their_labels() {
        let book = Fixture::Lighthouse.book();
        let resolved = book
            .resolve(&book.position("[2.6]"))
            .expect("The paragraph resolves");

        assert_eq!((resolved.chapter(), resolved.block()), (Some(1), 6));
    }

    #[test]
    fn the_cover_page_is_a_place_of_its_own() {
        let book = Fixture::Lighthouse.book();

        assert_eq!(book.describe("OEBPS/cover.xhtml#0/0"), "the cover page");
    }

    #[test]
    fn every_book_converts() {
        for fixture in [
            Fixture::Lighthouse,
            Fixture::Orchard,
            Fixture::OrchardRevised,
            Fixture::Tidepool,
            Fixture::Bare,
        ] {
            kepub(&fixture.book().to_epub());
        }
    }
}
