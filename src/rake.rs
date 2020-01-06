use inner::NumberChecker;
use keyword::{KeywordScore, KeywordSort};
use regex::Regex;
use std::collections::HashMap;
use stopwords::StopWords;

/// Represents an instance of Rake type
#[derive(Debug, Clone)]
pub struct Rake {
    stop_words: StopWords,
    num_re: Regex,
    punc_re: Regex,
}

impl Rake {
    /// Create a new instance of `Rake`.
    /// `stop_words` is an instance of `StopWords` struct.
    pub fn new(stop_words: StopWords) -> Self {
        Rake {
            stop_words: stop_words,
            num_re: Regex::new(r"-?\p{N}+[./٫,']?\p{N}*").expect("bad regex"),
            punc_re: Regex::new(r"[^\P{P}-]|\s+-\s+").expect("bad regex"),
        }
    }

    /// Runs RAKE algorithm on `text` and returns a vector of keywords.
    /// The returned vector is sorted by score (from greater to less).
    pub fn run(&self, text: &str) -> Vec<KeywordScore> {
        let phrases = self.phrases(text);
        let word_scores = self.word_scores(&phrases);
        self.candidate_keywords(&phrases, word_scores)
    }

    fn candidate_keywords<'a>(
        &self,
        phrases: &[Vec<&'a str>],
        word_scores: HashMap<&'a str, f64>,
    ) -> Vec<KeywordScore> {
        let mut keyword_score = HashMap::with_capacity(phrases.len());
        phrases.iter().for_each(|phrase| {
            let mut candidate_score = 0f64;
            phrase
                .iter()
                .filter(|word| !self.is_number(word))
                .for_each(|word| candidate_score += word_scores[word]);
            *keyword_score.entry(phrase.join(" ")).or_insert(0f64) = candidate_score;
        });
        let mut keywords = KeywordScore::from_map(keyword_score);
        keywords.sort_by_score();
        keywords
    }

    fn word_scores<'a>(&'a self, phrases: &[Vec<&'a str>]) -> HashMap<&'a str, f64> {
        let mut word_freq = HashMap::new();
        let mut word_degree = HashMap::new();
        phrases.iter().for_each(|phrase| {
            let len: usize = phrase
                .iter()
                .map(|word| if self.is_number(word) { 0 } else { 1 })
                .sum();
            if len > 0 {
                phrase
                    .iter()
                    .filter(|word| !self.is_number(word))
                    .for_each(|word| {
                        *word_freq.entry(*word).or_insert(0) += 1;
                        *word_degree.entry(*word).or_insert(0) += len - 1;
                    });
            }
        });
        let mut word_score = HashMap::new();
        for (word, freq) in word_freq {
            word_score
                .entry(word)
                .or_insert((word_degree[word] + freq) as f64 / freq as f64);
        }
        word_score
    }

    fn phrases<'a>(&'a self, text: &'a str) -> Vec<Vec<&'a str>> {
        let mut phrases = Vec::new();
        self.punc_re.split(text).filter(|s| !s.is_empty()).for_each(|s| {
            let mut phrase = Vec::new();
            s.split_whitespace().for_each(|word| {
                if self.stop_words.contains(word.to_lowercase().as_str()) {
                    if !phrase.is_empty() {
                        phrases.push(phrase.clone());
                        phrase.clear();
                    }
                } else {
                    phrase.push(word);
                }
            });
            if !phrase.is_empty() {
                phrases.push(phrase);
            }
        });
        phrases
    }
}

impl NumberChecker<&str> for &crate::Rake {
    fn is_number(&self, s: &str) -> bool {
        self.num_re.is_match(s)
    }
}
