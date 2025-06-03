// Example 16: Search Service Implementation
//
// This example demonstrates how to build a search service with indexing,
// full-text search capabilities, and result ranking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

// Struct: Document
//
// Represents a document that can be indexed and searched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    id: Uuid,
    title: String,
    content: String,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

impl Document {
    pub fn new(title: String, content: String, tags: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            tags,
            metadata: HashMap::new(),
        }
    }
}

// Struct: SearchResult
//
// Represents a search result with relevance scoring.
#[derive(Debug, Clone)]
pub struct SearchResult {
    document: Document,
    score: f64,
    matched_terms: Vec<String>,
}

// Struct: SearchService
//
// Main search service that handles indexing and querying.
pub struct SearchService {
    documents: HashMap<Uuid, Document>,
    word_index: HashMap<String, Vec<Uuid>>,
}

impl Default for SearchService {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchService {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            word_index: HashMap::new(),
        }
    }

    pub fn index_document(&mut self, document: Document) {
        let doc_id = document.id;

        // Index words from title and content
        let words = self.extract_words(&format!("{} {}", document.title, document.content));

        for word in words {
            self.word_index
                .entry(word.to_lowercase())
                .or_default()
                .push(doc_id);
        }

        // Index tags
        for tag in &document.tags {
            self.word_index
                .entry(tag.to_lowercase())
                .or_default()
                .push(doc_id);
        }

        self.documents.insert(doc_id, document);
        info!("Indexed document: {}", doc_id);
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_terms: Vec<String> = self.extract_words(query);
        let mut doc_scores: HashMap<Uuid, (f64, Vec<String>)> = HashMap::new();

        for term in &query_terms {
            let term_lower = term.to_lowercase();
            if let Some(doc_ids) = self.word_index.get(&term_lower) {
                for &doc_id in doc_ids {
                    let (score, matched_terms) =
                        doc_scores.entry(doc_id).or_insert((0.0, Vec::new()));
                    *score += 1.0; // Simple TF scoring
                    matched_terms.push(term.clone());
                }
            }
        }

        let mut results: Vec<SearchResult> = doc_scores
            .into_iter()
            .filter_map(|(doc_id, (score, matched_terms))| {
                self.documents.get(&doc_id).map(|doc| SearchResult {
                    document: doc.clone(),
                    score,
                    matched_terms,
                })
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    fn extract_words(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|word| word.chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|word: &String| !word.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        (self.documents.len(), self.word_index.len())
    }
}

// Function: demo_search_service
//
// Demonstrates the search service functionality.
fn demo_search_service() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Creating Search Service ===");
    let mut search_service = SearchService::new();

    // Index some sample documents
    search_service.index_document(Document::new(
        "Rust Programming Language".to_string(),
        "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_string(),
        vec!["programming".to_string(), "systems".to_string(), "rust".to_string()]
    ));

    search_service.index_document(Document::new(
        "Python for Data Science".to_string(),
        "Python is widely used in data science due to its simplicity and powerful libraries like pandas and numpy.".to_string(),
        vec!["python".to_string(), "data".to_string(), "science".to_string()]
    ));

    search_service.index_document(Document::new(
        "Web Development with JavaScript".to_string(),
        "JavaScript is essential for web development, both frontend and backend with Node.js."
            .to_string(),
        vec![
            "javascript".to_string(),
            "web".to_string(),
            "development".to_string(),
        ],
    ));

    info!("=== Performing Searches ===");

    // Search for "programming"
    let results = search_service.search("programming");
    info!(
        "Search results for 'programming': {} matches",
        results.len()
    );
    for result in results {
        info!("  - {} (score: {:.1})", result.document.title, result.score);
    }

    // Search for "data science"
    let results = search_service.search("data science");
    info!(
        "Search results for 'data science': {} matches",
        results.len()
    );
    for result in results {
        info!(
            "  - {} (score: {:.1}, matched: {:?})",
            result.document.title, result.score, result.matched_terms
        );
    }

    let (doc_count, term_count) = search_service.get_statistics();
    info!("=== Search Service Statistics ===");
    info!("Documents indexed: {}", doc_count);
    info!("Unique terms: {}", term_count);

    Ok(())
}

// Function: main
//
// Entry point demonstrating the search service implementation.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Search Service Example");
    demo_search_service()?;
    info!("Search Service Example completed successfully");

    Ok(())
}
