# dir_walker

[<img alt="github" src="https://img.shields.io/badge/github-gabrielecodes/dir_walker-8DBFCB?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/gabrielecodes/dir_walker)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/gabrielecodes/dir_walker/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/gabrielecodes/dir_walker/actions?query=branch%3Amain)
[<img alt="crates.io" src="https://img.shields.io/crates/v/dir_walker.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/dir_walker)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-dir_walker-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/dir_walker/latest/dir_walker/)

This crate provides a convenient way to traverse a directory recursively.
The objects in this crate can be used seamlessly with the standard library
types (`std::fs::*`) since `Entry` is based on `std::fs::DirEntry`. The
goal of this crate is to provide a file system representation with guaranteed
order and serializability allowing to send the serialized object over a network.

## Features

- `Entry` is an in-memory recursive structure that guarantees the order of the paths
that have been found during traversal. The order is alphabetic, directories first,
files last. To limit memory consumption the default value for the maximum
number of visited entries is limited to `10k` and the maximum depth of traversal to `100`.
These limit can be changed with the methods `max_entries` and `max_depth`.
- `Entry` can be used to build objects that can be serialized e.g. as Json.
- Symbolic links are skipped.

## Use

The entry point of this crate is the `Walker` (builder) struct. Use the `new` function
passing the entry point of the traversal as input to configure the `Walker`.
Then several options can be specified:

- use the method `skip_dotted` to skip dotted files
or directories during traversal.
- The method `skip_directories` allows to skip directories.
- Use `max_depth` to stop the traversal at a fixed depth.
- Use `max_entries` to set the maximum number of visited entries during traversal.

All of the above are optional. After setting the options use `walk_dir`
to traverse the file system starting from the `root`.

The result of the traversal is a recursively built `Entry` object that
exposes its information in its `dirent` field and lists its dependencies
in the `children` field.
Alternatively a flat list of entries is available to the `iterator` of the
`Entry` object.

Add this crate to your project:

```toml
[dependencies]
dir_walker = "0.1.9"
```

## Examples

Usage examples are in the [tests](https://github.com/gabrielecodes/dir_walker/blob/master/tests/walkdir.rs) folder.

## Minimal Example

```rust
    use dir_walker::Walker;

    let root = "./";
    let walker = Walker::new(root);
    let entries = walker.walk_dir().unwrap();

    // prints a depth first representation of the root directory
    entries.into_iter().for_each(|e| println!("{e:?}"));
```

## Using options

```rust
    use dir_walker::Walker;

    let root = "./";
    let skip = ["./target"];
    let entries = Walker::new(root)
        .skip_directories(&skip)
        .skip_dotted()
        .walk_dir()
        .unwrap();

    entries.into_iter().for_each(|e| println!("{e:?}"));
```

prints:

```text
EntryIterator { dirent: DirEntry("./src"), depth: 0 }
EntryIterator { dirent: DirEntry("./src/lib.rs"), depth: 1 }
EntryIterator { dirent: DirEntry("./tests"), depth: 0 }
EntryIterator { dirent: DirEntry("./tests/walkdir.rs"), depth: 1 }
EntryIterator { dirent: DirEntry("./Cargo.lock"), depth: 0 }
EntryIterator { dirent: DirEntry("./Cargo.toml"), depth: 0 }
EntryIterator { dirent: DirEntry("./README.md"), depth: 0 }
```
