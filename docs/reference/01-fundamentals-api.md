# Fundamentals API Reference

This reference provides complete API documentation for the fundamental command-line utilities in the repository.

## gcd - Greatest Common Divisor Calculator

### Command-Line Interface

```
gcd NUMBER ...
```

**Arguments:**
- `NUMBER ...`: One or more unsigned 64-bit integers

**Exit Codes:**
- `0`: Success
- `1`: No arguments provided or parsing error

**Example:**
```bash
$ gcd 14 15
The greatest common divisor of [14, 15] is 1

$ gcd 330 462
The greatest common divisor of [330, 462] is 66
```

### Function API

#### `gcd`

Computes the greatest common divisor using Euclid's algorithm.

**Signature:**
```rust
fn gcd(mut n: u64, mut m: u64) -> u64
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `n` | `u64` | First positive integer |
| `m` | `u64` | Second positive integer |

**Returns:**
- `u64`: The greatest common divisor of `n` and `m`

**Panics:**
- If `n == 0` or `m == 0`

**Complexity:**
- Time: O(log(min(n, m)))
- Space: O(1)

**Example:**
```rust
let result = gcd(2 * 3 * 5 * 11 * 17, 3 * 7 * 11 * 13 * 19);
assert_eq!(result, 3 * 11);
```

---

## copy - File and Directory Copy Utility

### Command-Line Interface

```
copy FILE... DESTINATION
```

**Arguments:**
- `FILE...`: One or more source files or directories
- `DESTINATION`: Target file or directory

**Behavior:**
- Single source + directory destination: Copies source into destination directory
- Single source + file destination: Copies source to destination path
- Multiple sources: All sources copied into destination directory (must be a directory)

**Exit Codes:**
- `0`: Success
- Non-zero: I/O error occurred

**Example:**
```bash
$ copy file.txt backup/
$ copy dir1/ dir2/ target/
```

### Function API

#### `copy_dir_to`

Recursively copies a directory to a new location.

**Signature:**
```rust
fn copy_dir_to(src: &Path, dst: &Path) -> io::Result<()>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `src` | `&Path` | Source directory path |
| `dst` | `&Path` | Destination directory path |

**Returns:**
- `io::Result<()>`: Success or I/O error

**Behavior:**
- Creates destination directory if it doesn't exist
- Recursively copies all entries

---

#### `copy_to`

Copies a file system entry (file, directory, or symlink) to a destination.

**Signature:**
```rust
fn copy_to(src: &Path, src_type: &fs::FileType, dst: &Path) -> io::Result<()>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `src` | `&Path` | Source path |
| `src_type` | `&fs::FileType` | Type of the source entry |
| `dst` | `&Path` | Destination path |

**Returns:**
- `io::Result<()>`: Success or I/O error

**Supported Types:**
- Regular files
- Directories (recursive)
- Symbolic links (Unix only)

---

#### `copy_into`

Copies a source into a destination directory, preserving the source name.

**Signature:**
```rust
fn copy_into<P, Q>(source: P, destination: Q) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `source` | `P: AsRef<Path>` | Source path |
| `destination` | `Q: AsRef<Path>` | Destination directory |

**Returns:**
- `io::Result<()>`: Success or I/O error

**Example:**
```rust
copy_into("file.txt", "backup/")?;
// Creates: backup/file.txt
```

---

#### `dwim_copy`

"Do What I Mean" copy - intelligently determines copy behavior.

**Signature:**
```rust
fn dwim_copy<P, Q>(source: P, destination: Q) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `source` | `P: AsRef<Path>` | Source path |
| `destination` | `Q: AsRef<Path>` | Destination path |

**Returns:**
- `io::Result<()>`: Success or I/O error

**Behavior:**
- If destination is a directory: Calls `copy_into`
- Otherwise: Calls `copy_to` with exact path

---

## grep - Text Search Utility

### Command-Line Interface

```
grep PATTERN FILE...
```

**Arguments:**
- `PATTERN`: String pattern to search for
- `FILE...`: Optional list of files to search (if omitted, reads from stdin)

**Output:**
- Prints each line containing the pattern to stdout

**Exit Codes:**
- `0`: Success
- `1`: Error occurred

**Example:**
```bash
$ grep "error" logfile.txt
$ echo "hello world" | grep "world"
world
```

### Function API

#### `grep`

Searches for lines containing a target string in a buffered reader.

**Signature:**
```rust
fn grep<R>(target: &str, reader: R) -> io::Result<()>
where
    R: BufRead
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `target` | `&str` | String pattern to search for |
| `reader` | `R: BufRead` | Buffered input source |

**Returns:**
- `io::Result<()>`: Success or I/O error

**Behavior:**
- Reads input line by line
- Prints lines containing `target` to stdout
- Uses `String::contains` for matching (substring match)

**Complexity:**
- Time: O(n × m) where n = number of lines, m = average line length
- Space: O(k) where k = maximum line length

**Example:**
```rust
use std::io::BufReader;

let data = b"hello\nworld\nhello world\n";
let reader = BufReader::new(&data[..]);
grep("world", reader)?;
// Prints:
// world
// hello world
```

---

## Error Handling

All three utilities follow Rust error handling best practices:

- Functions return `Result` types for operations that can fail
- Command-line interfaces print errors to stderr using `eprintln!`
- I/O errors are propagated using the `?` operator
- Panics are used only for programming errors (e.g., `gcd(0, 0)`)

## Platform Support

| Function | Unix | Windows | Notes |
|----------|------|---------|-------|
| `gcd` | ✓ | ✓ | Fully portable |
| `copy` (files) | ✓ | ✓ | Fully portable |
| `copy` (symlinks) | ✓ | ✗ | Returns error on Windows |
| `grep` | ✓ | ✓ | Fully portable |
