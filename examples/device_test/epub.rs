use std::{fmt::Write as _, io::Write as _};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

const BLOCKS: [&str; 6] = ["p", "h1", "h2", "div", "hr", "blockquote"];

#[derive(Clone, Debug)]
pub enum Node {
    Element {
        name: &'static str,
        attributes: Vec<(&'static str, String)>,
        children: Vec<Node>,
    },
    Text(String),
}

pub fn element(name: &'static str, children: Vec<Node>) -> Node {
    Node::Element {
        name,
        attributes: Vec::new(),
        children,
    }
}

pub fn text(content: &str) -> Node {
    Node::Text(content.to_owned())
}

pub fn image(source: &str) -> Node {
    Node::Element {
        name: "img",
        attributes: vec![("src", source.to_owned()), ("alt", String::new())],
        children: Vec::new(),
    }
}

pub struct Chapter {
    pub file: String,
    pub title: String,
    pub body: Vec<Node>,
    flat: String,
    runs: Vec<Run>,
}

struct Run {
    path: Vec<usize>,
    text_index: usize,
    start: usize,
    len: usize,
}

pub struct Book {
    pub title: String,
    pub author: String,
    pub identifier: String,
    pub chapters: Vec<Chapter>,
    pub images: Vec<(String, Vec<u8>)>,
    pub publisher: Option<String>,
    pub cover: Option<Vec<u8>>,
}

pub struct Span {
    pub start: String,
    pub end: String,
}

pub enum Resolved {
    Cover,
    Text {
        chapter: usize,
        block: usize,
        at: usize,
    },
    Element {
        chapter: usize,
        block: usize,
        name: &'static str,
    },
}

impl Resolved {
    pub fn chapter(&self) -> Option<usize> {
        match self {
            Resolved::Cover => None,
            Resolved::Text { chapter, .. } | Resolved::Element { chapter, .. } => Some(*chapter),
        }
    }

    pub fn block(&self) -> usize {
        match self {
            Resolved::Cover => 0,
            Resolved::Text { block, .. } | Resolved::Element { block, .. } => *block,
        }
    }
}

impl Chapter {
    pub fn new(file: &str, title: &str, body: Vec<Node>) -> Self {
        let mut flat = String::new();
        let mut runs = Vec::new();
        flatten(&body, &mut Vec::new(), &mut flat, &mut runs);

        Self {
            file: format!("OEBPS/{file}"),
            title: title.to_owned(),
            body,
            flat,
            runs,
        }
    }

    fn href(&self) -> &str {
        self.file.trim_start_matches("OEBPS/")
    }

    fn xhtml(&self) -> String {
        let mut body = String::new();
        for node in &self.body {
            serialize(node, &mut body);
        }

        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n\
             <html xmlns=\"http://www.w3.org/1999/xhtml\" lang=\"en\" xml:lang=\"en\">\
             <head><title>{}</title></head><body>{body}</body></html>",
            escape(&self.title)
        )
    }

    fn location(&self, run: &Run, offset: usize) -> String {
        let mut location = format!("{}#", self.file);
        for index in &run.path {
            let _ = write!(location, "{index}/");
        }
        let _ = write!(location, "t{}:{offset}", run.text_index);
        location
    }

    /// The first character of a phrase is found in the run holding it, but
    /// its end in the run holding its last character, so a phrase that
    /// finishes a run ends at that run's length rather than at 0 of the next.
    fn span(&self, start: usize, end: usize) -> Span {
        let first = self
            .runs
            .iter()
            .find(|run| run.start <= start && start < run.start + run.len)
            .expect("A phrase starts inside a run");
        let last = self
            .runs
            .iter()
            .find(|run| run.start < end && end <= run.start + run.len)
            .expect("A phrase ends inside a run");

        Span {
            start: self.location(first, start - first.start),
            end: self.location(last, end - last.start),
        }
    }

    fn element_at(&self, path: &[usize]) -> Option<&'static str> {
        let mut children = &self.body;
        let mut found = None;

        for &index in path {
            let Node::Element {
                name,
                children: inner,
                ..
            } = children
                .iter()
                .filter(|node| matches!(node, Node::Element { .. }))
                .nth(index)?
            else {
                return None;
            };
            found = Some(*name);
            children = inner;
        }

        found
    }
}

impl Book {
    pub fn to_epub(&self) -> Vec<u8> {
        let mut zip = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let deflated = SimpleFileOptions::default();

        let mut add = |name: &str, content: &[u8], options: SimpleFileOptions| {
            zip.start_file(name, options)
                .expect("Failed to start an EPUB entry");
            zip.write_all(content).expect("Failed to write an EPUB entry");
        };

        add("mimetype", b"application/epub+zip", stored);
        add("META-INF/container.xml", CONTAINER.as_bytes(), deflated);
        add("OEBPS/content.opf", self.opf().as_bytes(), deflated);
        add("OEBPS/nav.xhtml", self.nav().as_bytes(), deflated);
        add("OEBPS/toc.ncx", self.ncx().as_bytes(), deflated);
        for chapter in &self.chapters {
            add(&chapter.file, chapter.xhtml().as_bytes(), deflated);
        }
        for (name, png) in &self.images {
            add(&format!("OEBPS/images/{name}"), png, stored);
        }
        if let Some(cover) = &self.cover {
            add("OEBPS/images/cover.png", cover, stored);
            add(COVER_FILE, COVER_PAGE.as_bytes(), deflated);
        }

        zip.finish().expect("Failed to finish the EPUB").into_inner()
    }

