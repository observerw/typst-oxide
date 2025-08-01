use crate::parser::models::Wikilink;
use regex::Regex;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum WikilinkError {
    #[error("regex compilation failed")]
    RegexError(#[from] regex::Error),
}

pub struct WikilinkParser {
    wikilink_regex: Regex,
}

impl WikilinkParser {
    pub fn new() -> Result<Self, WikilinkError> {
        // Matches: [[target]], [[target|alias]], [[target:label]], [[target:label|alias]]
        let regex = Regex::new(r"\[\[([^|\]:\n]+)(?::([^|\]\n]+))?(?:\|([^|\]\n]+))?\]\]")?;
        
        Ok(Self {
            wikilink_regex: regex,
        })
    }

    pub fn parse_wikilinks(&self,
        content: &str,
        file_path: &Path,
    ) -> Vec<Wikilink> {
        let mut wikilinks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            for cap in self.wikilink_regex.captures_iter(line) {
                let full_match = cap.get(0).unwrap();
                let target = cap.get(1).unwrap().as_str().to_string();
                let label = cap.get(2).map(|m| m.as_str().to_string());
                let alias = cap.get(3).map(|m| m.as_str().to_string());

                let column = line[..full_match.start()].chars().count() + 1;

                wikilinks.push(Wikilink {
                    target,
                    alias,
                    label,
                    line: line_idx + 1,
                    column,
                });
            }
        }

        wikilinks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_simple_wikilink() {
        let parser = WikilinkParser::new().unwrap();
        let content = "This is a [[simple]] wikilink.";
        let path = PathBuf::from("test.typ");
        
        let wikilinks = parser.parse_wikilinks(content, &path);
        
        assert_eq!(wikilinks.len(), 1);
        assert_eq!(wikilinks[0].target, "simple");
        assert_eq!(wikilinks[0].alias, None);
        assert_eq!(wikilinks[0].label, None);
        assert_eq!(wikilinks[0].line, 1);
        assert_eq!(wikilinks[0].column, 11);
    }

    #[test]
    fn test_parse_wikilink_with_alias() {
        let parser = WikilinkParser::new().unwrap();
        let content = "Link to [[target|alias name]].";
        let path = PathBuf::from("test.typ");
        
        let wikilinks = parser.parse_wikilinks(content, &path);
        
        assert_eq!(wikilinks.len(), 1);
        assert_eq!(wikilinks[0].target, "target");
        assert_eq!(wikilinks[0].alias, Some("alias name".to_string()));
        assert_eq!(wikilinks[0].label, None);
    }

    #[test]
    fn test_parse_wikilink_with_label() {
        let parser = WikilinkParser::new().unwrap();
        let content = "Link to [[file:section]].";
        let path = PathBuf::from("test.typ");
        
        let wikilinks = parser.parse_wikilinks(content, &path);
        
        assert_eq!(wikilinks.len(), 1);
        assert_eq!(wikilinks[0].target, "file");
        assert_eq!(wikilinks[0].alias, None);
        assert_eq!(wikilinks[0].label, Some("section".to_string()));
    }

    #[test]
    fn test_parse_wikilink_with_label_and_alias() {
        let parser = WikilinkParser::new().unwrap();
        let content = "Link to [[file:section|Section 1]].";
        let path = PathBuf::from("test.typ");
        
        let wikilinks = parser.parse_wikilinks(content, &path);
        
        assert_eq!(wikilinks.len(), 1);
        assert_eq!(wikilinks[0].target, "file");
        assert_eq!(wikilinks[0].alias, Some("Section 1".to_string()));
        assert_eq!(wikilinks[0].label, Some("section".to_string()));
    }

    #[test]
    fn test_parse_multiple_wikilinks() {
        let parser = WikilinkParser::new().unwrap();
        let content = "First [[link1]] and second [[link2|alias]].";
        let path = PathBuf::from("test.typ");
        
        let wikilinks = parser.parse_wikilinks(content, &path);
        
        assert_eq!(wikilinks.len(), 2);
        assert_eq!(wikilinks[0].target, "link1");
        assert_eq!(wikilinks[1].target, "link2");
    }

    #[test]
    fn test_parse_multiline_wikilinks() {
        let parser = WikilinkParser::new().unwrap();
        let content = "First line\n[[line2]] on second line\n[[line3|alias]] on third";
        let path = PathBuf::from("test.typ");
        
        let wikilinks = parser.parse_wikilinks(content, &path);
        
        assert_eq!(wikilinks.len(), 2);
        assert_eq!(wikilinks[0].target, "line2");
        assert_eq!(wikilinks[0].line, 2);
        assert_eq!(wikilinks[1].target, "line3");
        assert_eq!(wikilinks[1].line, 3);
    }

    #[test]
    fn test_parse_file_with_extension() {
        let parser = WikilinkParser::new().unwrap();
        let content = "See [[document.pdf]] for details.";
        let path = PathBuf::from("test.typ");
        
        let wikilinks = parser.parse_wikilinks(content, &path);
        
        assert_eq!(wikilinks.len(), 1);
        assert_eq!(wikilinks[0].target, "document.pdf");
    }
}