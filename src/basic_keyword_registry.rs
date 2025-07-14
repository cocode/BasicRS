use std::collections::HashMap;
use crate::basic_types::Token;

/// Registry that defines all BASIC keywords and their corresponding tokens
/// This serves as the single source of truth for all keyword definitions
pub struct KeywordRegistry {
    keywords: HashMap<&'static str, Token>,
}

impl KeywordRegistry {
    pub fn new() -> Self {
        let mut registry = KeywordRegistry {
            keywords: HashMap::new(),
        };
        registry.register_keywords();
        registry
    }

    fn register_keywords(&mut self) {
        // Define all keywords and their corresponding tokens
        self.keywords.insert("REM", Token::Rem);
        self.keywords.insert("LET", Token::Let);
        self.keywords.insert("PRINT", Token::Print);
        self.keywords.insert("INPUT", Token::Input);
        self.keywords.insert("IF", Token::If);
        self.keywords.insert("THEN", Token::Then);
        self.keywords.insert("ELSE", Token::Else);
        self.keywords.insert("FOR", Token::For);
        self.keywords.insert("TO", Token::To);
        self.keywords.insert("STEP", Token::Step);
        self.keywords.insert("NEXT", Token::Next);
        self.keywords.insert("GOTO", Token::Goto);
        self.keywords.insert("GOSUB", Token::Gosub);
        self.keywords.insert("RETURN", Token::Return);
        self.keywords.insert("END", Token::End);
        self.keywords.insert("STOP", Token::Stop);
        self.keywords.insert("DATA", Token::Data);
        self.keywords.insert("READ", Token::Read);
        self.keywords.insert("RESTORE", Token::Restore);
        self.keywords.insert("DIM", Token::Dim);
        self.keywords.insert("ON", Token::On);
        self.keywords.insert("DEF", Token::Def);
        self.keywords.insert("AND", Token::And);
        self.keywords.insert("OR", Token::Or);
        self.keywords.insert("NOT", Token::Not);
    }

    /// Get all keyword names
    pub fn get_keyword_names(&self) -> Vec<&'static str> {
        self.keywords.keys().copied().collect()
    }

    /// Check if a string is a keyword
    pub fn is_keyword(&self, name: &str) -> bool {
        self.keywords.contains_key(name)
    }

    /// Get the token for a keyword
    pub fn get_token_for_keyword(&self, name: &str) -> Option<Token> {
        self.keywords.get(name).cloned()
    }

    /// Get all keyword-token pairs
    pub fn get_keyword_token_pairs(&self) -> Vec<(&'static str, Token)> {
        self.keywords.iter().map(|(&k, v)| (k, v.clone())).collect()
    }
}

// Global singleton instance
lazy_static::lazy_static! {
    pub static ref KEYWORD_REGISTRY: KeywordRegistry = KeywordRegistry::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_registry_basic_functionality() {
        let registry = &*KEYWORD_REGISTRY;
        
        // Test keyword recognition
        assert!(registry.is_keyword("LET"));
        assert!(registry.is_keyword("PRINT"));
        assert!(registry.is_keyword("IF"));
        assert!(!registry.is_keyword("INVALID"));
        
        // Test token retrieval
        assert_eq!(registry.get_token_for_keyword("LET"), Some(Token::Let));
        assert_eq!(registry.get_token_for_keyword("PRINT"), Some(Token::Print));
        assert_eq!(registry.get_token_for_keyword("INVALID"), None);
    }

    #[test]
    fn test_all_keywords_present() {
        let registry = &*KEYWORD_REGISTRY;
        let keywords = registry.get_keyword_names();
        
        // Test that all expected keywords are present
        let expected = vec![
            "REM", "LET", "PRINT", "INPUT", "IF", "THEN", "ELSE",
            "FOR", "TO", "STEP", "NEXT", "GOTO", "GOSUB", "RETURN",
            "END", "STOP", "DATA", "READ", "RESTORE", "DIM", "ON",
            "DEF", "AND", "OR", "NOT"
        ];
        
        for expected_keyword in expected {
            assert!(keywords.contains(&expected_keyword), 
                "Missing keyword: {}", expected_keyword);
        }
    }

    #[test]
    fn test_keyword_token_pairs() {
        let registry = &*KEYWORD_REGISTRY;
        let pairs = registry.get_keyword_token_pairs();
        
        // Should have 25 keyword-token pairs
        assert_eq!(pairs.len(), 25);
        
        // Test a few specific mappings
        assert!(pairs.contains(&("LET", Token::Let)));
        assert!(pairs.contains(&("PRINT", Token::Print)));
        assert!(pairs.contains(&("FOR", Token::For)));
    }
} 