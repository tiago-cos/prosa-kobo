use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};
use strum_macros::{EnumMessage, EnumProperty};

#[derive(EnumMessage, EnumProperty, Debug)]
pub enum KepubError {
    #[strum(message = "ConversionFailed")]
    #[strum(detailed_message = "The book could not be converted to a kepub.")]
    #[strum(props(StatusCode = "500"))]
    ConversionFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KoboPosition {
    pub chapter: String,
    pub span: String,
    pub offset: u32,
}

impl KoboPosition {
    pub fn new(chapter: &str, span: &str, offset: u32) -> Self {
        Self {
            chapter: chapter.to_owned(),
            span: span_id(span),
            offset,
        }
    }

    pub fn selector(&self) -> String {
        format!("span#{}", self.span.replace('\\', r"\\").replace('.', r"\."))
    }
}

fn span_id(span: &str) -> String {
    let tail = span.rsplit_once('#').map_or(span, |(_, tail)| tail);

    tail.trim()
        .split([' ', '\t', '>', '[', '/', ':'])
        .next()
        .unwrap_or_default()
        .replace('\\', "")
}

pub struct KepubCache {
    capacity: u64,
    entries: Mutex<Entries>,
}

#[derive(Default)]
struct Entries {
    books: HashMap<String, Arc<[u8]>>,
    recency: VecDeque<String>,
    size: u64,
}

impl KepubCache {
    pub fn new(capacity_bytes: u64) -> Self {
        Self {
            capacity: capacity_bytes,
            entries: Mutex::new(Entries::default()),
        }
    }

    pub fn get(&self, book_id: &str) -> Option<Arc<[u8]>> {
        let mut entries = self.lock();
        let kepub = entries.books.get(book_id)?.clone();

        entries.touch(book_id);

        Some(kepub)
    }

    pub fn insert(&self, book_id: &str, kepub: &Arc<[u8]>) {
        let size = kepub.len() as u64;

        if size > self.capacity {
            return;
        }

        let mut entries = self.lock();
        entries.remove(book_id);

        while entries.size + size > self.capacity {
            let Some(oldest) = entries.recency.front().cloned() else {
                break;
            };

            entries.remove(&oldest);
        }

        entries.books.insert(book_id.to_owned(), Arc::clone(kepub));
        entries.recency.push_back(book_id.to_owned());
        entries.size += size;
    }

    pub fn evict(&self, book_id: &str) {
        self.lock().remove(book_id);
    }

    pub fn clear(&self) {
        *self.lock() = Entries::default();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Entries> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Entries {
    fn remove(&mut self, book_id: &str) {
        if let Some(kepub) = self.books.remove(book_id) {
            self.size -= kepub.len() as u64;
            self.recency.retain(|id| id != book_id);
        }
    }

    fn touch(&mut self, book_id: &str) {
        self.recency.retain(|id| id != book_id);
        self.recency.push_back(book_id.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kepub(size: usize) -> Arc<[u8]> {
        vec![0u8; size].into()
    }

    #[test]
    fn returns_what_it_was_given() {
        let cache = KepubCache::new(100);
        cache.insert("book", &kepub(10));

        assert_eq!(cache.get("book").expect("Expected a hit").len(), 10);
    }

    #[test]
    fn misses_a_book_it_does_not_hold() {
        let cache = KepubCache::new(100);

        assert!(cache.get("book").is_none());
    }

    #[test]
    fn drops_the_least_recently_used_book_to_make_room() {
        let cache = KepubCache::new(100);
        cache.insert("first", &kepub(50));
        cache.insert("second", &kepub(50));

        cache.get("first");
        cache.insert("third", &kepub(50));

        assert!(cache.get("first").is_some());
        assert!(cache.get("second").is_none());
        assert!(cache.get("third").is_some());
    }

    #[test]
    fn serves_but_does_not_hold_a_book_larger_than_the_cache() {
        let cache = KepubCache::new(100);
        cache.insert("huge", &kepub(101));

        assert!(cache.get("huge").is_none());
    }

    #[test]
    fn replacing_a_book_does_not_double_count_its_size() {
        let cache = KepubCache::new(100);
        cache.insert("book", &kepub(60));
        cache.insert("book", &kepub(60));
        cache.insert("other", &kepub(40));

        assert!(cache.get("book").is_some());
        assert!(cache.get("other").is_some());
    }

    #[test]
    fn evicting_frees_the_room_it_took() {
        let cache = KepubCache::new(100);
        cache.insert("book", &kepub(100));
        cache.evict("book");
        cache.insert("other", &kepub(100));

        assert!(cache.get("book").is_none());
        assert!(cache.get("other").is_some());
    }
}

#[cfg(test)]
mod position_tests {
    use super::*;

    fn span_of(raw: &str) -> String {
        KoboPosition::new("chapter.xhtml", raw, 0).span
    }

    #[test]
    fn reads_the_selector_an_annotation_carries() {
        assert_eq!(span_of(r"span#kobo\.4\.1"), "kobo.4.1");
    }

    #[test]
    fn reads_the_bare_id_a_bookmark_carries() {
        assert_eq!(span_of("kobo.4.1"), "kobo.4.1");
    }

    #[test]
    fn reads_a_selector_that_was_never_escaped() {
        assert_eq!(span_of("span#kobo.4.1"), "kobo.4.1");
    }

    #[test]
    fn keeps_only_the_element_a_longer_selector_ends_at() {
        assert_eq!(span_of(r"div#chapter > span#kobo\.4\.1"), "kobo.4.1");
        assert_eq!(span_of(r"span#kobo\.4\.1:first-child"), "kobo.4.1");
        assert_eq!(span_of(r"span#kobo\.4\.1[data-x]"), "kobo.4.1");
    }

    #[test]
    fn writes_the_selector_the_device_expects() {
        let position = KoboPosition::new("chapter.xhtml", "kobo.4.1", 0);

        assert_eq!(position.selector(), r"span#kobo\.4\.1");
    }

    #[test]
    fn a_selector_round_trips() {
        let raw = r"span#kobo\.12\.3";
        let position = KoboPosition::new("chapter.xhtml", raw, 0);

        assert_eq!(position.selector(), raw);
    }
}
