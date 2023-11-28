use dir_walker::Walker;

#[test]
fn minimal_example() {
    let root = "./";
    let walker = Walker::new(root);
    let entries = walker.walk_dir().unwrap();

    // Depth first representation of the root directory
    entries.into_iter().for_each(|e| println!("{e:?}"));
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

    // Depth first representation of the root directory
    entries.into_iter().for_each(|e| println!("{e:?}"));
}

#[test]
fn should_find_lib() {
    let walker = Walker::new("./");
    let entries = walker.walk_dir().unwrap();
    let found = entries.find("lib.rs").unwrap();
    println!("Found file: {found:?}");
}

#[test]
fn should_stop_at_max_depth() {
    let entries = Walker::new("./")
        .max_depth(2)
        .skip_dotted()
        .walk_dir()
        .unwrap();

    // Depth first representation of the root directory
    entries.into_iter().for_each(|e| println!("{e:?}"));
}
