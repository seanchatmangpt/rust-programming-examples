# Recursive Directory Walk

## Context

You are building a file system utility that needs to traverse directory structures. This could be a file copying tool, a search utility, a backup program, a file synchronization tool, or any application that needs to visit every file and subdirectory within a directory tree.

The file system has a hierarchical structure where directories can contain files and other directories. Your code needs to handle this recursive structure robustly, dealing with different file types (regular files, directories, symbolic links) and potential I/O errors.

## Problem

**How do you traverse a directory tree recursively, processing all files and subdirectories while handling errors, different file types, and avoiding common pitfalls like infinite loops from circular symlinks?**

Simple recursive directory traversal can lead to stack overflow on deep hierarchies, fail to handle symlinks properly, or miss error conditions. Rust's ownership system adds complexity around passing paths and handling directory iterators across recursive calls.

## Forces

- **Recursion depth**: Deep directory trees could overflow the stack
- **Error handling**: Many operations can fail (permissions, I/O errors, broken symlinks)
- **File types**: Must handle regular files, directories, symlinks, and special files differently
- **Symlinks**: Can create circular references leading to infinite recursion
- **Performance**: Minimize system calls and memory allocations
- **Cross-platform**: Behavior differs between Unix and Windows
- **Ownership**: Directory entries must be consumed from iterators carefully

## Solution

**Use `read_dir()` to get a directory iterator, process each entry by checking its file type with `file_type()`, and recurse on directories.** Handle each entry's `Result`, check file types explicitly, and pass owned `PathBuf` values to recursive calls.

### Structure

```rust
use std::fs;
use std::io;
use std::path::Path;

fn walk_directory(dir: &Path, process_fn: &dyn Fn(&Path) -> io::Result<()>) -> io::Result<()> {
    for entry_result in dir.read_dir()? {
        let entry = entry_result?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            walk_directory(&path, process_fn)?;
        } else {
            process_fn(&path)?;
        }
    }
    Ok(())
}
```

### Real Implementation (from copy)

```rust
use std::fs;
use std::io;
use std::path::Path;

/// Copy the existing directory `src` to the target path `dst`.
fn copy_dir_to(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.is_dir() {
        fs::create_dir(dst)?;
    }

    for entry_result in src.read_dir()? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        copy_to(&entry.path(), &file_type, &dst.join(entry.file_name()))?;
    }

    Ok(())
}

/// Copy whatever is at `src` to the target path `dst`.
fn copy_to(src: &Path, src_type: &fs::FileType, dst: &Path) -> io::Result<()> {
    if src_type.is_file() {
        fs::copy(src, dst)?;
    } else if src_type.is_dir() {
        copy_dir_to(src, dst)?;
    } else if src_type.is_symlink() {
        let target = src.read_link()?;
        symlink(target, dst)?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("don't know how to copy: {}", src.display())
        ));
    }
    Ok(())
}
```

### Key Elements

1. **read_dir() returns iterator**: Each call to `read_dir()` returns `ReadDir`, an iterator over `Result<DirEntry, io::Error>`
2. **Double error handling**: `entry_result?` unwraps the outer Result, `entry.file_type()?` handles potential metadata errors
3. **file_type() for dispatch**: Use `file_type()` to determine how to process the entry
4. **Recursion on directories**: Call the function recursively for subdirectories
5. **Path construction**: Use `entry.path()` for full paths, `entry.file_name()` for just the name
6. **join() for paths**: Build new paths with `dst.join(entry.file_name())`

### Error Handling Pattern

```rust
// TWO levels of Result unwrapping:

// 1. read_dir() can fail (permissions, path doesn't exist)
for entry_result in src.read_dir()? {
    // 2. Each entry iteration can fail (I/O errors)
    let entry = entry_result?;

    // 3. Getting metadata/file type can fail
    let file_type = entry.file_type()?;

    // 4. Processing the entry can fail
    process_entry(&entry.path(), &file_type)?;
}
```

## Resulting Context

### Benefits

- **Complete traversal**: Visits every accessible file and directory
- **Error propagation**: Failures bubble up with context
- **Type-safe dispatch**: File type determines processing logic
- **Composable**: Can combine with other file operations
- **Memory efficient**: Only current directory's entries in memory at once

### Liabilities

- **Stack depth**: Very deep hierarchies can overflow the stack
- **Symlink loops**: Infinite recursion possible without cycle detection
- **Performance**: Recursive calls have overhead; iterative solutions can be faster
- **Error interruption**: First error stops traversal; partial work may be done
- **Metadata costs**: `file_type()` requires a system call on some platforms

### Performance Characteristics

- **System calls**: One `read_dir()` per directory, one `file_type()` per entry
- **Memory**: O(depth) stack space for recursion
- **Worst case**: Deeply nested single-child directories

## Variations

### Iterative Traversal (Stack-Based)

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::collections::VecDeque;

fn walk_iterative(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(root.to_path_buf());

    while let Some(dir) = queue.pop_front() {
        for entry_result in dir.read_dir()? {
            let entry = entry_result?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                queue.push_back(path); // Add to queue for later
            } else {
                result.push(path); // Collect files
            }
        }
    }

    Ok(result)
}
```

### Depth Limit

```rust
use std::fs;
use std::io;
use std::path::Path;

fn walk_limited(dir: &Path, max_depth: usize) -> io::Result<()> {
    walk_recursive(dir, max_depth, 0)
}

