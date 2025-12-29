# Maintenance Guide

This guide covers ongoing maintenance tasks for the Clap Architecture Book.

## Regular Maintenance Tasks

### Weekly

- [ ] Check for mdbook updates: `cargo install mdbook --force`
- [ ] Review and merge content contributions
- [ ] Monitor deployment status

### Monthly

- [ ] Update clap dependency versions in examples
- [ ] Review and update deprecated API references
- [ ] Check all external links
- [ ] Update changelog if needed

### Quarterly

- [ ] Review overall book structure
- [ ] Update migration guides for new Clap versions
- [ ] Refresh case studies with current best practices
- [ ] Performance optimization review

## Content Updates

### Adding a New Chapter

1. Create the markdown file in the appropriate `src/partX-xxx/` directory:
   ```bash
   touch src/part2-core-patterns/new-chapter.md
   ```

2. Update `src/SUMMARY.md`:
   ```markdown
   - [New Chapter Title](./part2-core-patterns/new-chapter.md)
   ```

3. Write content following the style guide (see below)

4. Add any code examples to `clap-examples/`:
   ```bash
   mkdir clap-examples/examples/XX-new-example
   # Create Cargo.toml and src/main.rs
   ```

5. Update workspace `Cargo.toml`:
   ```toml
   members = [
       # ... existing members
       "examples/XX-new-example",
   ]
   ```

6. Build and verify:
   ```bash
   cd docs/clap-architecture-book && mdbook build
   cd ../../clap-examples && cargo build --workspace
   ```

### Updating Existing Content

1. Edit the relevant markdown file
2. If code examples changed, update `clap-examples/`
3. Rebuild and verify all examples compile
4. Update CHANGELOG.md

### Style Guide

#### Markdown Conventions

- Use ATX-style headers (`#`, `##`, etc.)
- Code blocks with language specification: ` ```rust `
- One sentence per line for easier diffs
- Use `**bold**` for emphasis, `*italic*` for terms
- Reference code examples with relative paths

#### Code Examples

- All examples must compile without errors
- Use `// ...` to indicate omitted code
- Include comments explaining non-obvious behavior
- Follow Rust API Guidelines naming conventions

#### Visual Diagrams

- Use ASCII art for diagrams (portable, version-controllable)
- Keep diagrams under 80 characters wide
- Add explanatory text below each diagram

## Dependency Management

### Updating Clap Version

1. Update `clap-examples/Cargo.toml`:
   ```toml
   [workspace.dependencies]
   clap = { version = "4.X", features = ["derive", "env", "cargo", "string"] }
   ```

2. Run tests:
   ```bash
   cd clap-examples
   cargo update
   cargo build --workspace
   cargo test --workspace
   ```

3. Review and fix any deprecation warnings

4. Update book content to reflect API changes

5. Update migration guide if major version change

### Updating mdbook

```bash
cargo install mdbook --force
mdbook --version
```

After updating, rebuild and verify output:
```bash
cd docs/clap-architecture-book
mdbook clean
mdbook build
```

## Link Checking

### Automated Link Check

Install and run `mdbook-linkcheck`:

```bash
cargo install mdbook-linkcheck
cd docs/clap-architecture-book
mdbook build
```

Add to `book.toml`:
```toml
[output.linkcheck]
follow-web-links = true
warning-policy = "warn"
```

### Manual Verification

Check critical links:
- Clap documentation: https://docs.rs/clap/
- Rust documentation: https://doc.rust-lang.org/
- Repository links in examples

## Search Index

The search index is automatically generated during build. To verify:

1. Build the book: `mdbook build`
2. Check `book/searchindex-*.js` exists
3. Open `book/index.html` and test search functionality

### Search Configuration

In `book.toml`:
```toml
[output.html.search]
enable = true
limit-results = 30
teaser-word-count = 30
copy-js = true
```

## Handling Issues

### Common Build Errors

**Error: Missing chapter file**
```
ERROR File not found: src/xxx.md
```
Solution: Create the file or remove from SUMMARY.md

**Error: Unclosed HTML tag**
```
WARN unclosed HTML tag `<T>` found
```
Solution: Escape angle brackets in generic types: `<code>&lt;T&gt;</code>` or use inline code.

**Error: Duplicate file**
```
ERROR Duplicate file in SUMMARY.md
```
Solution: Each file can only appear once as a main entry

### Content Corrections

1. Open an issue describing the correction needed
2. Create a branch: `git checkout -b fix/chapter-X-correction`
3. Make the correction
4. Submit a pull request with:
   - Description of the correction
   - Reference to any related issues
   - Verification that build passes

## Version Management

### Semantic Versioning

- **Major** (X.0.0): Significant content reorganization, new Clap major version
- **Minor** (0.X.0): New chapters, significant updates
- **Patch** (0.0.X): Typo fixes, minor corrections, clarifications

### Release Process

1. Update version in CHANGELOG.md
2. Tag the release:
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```
3. Update deployment if needed
4. Announce release if significant

## Contributor Guidelines

### Setting Up Development Environment

```bash
# Clone repository
git clone https://github.com/ProgrammingRust/code-examples.git
cd code-examples

# Install tools
cargo install mdbook

# Build book
cd docs/clap-architecture-book
mdbook serve  # Opens live preview

# In another terminal, test examples
cd clap-examples
cargo test --workspace
```

### Pull Request Checklist

- [ ] Book builds without errors: `mdbook build`
- [ ] Examples compile: `cargo build --workspace`
- [ ] Tests pass: `cargo test --workspace`
- [ ] Content follows style guide
- [ ] CHANGELOG updated (if applicable)

### Code of Conduct

All contributors should follow the Rust Code of Conduct:
https://www.rust-lang.org/policies/code-of-conduct

## Archival and Backup

### Git History

All content is version-controlled. Important milestones should be tagged.

### Export Options

- **PDF**: Use browser print function on `/print.html`
- **EPUB**: Consider `mdbook-epub` plugin if needed
- **Offline Archive**: `tar -czf book-archive.tar.gz book/`

## Contact and Support

- **Issues**: Open GitHub issue in repository
- **Discussions**: Use GitHub Discussions for questions
- **Security**: Report security issues privately to maintainers
