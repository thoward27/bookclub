use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSearchResult {
    pub title: String,
    pub author: String,
    pub isbn10: String,
    pub isbn13: String,
}

impl BookSearchResult {
    /// Merge another result into this one, filling in any empty fields.
    fn merge(&mut self, other: &BookSearchResult) {
        if self.title.is_empty() && !other.title.is_empty() {
            self.title.clone_from(&other.title);
        }
        if self.author.is_empty() && !other.author.is_empty() {
            self.author.clone_from(&other.author);
        }
        if self.isbn10.is_empty() && !other.isbn10.is_empty() {
            self.isbn10.clone_from(&other.isbn10);
        }
        if self.isbn13.is_empty() && !other.isbn13.is_empty() {
            self.isbn13.clone_from(&other.isbn13);
        }
    }

    fn quality_score(&self) -> u8 {
        let mut score = 0;
        if !self.author.is_empty() {
            score += 1;
        }
        if !self.isbn10.is_empty() {
            score += 1;
        }
        if !self.isbn13.is_empty() {
            score += 1;
        }
        score
    }
}

/// Deduplicate results by case-insensitive title, merging fields from
/// duplicates into the highest-quality entry. Preserves original ordering
/// based on when a title was first seen.
fn dedup_results(results: Vec<BookSearchResult>, limit: usize) -> Vec<BookSearchResult> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut deduped: Vec<BookSearchResult> = Vec::new();

    for result in results {
        let key = result.title.trim().to_lowercase();
        if let Some(&idx) = seen.get(&key) {
            let existing = &mut deduped[idx];
            if result.quality_score() > existing.quality_score() {
                // New entry is better — merge the old one's fields into the new one,
                // then replace.
                let mut better = result;
                better.merge(existing);
                deduped[idx] = better;
            } else {
                // Existing entry is better — merge the new one's fields into it.
                existing.merge(&result);
            }
        } else {
            seen.insert(key, deduped.len());
            deduped.push(result);
        }
    }

    deduped.truncate(limit);
    deduped
}

#[async_trait]
pub trait BookSearchProvider: Send + Sync {
    async fn search(&self, query: &str, limit: usize) -> eyre::Result<Vec<BookSearchResult>>;
}

// --- Open Library implementation ---

pub struct OpenLibraryProvider {
    client: reqwest::Client,
}

impl Default for OpenLibraryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenLibraryProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenLibraryResponse {
    docs: Vec<OpenLibraryDoc>,
}

#[derive(Debug, Deserialize)]
struct OpenLibraryDoc {
    title: Option<String>,
    author_name: Option<Vec<String>>,
    isbn: Option<Vec<String>>,
}

impl OpenLibraryDoc {
    fn into_result(self) -> BookSearchResult {
        let isbn10 = self
            .isbn
            .as_ref()
            .and_then(|isbns| isbns.iter().find(|s| s.len() == 10).cloned())
            .unwrap_or_default();
        let isbn13 = self
            .isbn
            .as_ref()
            .and_then(|isbns| isbns.iter().find(|s| s.len() == 13).cloned())
            .unwrap_or_default();

        BookSearchResult {
            title: self.title.unwrap_or_default(),
            author: self.author_name.map(|a| a.join(", ")).unwrap_or_default(),
            isbn10,
            isbn13,
        }
    }
}

#[async_trait]
impl BookSearchProvider for OpenLibraryProvider {
    async fn search(&self, query: &str, limit: usize) -> eyre::Result<Vec<BookSearchResult>> {
        let fetch_limit = (limit * 2).to_string();
        let http_resp = self
            .client
            .get("https://openlibrary.org/search.json")
            .query(&[
                ("q", query),
                ("limit", &fetch_limit),
                ("fields", "title,author_name,isbn"),
            ])
            .send()
            .await?;

        let status = http_resp.status();
        if !status.is_success() {
            tracing::warn!(status = %status, query = %query, "open library returned non-success status");
            return Ok(vec![]);
        }

        let resp = http_resp.json::<OpenLibraryResponse>().await?;
        tracing::debug!(query = %query, result_count = %resp.docs.len(), "open library search completed");

        let results = resp.docs.into_iter().map(|d| d.into_result()).collect();
        Ok(dedup_results(results, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_merges_case_insensitive_titles() {
        let results = vec![
            BookSearchResult {
                title: "Rhythm of War".into(),
                author: "Brandon Sanderson".into(),
                isbn10: "".into(),
                isbn13: "9781250757302".into(),
            },
            BookSearchResult {
                title: "Rhythm Of War".into(),
                author: "".into(),
                isbn10: "1250759781".into(),
                isbn13: "".into(),
            },
        ];

        let deduped = dedup_results(results, 10);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].title, "Rhythm of War");
        assert_eq!(deduped[0].author, "Brandon Sanderson");
        assert_eq!(deduped[0].isbn10, "1250759781");
        assert_eq!(deduped[0].isbn13, "9781250757302");
    }

    #[test]
    fn dedup_keeps_better_entry_as_base() {
        let results = vec![
            BookSearchResult {
                title: "some book".into(),
                author: "".into(),
                isbn10: "".into(),
                isbn13: "".into(),
            },
            BookSearchResult {
                title: "Some Book".into(),
                author: "Author Name".into(),
                isbn10: "1234567890".into(),
                isbn13: "1234567890123".into(),
            },
        ];

        let deduped = dedup_results(results, 10);
        assert_eq!(deduped.len(), 1);
        // The second entry had higher quality, so its title wins
        assert_eq!(deduped[0].title, "Some Book");
        assert_eq!(deduped[0].author, "Author Name");
    }

    #[test]
    fn dedup_preserves_unique_entries() {
        let results = vec![
            BookSearchResult {
                title: "Book A".into(),
                author: "Author A".into(),
                isbn10: "".into(),
                isbn13: "".into(),
            },
            BookSearchResult {
                title: "Book B".into(),
                author: "Author B".into(),
                isbn10: "".into(),
                isbn13: "".into(),
            },
        ];

        let deduped = dedup_results(results, 10);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn dedup_respects_limit() {
        let results: Vec<BookSearchResult> = (0..20)
            .map(|i| BookSearchResult {
                title: format!("Book {i}"),
                author: "".into(),
                isbn10: "".into(),
                isbn13: "".into(),
            })
            .collect();

        let deduped = dedup_results(results, 5);
        assert_eq!(deduped.len(), 5);
    }
}