    pub fn chapter_of(&self, file: &str) -> Option<usize> {
        self.chapters.iter().position(|chapter| chapter.file == file)
    }

    pub fn locate(&self, phrase: &str) -> Span {
        let mut found = self.chapters.iter().filter_map(|chapter| {
            let byte = chapter.flat.find(phrase)?;
            let start = chapter.flat[..byte].chars().count();
            Some(chapter.span(start, start + phrase.chars().count()))
        });

        let span = found
            .next()
            .unwrap_or_else(|| panic!("{phrase:?} is not in {}", self.title));
        assert!(found.next().is_none(), "{phrase:?} is in {} twice", self.title);

        span
    }

    pub fn position(&self, phrase: &str) -> String {
        self.locate(phrase).start
    }

    pub fn position_after(&self, phrase: &str) -> String {
        self.locate(phrase).end
    }

    pub fn element_location(&self, chapter: usize, path: &[usize]) -> String {
        let chapter = &self.chapters[chapter];
        assert!(chapter.element_at(path).is_some(), "No element at {path:?}");
        let path: Vec<String> = path.iter().map(ToString::to_string).collect();

        format!("{}#{}", chapter.file, path.join("/"))
    }

    pub fn text_location(&self, chapter: usize, path: &[usize], offset: usize) -> String {
        let chapter = &self.chapters[chapter];
        let run = chapter
            .runs
            .iter()
            .find(|run| run.path == path && run.text_index == 0)
            .unwrap_or_else(|| panic!("No text at {path:?}"));
        assert!(
            offset <= run.len,
            "{offset} is past the end of the text at {path:?}"
        );

        chapter.location(run, offset)
    }

    pub fn resolve(&self, location: &str) -> Result<Resolved, String> {
        let (file, position) = location
            .split_once('#')
            .ok_or_else(|| format!("{location} names no position"))?;
        if file == COVER_FILE && self.cover.is_some() {
            return Ok(Resolved::Cover);
        }
        let chapter_index = self
            .chapter_of(file)
            .ok_or_else(|| format!("{file} is not a chapter of {}", self.title))?;
        let chapter = &self.chapters[chapter_index];

        let segments: Vec<&str> = position.split('/').collect();
        let (last, elements) = segments.split_last().ok_or("An empty position")?;
        let path = elements
            .iter()
            .map(|segment| segment.parse::<usize>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| format!("{position} has a malformed element path"))?;

        if let Some(run) = last.strip_prefix('t') {
            let (text_index, offset) = run
                .split_once(':')
                .ok_or_else(|| format!("{position} names a text run without an offset"))?;
            let text_index: usize = text_index.parse().map_err(|_| "A malformed text index")?;
            let offset: usize = offset.parse().map_err(|_| "A malformed offset")?;
            let run = chapter
                .runs
                .iter()
                .find(|run| run.path == path && run.text_index == text_index)
                .ok_or_else(|| format!("{position} names no text run"))?;
            if offset > run.len {
                return Err(format!("{position} is past the end of its text run"));
            }

            return Ok(Resolved::Text {
                chapter: chapter_index,
                block: path.first().copied().unwrap_or_default(),
                at: run.start + offset,
            });
        }

        let mut path = path;
        path.push(
            last.parse()
                .map_err(|_| format!("{position} ends in neither a run nor an element"))?,
        );
        let name = chapter
            .element_at(&path)
            .ok_or_else(|| format!("{position} names no element"))?;

        Ok(Resolved::Element {
            chapter: chapter_index,
            block: path[0],
            name,
        })
    }

    pub fn text_between(&self, start: &str, end: &str) -> Result<String, String> {
        match (self.resolve(start)?, self.resolve(end)?) {
            (
                Resolved::Text {
                    chapter, at: from, ..
                },
                Resolved::Text {
                    chapter: other,
                    at: to,
                    ..
                },
            ) if chapter == other && from <= to => Ok(self.chapters[chapter]
                .flat
                .chars()
                .skip(from)
                .take(to - from)
                .collect()),
            _ => Err(format!("{start} .. {end} is not a span of text")),
        }
    }

    pub fn describe(&self, location: &str) -> String {
        match self.resolve(location) {
            Ok(Resolved::Text { chapter, at, .. }) => {
                let context: String = self.chapters[chapter].flat.chars().skip(at).take(60).collect();
                format!(
                    "chapter {} “{}”, at “{}…”",
                    chapter + 1,
                    self.chapters[chapter].title,
                    context.replace('\n', " ⏎ ")
                )
            }
            Ok(Resolved::Cover) => "the cover page".to_owned(),
            Ok(Resolved::Element { chapter, name, .. }) => format!(
                "chapter {} “{}”, the <{name}> itself",
                chapter + 1,
                self.chapters[chapter].title
            ),
            Err(problem) => format!("{location}, which does not resolve: {problem}"),
        }
    }

