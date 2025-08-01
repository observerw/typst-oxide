use std::io::Write;
use tempfile::NamedTempFile;

use typst_oxide::parser::Parser;

fn create_test_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", content).unwrap();
    file
}

#[test]
fn test_parse_simple_typst_file() {
    let content = r#"#meta(
    title: "Simple Note",
    tags: ("tag1", "tag2"),
)

= Introduction

This is a [[simple-wikilink]] in the text.

== Details

More information can be found [[here|alias]] and [[other:section|Section]].

<explicit-label>
"#;

    let file = create_test_file(content);
    let parser = Parser::new().unwrap();

    let parsed = parser.parse_content(content, file.path()).unwrap();

    // Check metadata
    assert_eq!(parsed.metadata.title, None); // Will be None since we're using parse_content
    assert_eq!(parsed.metadata.tags.len(), 0);

    // Check wikilinks
    assert_eq!(parsed.wikilinks.len(), 3);
    assert_eq!(parsed.wikilinks[0].target, "simple-wikilink");
    assert_eq!(parsed.wikilinks[0].alias, None);
    assert_eq!(parsed.wikilinks[1].target, "here");
    assert_eq!(parsed.wikilinks[1].alias, Some("alias".to_string()));
    assert_eq!(parsed.wikilinks[2].target, "other");
    assert_eq!(parsed.wikilinks[2].label, Some("section".to_string()));
    assert_eq!(parsed.wikilinks[2].alias, Some("Section".to_string()));

    // Check labels
    let explicit_labels: Vec<_> = parsed.labels.iter().filter(|l| !l.is_implicit).collect();
    let implicit_labels: Vec<_> = parsed.labels.iter().filter(|l| l.is_implicit).collect();

    assert_eq!(explicit_labels.len(), 1);
    assert_eq!(explicit_labels[0].name, "explicit-label");

    assert_eq!(implicit_labels.len(), 2);
    assert_eq!(implicit_labels[0].name, "introduction");
    assert_eq!(implicit_labels[1].name, "details");
}

#[test]
fn test_parse_empty_file() {
    let content = "";
    let file = create_test_file(content);
    let parser = Parser::new().unwrap();

    let parsed = parser.parse_content(content, file.path()).unwrap();

    assert_eq!(parsed.wikilinks.len(), 0);
    assert_eq!(parsed.labels.len(), 0);
}

#[test]
fn test_parse_file_with_only_metadata() {
    let content = r#"#meta(
    title: "Metadata Only",
    tags: ("single"),
)
"#;

    let file = create_test_file(content);
    let parser = Parser::new().unwrap();

    let parsed = parser.parse_content(content, file.path()).unwrap();

    assert_eq!(parsed.wikilinks.len(), 0);
    assert_eq!(parsed.labels.len(), 0);
}

#[test]
fn test_parse_file_with_complex_wikilinks() {
    let content = r#"= Complex WikiLinks

Here are various wikilink formats:

1. [[simple]]
2. [[file.typ]]
3. [[file|alias]]
4. [[file:label]]
5. [[file:label|custom alias]]
6. [[deep/path/file.typ]]
7. [[../parent/file.typ|Parent File]]

<custom-label>
<label-with-numbers123>
<label_with_underscores>
"#;

    let file = create_test_file(content);
    let parser = Parser::new().unwrap();

    let parsed = parser.parse_content(content, file.path()).unwrap();

    // Check all wikilinks
    let targets: Vec<&str> = parsed.wikilinks.iter().map(|w| w.target.as_str()).collect();
    assert!(targets.contains(&"simple"));
    assert!(targets.contains(&"file.typ"));
    assert!(targets.contains(&"file"));
    assert!(targets.contains(&"deep/path/file.typ"));
    assert!(targets.contains(&"../parent/file.typ"));

    // Check aliases
    let aliases: Vec<Option<&str>> = parsed
        .wikilinks
        .iter()
        .map(|w| w.alias.as_deref())
        .collect();
    assert!(aliases.contains(&Some("alias")));
    assert!(aliases.contains(&Some("custom alias")));
    assert!(aliases.contains(&Some("Parent File")));

    // Check labels
    assert_eq!(parsed.labels.len(), 4); // 1 heading + 3 explicit labels

    let label_names: Vec<&str> = parsed.labels.iter().map(|l| l.name.as_str()).collect();
    assert!(label_names.contains(&"complex-wikilinks"));
    assert!(label_names.contains(&"custom-label"));
    assert!(label_names.contains(&"label-with-numbers123"));
    assert!(label_names.contains(&"label_with_underscores"));
}