fn walk_recursive(dir: &Path, max_depth: usize, current_depth: usize) -> io::Result<()> {
    if current_depth >= max_depth {
        return Ok(());
    }

    for entry_result in dir.read_dir()? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            walk_recursive(&entry.path(), max_depth, current_depth + 1)?;
        } else {
            process_file(&entry.path())?;
        }
    }

    Ok(())
}
```

### Symlink Cycle Detection

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

fn walk_with_cycle_detection(dir: &Path, visited: &mut HashSet<PathBuf>) -> io::Result<()> {
    let canonical = fs::canonicalize(dir)?;

    if !visited.insert(canonical.clone()) {
        // Already visited - cycle detected
        return Ok(());
    }

    for entry_result in dir.read_dir()? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            walk_with_cycle_detection(&entry.path(), visited)?;
        } else if file_type.is_symlink() {
            // Follow symlink carefully
            if let Ok(target) = entry.path().read_link() {
                if target.is_dir() {
                    walk_with_cycle_detection(&target, visited)?;
                }
            }
        } else {
            process_file(&entry.path())?;
        }
    }

    Ok(())
}
```

### Parallel Traversal

```rust
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

fn walk_parallel(root: &Path) -> io::Result<Vec<PathBuf>> {
    let (tx, rx) = mpsc::channel();
    let root = root.to_path_buf();

    thread::spawn(move || {
        walk_and_send(&root, &tx);
    });

    Ok(rx.iter().collect())
}

fn walk_and_send(dir: &Path, tx: &mpsc::Sender<PathBuf>) {
    if let Ok(entries) = dir.read_dir() {
        for entry_result in entries {
            if let Ok(entry) = entry_result {
                let path = entry.path();

                if path.is_dir() {
                    walk_and_send(&path, tx);
                } else {
                    let _ = tx.send(path);
                }
            }
        }
    }
}
```

## Related Patterns

- **Error Propagation**: Essential for handling filesystem errors
- **Platform-Specific Code**: Symlink handling varies by platform
- **Path Manipulation**: Building and joining paths correctly
- **Iterator Processing**: Directory entries are iterators

## Known Uses

### Standard Tools

- **cp -r**: Recursive copy command
- **find**: Search for files in directory hierarchy
- **du**: Disk usage analyzer
- **tree**: Display directory structure
- **rsync**: File synchronization tool

### Real Projects

```rust
// File search tool
use std::path::{Path, PathBuf};

fn find_files_by_extension(dir: &Path, ext: &str) -> io::Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    find_recursive(dir, ext, &mut results)?;
    Ok(results)
}

fn find_recursive(dir: &Path, ext: &str, results: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry_result in dir.read_dir()? {
        let entry = entry_result?;
        let path = entry.path();

        if path.is_dir() {
            find_recursive(&path, ext, results)?;
        } else if path.extension().map_or(false, |e| e == ext) {
            results.push(path);
        }
    }
    Ok(())
}

// Disk usage calculator
fn calculate_size(dir: &Path) -> io::Result<u64> {
    let mut total = 0;

    for entry_result in dir.read_dir()? {
        let entry = entry_result?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            total += calculate_size(&path)?;
        } else {
            total += metadata.len();
        }
    }

    Ok(total)
}

// Backup tool with filtering
fn backup_directory<P: Fn(&Path) -> bool>(
    src: &Path,
    dst: &Path,
    predicate: &P
) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry_result in src.read_dir()? {
        let entry = entry_result?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if !predicate(&path) {
            continue; // Skip filtered entries
        }

        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            backup_directory(&path, &dest_path, predicate)?;
        } else if file_type.is_file() {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}
```

### Using the walkdir Crate

For production use, consider the `walkdir` crate which handles edge cases:

```rust
use walkdir::WalkDir;

fn walk_with_library(root: &Path) -> io::Result<()> {
    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        println!("{}", entry.path().display());
    }
    Ok(())
}
```

## Implementation Notes

### Handling Permissions Errors

```rust
fn walk_robust(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let entries = match dir.read_dir() {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            eprintln!("Permission denied: {}", dir.display());
            return Ok(files); // Skip this directory
        }
        Err(e) => return Err(e),
    };

    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error reading entry: {}", e);
                continue; // Skip this entry
            }
        };

        // Continue processing...
    }

    Ok(files)
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_recursive_copy() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        // Create test structure
        fs::create_dir(&src).unwrap();
        fs::write(src.join("file.txt"), "content").unwrap();
        fs::create_dir(src.join("subdir")).unwrap();
        fs::write(src.join("subdir/nested.txt"), "nested").unwrap();

        // Copy recursively
        copy_dir_to(&src, &dst).unwrap();

        // Verify
        assert!(dst.join("file.txt").exists());
        assert!(dst.join("subdir/nested.txt").exists());
    }
}
```

### Performance Optimization

```rust
// Pre-allocate when result size is known
fn collect_files_fast(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::with_capacity(100); // Pre-allocate

    // Use metadata() only when needed
    for entry_result in dir.read_dir()? {
        let entry = entry_result?;

        // is_dir() doesn't require metadata on Unix
        if entry.path().is_dir() {
            let mut subfiles = collect_files_fast(&entry.path())?;
            files.append(&mut subfiles);
        } else {
            files.push(entry.path());
        }
    }

    Ok(files)
}
```

## References

- Rust std::fs documentation
- "Programming Rust" Chapter 18: Input and Output
- walkdir crate: https://docs.rs/walkdir
- ignore crate (respects .gitignore): https://docs.rs/ignore
