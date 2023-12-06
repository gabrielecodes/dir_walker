use dir_walker::Walker;
use std::fs::{read_dir, DirEntry};

fn get_direntry(path: impl AsRef<std::path::Path>, root: &str) -> Result<DirEntry, std::io::Error> {
    let entry = read_dir(root)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() == path.as_ref())
        .collect::<Vec<DirEntry>>();

    entry.into_iter().next().ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Error: could not find 'path' in the root",
    ))
}

#[test]
fn minimal_example() {
    let path = "./src";
    let walker = Walker::new(path);
    let entries = walker.walk_dir().unwrap();

    // print the directory tree as nested objects
    println!("entries:\n{entries:?}");

    // match "./src"
    let dirent = entries.dirent.unwrap();
    let target_entry = get_direntry("./src", "./").unwrap();
    assert_eq!(target_entry.path(), dirent.path());

    // match "./src/lib.rs"
    let dirent = entries.children.into_iter().next().unwrap().dirent.unwrap();
    let target_entry = get_direntry(&"./src/lib.rs", "./src").unwrap();
    assert_eq!(target_entry.path(), dirent.path());
}

#[test]
fn should_skip_entries() {
    let root = "./src";
    let skip = ["./target"];
    let entries = Walker::new(root)
        .skip_directories(&skip)
        .skip_dotted()
        .walk_dir()
        .unwrap();

    let target_entry = get_direntry("./target", "./").unwrap();

    // test absence of ./target in "entries"
    entries
        .into_iter()
        .inspect(|e| println!("{e:?}"))
        .for_each(|e| assert_ne!(e.dirent.path(), target_entry.path()));
}

#[test]
fn should_find_lib() {
    let walker = Walker::new("./src");
    let entries = walker.walk_dir().unwrap();
    let found = entries.find("lib.rs").unwrap();

    let lib = get_direntry("./src/lib.rs", "./src").unwrap();

    assert_eq!(found.dirent.unwrap().path(), lib.path());
}

#[test]
fn should_stop_at_max_depth() {
    let entries = Walker::new("./src").max_depth(2).walk_dir().unwrap();

    entries.into_iter().for_each(|e| assert!(e.depth <= 2));
}

#[test]
fn should_walk_single_file() {
    let entries = Walker::new("./src/lib.rs").walk_dir().unwrap();

    let lib_entry = get_direntry("./src/lib.rs", "./src").unwrap();

    entries.into_iter().for_each(|e| println!("{e:?}"));
    let target_entry = get_direntry("./src/lib.rs", "./src").unwrap();

    assert_eq!(lib_entry.path(), target_entry.path())
}
