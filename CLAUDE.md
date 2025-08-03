# PKM Language Server Specification

## Overview

A language server for Personal Knowledge Management (PKM) systems using Typst as the markup language. The server parses PKM files containing metadata, content, labels, and wikilinks, caching the extracted information in SQLite for fast LSP operations.

## Architecture

### Core Components

- **Parser**: Extracts metadata using `typst query <file> "metadata" --field value --one`, and labels/wikilinks via regex parsing
- **Database**: SQLite-based caching layer for parsed data
- **LSP Server**: Provides language server protocol services
- **File Watcher**: Monitors file changes for real-time updates

### File Format

PKM files use Typst syntax with special conventions:

```typ
// current.typ
#meta(
  title: "Note Title",
  tags: (
    "tag1",
    "tag2",
  )
)
 
= Wikilink Format

This is a wikilink: [[other]] that links the `other.typ` file.

This is a wikilink with alias: [[other|alias1]].

This is a wikilink with label: [[other:math]]. The label can be any heading like `== Section`, or label like `<label>`.

A complete example of wikilink: [[other:math|other-math]].

Wikilink can also links non-typ file like [[some-file.pdf]], which requires a file extension.

= Label Format

In typst, label is a name wrapped in `<>` syntax, e.g., `<my-label>`. A label's name can contain letters, numbers, `_`, `-`, `:`, and `...`.

// other.typ
#meta(
    title: "Other Note",
    alias: (
    "alias1",
    "alias2",
  ),
)

= Title

== Content

Some math equation: 
$
ee^(ii pi) + 1 = 0
$ <math>
```

## Database Schema

### Tables

- **files**: File metadata and content with labels
- **metadata**: Key-value metadata pairs for files
- **wikilinks**: Wiki-link references and targets
- **labels**: Explicit and implicit labels with positions

### Schema Details

```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    created_at DATETIME,
    modified_at DATETIME,
    last_parsed DATETIME
);

CREATE TABLE metadata (
    id INTEGER PRIMARY KEY,
    file_id INTEGER,
    key TEXT NOT NULL,
    value TEXT,
    FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE TABLE wikilinks (
    id INTEGER PRIMARY KEY,
    file_id INTEGER,
    target TEXT NOT NULL,
    alias TEXT,
    label TEXT,
    line INTEGER,
    column INTEGER,
    FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE TABLE labels (
    id INTEGER PRIMARY KEY,
    file_id INTEGER,
    name TEXT NOT NULL,
    line INTEGER,
    column INTEGER,
    is_implicit BOOLEAN,
    FOREIGN KEY (file_id) REFERENCES files(id)
);
```

## LSP Features

### Completion

- **Wikilink completion**: Suggests existing note titles as targets
- **Label completion**: Suggests labels within current file scope
- **Metadata completion**: Suggests existing metadata keys and values

### Navigation

- **Go to definition**: Jump to label definitions or linked notes
- **Find references**: Show all references to labels or notes

### Refactoring

- **Rename labels**: Update label names across all references
- **Rename notes**: Update wikilink targets when note titles change

### Diagnostics

- **Broken wikilinks**: Highlight links to non-existent notes
- **Duplicate labels**: Warn about label name conflicts
- **Syntax errors**: Show Typst parsing issues

## Protocol Extensions

### Custom Methods

- `pkm/forwardLinks`: Get all forward links from a file
- `pkm/backlinks`: Get all backlinks to a file
- `pkm/metadata`: Get metadata for a file or all metadata keys in workspace
- `pkm/graph`: Get knowledge graph representation

## Performance Considerations

### Caching Strategy

- Parse files on first access and cache results
- Incremental updates on file changes
- Background re-indexing for large workspaces

### Optimization

- Lazy loading of file content
- Indexed lookups for labels and wikilinks
- Connection pooling for SQLite queries

## Security

### File Access

- Restrict file access to workspace directory
- Validate file paths to prevent directory traversal
- Sanitize user input in queries

### Database Security

- Use parameterized queries to prevent SQL injection
- Limit query result sizes
- Implement proper error handling without information leakage

## Error Handling

### Recovery Strategies

- Graceful degradation on parse failures
- Fallback to previous successful parse
- Clear error messages for users

### Logging

- Structured logging for debugging
- Performance metrics for monitoring
- User-friendly error notifications
