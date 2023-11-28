# dir_walker

This crate provides a convenient way to traverse a directory recursively.
The objects in this crate can be used seamlessly with the standard library
types (`std::fs::*`) since `Entry` is based on `std::fs::DirEntry`.
The entry point of this crate is the `Walker` struct. Use the [`new`] function
passing the entry point of the traversal as input to configure the `Walker`. Several
options can be specified:

- use the method `skip_dotted` to skip dotted files
or directories during traversal.
- The method `skip_directories` allows to skip directories.
- Use `max_depth` to stop the traversal at a fixed depth.

All of the above are optional. After setting the options use [`walk_dir`]
to traverse the file system starting from the `root`.

The result of the traversal is a recursively built [`Entry`] object that
exposes its information in its [`dirent`] field and lists its dependencies
in the [`children`] field.

# Examples

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