    fn opf(&self) -> String {
        let mut manifest = String::from(
            "<item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\
             <item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>",
        );
        let mut spine = String::new();
        let mut cover_meta = "";
        if self.cover.is_some() {
            manifest.push_str(
                "<item id=\"cover-image\" href=\"images/cover.png\" media-type=\"image/png\" properties=\"cover-image\"/>\
                 <item id=\"cover\" href=\"cover.xhtml\" media-type=\"application/xhtml+xml\"/>",
            );
            spine.push_str("<itemref idref=\"cover\"/>");
            cover_meta = "<meta name=\"cover\" content=\"cover-image\"/>";
        }
        let publisher = self
            .publisher
            .as_deref()
            .map(|publisher| format!("<dc:publisher>{}</dc:publisher>", escape(publisher)))
            .unwrap_or_default();
        for (index, chapter) in self.chapters.iter().enumerate() {
            let _ = write!(
                manifest,
                "<item id=\"c{index}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>",
                chapter.href()
            );
            let _ = write!(spine, "<itemref idref=\"c{index}\"/>");
        }
        for (index, (name, _)) in self.images.iter().enumerate() {
            let _ = write!(
                manifest,
                "<item id=\"i{index}\" href=\"images/{name}\" media-type=\"image/png\"/>"
            );
        }

        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
             <package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"id\">\
             <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\
             <dc:identifier id=\"id\">urn:uuid:{}</dc:identifier><dc:title>{}</dc:title>\
             <dc:creator>{}</dc:creator>{publisher}<dc:language>en</dc:language>\
             <meta property=\"dcterms:modified\">2026-01-01T00:00:00Z</meta>{cover_meta}</metadata>\
             <manifest>{manifest}</manifest><spine toc=\"ncx\">{spine}</spine></package>",
            self.identifier,
            escape(&self.title),
            escape(&self.author)
        )
    }

    fn nav(&self) -> String {
        let mut items = String::new();
        for chapter in &self.chapters {
            let _ = write!(
                items,
                "<li><a href=\"{}\">{}</a></li>",
                chapter.href(),
                escape(&chapter.title)
            );
        }

        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n\
             <html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">\
             <head><title>Contents</title></head><body>\
             <nav epub:type=\"toc\"><ol>{items}</ol></nav></body></html>"
        )
    }

    fn ncx(&self) -> String {
        let mut points = String::new();
        for (index, chapter) in self.chapters.iter().enumerate() {
            let _ = write!(
                points,
                "<navPoint id=\"n{index}\" playOrder=\"{}\"><navLabel><text>{}</text></navLabel>\
                 <content src=\"{}\"/></navPoint>",
                index + 1,
                escape(&chapter.title),
                chapter.href()
            );
        }

        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
             <ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\">\
             <head><meta name=\"dtb:uid\" content=\"urn:uuid:{}\"/></head>\
             <docTitle><text>{}</text></docTitle><navMap>{points}</navMap></ncx>",
            self.identifier,
            escape(&self.title)
        )
    }
}

const COVER_FILE: &str = "OEBPS/cover.xhtml";

const COVER_PAGE: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n\
    <html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Cover</title></head>\
    <body><div><img src=\"images/cover.png\" alt=\"\"/></div></body></html>";

const CONTAINER: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
    <container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">\
    <rootfiles><rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/>\
    </rootfiles></container>";

/// Mirrors how kepub-rs counts: element and text indices are separate
/// sequences at each level. A line break stands between blocks so a phrase
/// cannot straddle two paragraphs by accident.
fn flatten(nodes: &[Node], path: &mut Vec<usize>, flat: &mut String, runs: &mut Vec<Run>) {
    let mut elements = 0;
    let mut texts = 0;

    for node in nodes {
        match node {
            Node::Text(content) => {
                runs.push(Run {
                    path: path.clone(),
                    text_index: texts,
                    start: flat.chars().count(),
                    len: content.chars().count(),
                });
                flat.push_str(content);
                texts += 1;
            }
            Node::Element { name, children, .. } => {
                path.push(elements);
                flatten(children, path, flat, runs);
                path.pop();
                elements += 1;

                if BLOCKS.contains(name) && !flat.is_empty() && !flat.ends_with('\n') {
                    flat.push('\n');
                }
            }
        }
    }
}

fn serialize(node: &Node, out: &mut String) {
    match node {
        Node::Text(content) => out.push_str(&escape(content)),
        Node::Element {
            name,
            attributes,
            children,
        } => {
            let _ = write!(out, "<{name}");
            for (key, value) in attributes {
                let _ = write!(out, " {key}=\"{}\"", escape(value));
            }
            if children.is_empty() {
                out.push_str("/>");
                return;
            }
            out.push('>');
            for child in children {
                serialize(child, out);
            }
            let _ = write!(out, "</{name}>");
        }
    }
}

fn escape(content: &str) -> String {
    content
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