#[test]
fn test_parse_multiline_content() {
    let content = r#"= First Section
Some content here.

== Second Section
With [[wikilink1]] and <label1>

=== Third Section
More [[wikilink2|alias]] and <label2>

This continues on multiple lines.

[[wikilink3]] at the end.
"#;

    let file = create_test_file(content);
    let parser = Parser::new().unwrap();

    let parsed = parser.parse_content(content, file.path()).unwrap();

    // Check wikilinks with line numbers
    assert_eq!(parsed.wikilinks.len(), 3);
    assert_eq!(parsed.wikilinks[0].line, 5);
    assert_eq!(parsed.wikilinks[1].line, 9);
    assert_eq!(parsed.wikilinks[2].line, 11);

    // Check labels with line numbers
    let explicit_labels: Vec<_> = parsed.labels.iter().filter(|l| !l.is_implicit).collect();
    assert_eq!(explicit_labels.len(), 2);
    assert_eq!(explicit_labels[0].line, 5);
    assert_eq!(explicit_labels[1].line, 9);

    let implicit_labels: Vec<_> = parsed.labels.iter().filter(|l| l.is_implicit).collect();
    assert_eq!(implicit_labels.len(), 3);
    assert_eq!(implicit_labels[0].line, 1);
    assert_eq!(implicit_labels[1].line, 5);
    assert_eq!(implicit_labels[2].line, 9);
}

#[test]
fn test_parse_edge_cases() {
    let content = r#"= Edge Cases

- [[Link with spaces]] - should work
- [[file-with-dashes]] - should work
- [[file_with_underscores]] - should work
- [[file.with.dots]] - should work
- [[UPPERCASE]] - should work

Labels:
- <UPPERCASE-LABEL>
- <123numeric>
- <label-with-many-dashes>
- <label_with_underscores>

== Section with numbers 123
=== Section with special chars: !@#$
"#;

    let file = create_test_file(content);
    let parser = Parser::new().unwrap();

    let parsed = parser.parse_content(content, file.path()).unwrap();

    // Check all edge cases handled correctly
    let targets: Vec<&str> = parsed.wikilinks.iter().map(|w| w.target.as_str()).collect();
    assert!(targets.contains(&"Link with spaces"));
    assert!(targets.contains(&"file-with-dashes"));
    assert!(targets.contains(&"file_with_underscores"));
    assert!(targets.contains(&"file.with.dots"));
    assert!(targets.contains(&"UPPERCASE"));

    let label_names: Vec<&str> = parsed.labels.iter().map(|l| l.name.as_str()).collect();
    assert!(label_names.contains(&"edge-cases"));
    assert!(label_names.contains(&"UPPERCASE-LABEL"));
    assert!(label_names.contains(&"123numeric"));
    assert!(label_names.contains(&"label-with-many-dashes"));
    assert!(label_names.contains(&"label_with_underscores"));
    assert!(label_names.contains(&"section-with-numbers-123"));
    assert!(label_names.contains(&"section-with-special-chars"));
}

#[test]
fn test_parse_wikilink_positions() {
    let content = r#"= Test
    [[indented-wikilink]]
	[[tab-indented-wikilink]]
[[start-of-line]]
    Text [[end-of-line]]
"#;

    let file = create_test_file(content);
    let parser = Parser::new().unwrap();

    let parsed = parser.parse_content(content, file.path()).unwrap();

    assert_eq!(parsed.wikilinks.len(), 4);

    // Check column positions
    let columns: Vec<usize> = parsed.wikilinks.iter().map(|w| w.column).collect();
    assert!(columns.iter().all(|&c| c > 0));

    // Check line positions
    let lines: Vec<usize> = parsed.wikilinks.iter().map(|w| w.line).collect();
    assert_eq!(lines, vec![2, 3, 4, 5]);
}

#[test]
fn test_parse_label_positions() {
    let content = r#"= Test
    <indented-label>
	<tab-indented-label>
<start-of-line-label>
    Text <end-of-line-label>
"#;

    let file = create_test_file(content);
    let parser = Parser::new().unwrap();

    let parsed = parser.parse_content(content, file.path()).unwrap();

    let explicit_labels: Vec<_> = parsed.labels.iter().filter(|l| !l.is_implicit).collect();
    assert_eq!(explicit_labels.len(), 4);

    // Check column positions
    let columns: Vec<usize> = explicit_labels.iter().map(|l| l.column).collect();
    assert!(columns.iter().all(|&c| c > 0));

    // Check line positions (labels are on lines 2-5)
    let lines: Vec<usize> = explicit_labels.iter().map(|l| l.line).collect();
    assert_eq!(lines, vec![2, 3, 4, 5]);
}
