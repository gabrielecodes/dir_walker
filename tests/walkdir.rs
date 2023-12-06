use dir_walker::{EntryItem, Walker};
use std::fs::{read_dir, DirEntry};
use std::path::Path;

#[test]
fn minimal_example() {
    let path = "./src";
    let mut walker = Walker::new(path);
    let entries = walker.walk_dir().unwrap();

    // print the directory tree as nested objects
    println!("entries:\n{entries:?}");

    // match "./src"
    let dirent = entries.dirent.unwrap();
    let target_entry = Path::new("./src").canonicalize().unwrap();
    assert_eq!(target_entry, dirent.path());

    // match "./src/lib.rs"
    let dirent = entries.children.into_iter().next().unwrap().dirent.unwrap();
    let target_entry = Path::new(&"./src/lib.rs").canonicalize().unwrap();
    assert_eq!(target_entry, dirent.path());
}

#[test]
fn should_skip_entries() {
    let root = "./";
    let skip = ["./target"];
    let entries = Walker::new(root)
        .skip_directories(&skip)
        .skip_dotted()
        .walk_dir()
        .unwrap();

    let target_entry = Path::new("./target").canonicalize().unwrap();
    let git_entry = Path::new("./.git").canonicalize().unwrap();
    let github_entry = Path::new("./.github").canonicalize().unwrap();

    // test absence of ./target, ./.git and ./.github in "entries"
    entries
        .into_iter()
        .inspect(|e| println!("{e:?}"))
        .for_each(|e| {
            assert_ne!(e.dirent.path(), git_entry);
            assert_ne!(e.dirent.path(), github_entry);
            assert_ne!(e.dirent.path(), target_entry);
        });
}

#[test]
fn should_visit_max_entries() {
    let max_entries = 8;
    let max_depth = 3;
    let root = "./";
    let skip = ["./target"];

    let entries = Walker::new(root)
        .max_entries(max_entries)
        .max_depth(max_depth)
        .skip_directories(&skip)
        .skip_dotted()
        .walk_dir()
        .unwrap();

    let target_entry = Path::new("./target").canonicalize().unwrap();
    let git_entry = Path::new("./.git").canonicalize().unwrap();
    let github_entry = Path::new("./.github").canonicalize().unwrap();

    let items = entries
        .into_iter()
        .inspect(|e| println!("{e:?}"))
        .map(|e| {
            assert_ne!(e.dirent.path(), git_entry);
            assert_ne!(e.dirent.path(), github_entry);
            assert_ne!(e.dirent.path(), target_entry);
            e
        })
        .collect::<Vec<EntryItem>>();

    assert_eq!(items.len(), max_entries);
}

#[test]
fn should_find_lib() {
    let mut walker = Walker::new("./src");
    let entries = walker.walk_dir().unwrap();
    let found = entries.find("lib.rs").unwrap();

    let lib_path = Path::new("./src/lib.rs").canonicalize().unwrap();

    assert_eq!(found.dirent.unwrap().path(), lib_path);
}

#[test]
fn should_stop_at_max_depth() {
    let entries = Walker::new("./src").max_depth(2).walk_dir().unwrap();

    entries.into_iter().for_each(|e| assert!(e.depth <= 2));
}

#[test]
fn should_walk_single_file() {
    let entries = Walker::new("./src/lib.rs").walk_dir().unwrap();
    let entries = entries
        .into_iter()
        .map(|e| e.dirent.path())
        .collect::<Vec<std::path::PathBuf>>();

    let target_entry = Path::new("./src/lib.rs").canonicalize().unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries.into_iter().next().unwrap(), target_entry)
}
