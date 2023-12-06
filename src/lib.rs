/*!
This crate provides a convenient way to traverse a directory recursively.
The objects in this crate can be used seamlessly with the standard library
types (`std::fs::*`) since [`Entry`] is based on `std::fs::DirEntry`. The
goal of this crate is to provide a file system representation with guaranteed
order and serializability allowing to send the serialized object over a network.
## Features
- `Entry` is an in-memory recursive structure that guarantees the order of the paths
that have been found during traversal. The order is alphabetic, directories first,
files last. To limit memory consumption the default value for the maximum
number of visited entries is limited to `10k` and the maximum depth of traversal to `100`.
These limit can be changed with the methods [`max_entries`] and [`max_depth`].
- `Entry` can be used to build objects that can be serialized e.g. as Json, due
to it being in-memory.
- Symbolic links are skipped.

## Use
The entry point of this crate is the [`Walker`] (builder) struct. Use the [`new`] function
passing the entry point of the traversal as input to configure the `Walker`.
Then several options can be specified:
- use the method [`skip_dotted`] to skip dotted files
or directories during traversal.
- The method [`skip_directories`] allows to skip directories.
- Use [`max_depth`] to stop the traversal at a fixed depth.
- Use [`max_entries`] to set the maximum number of visited entries during traversal.

All of the above are optional. After setting the options use [`walk_dir`]
to traverse the file system starting from the `root`.

The result of the traversal is a recursively built [`Entry`] object that
exposes its information in its [`dirent`] field and lists its dependencies
in the [`children`] field.
Alternatively a flat list of entries is available to the [`iterator`] of the
[`Entry`] object.

[`new`]: struct.Walker.html#method.new
[`skip_dotted`]: struct.Walker.html#method.skip_dotted
[`skip_directories`]: struct.Walker.html#method.skip_directories
[`max_depth`]: struct.Walker.html#method.max_depth
[`max_entries`]: struct.Walker.html#method.max_entries
[`walk_dir`]: struct.Walker.html#method.walk_dir
[`dirent`]: struct.Value.html#structfield.dirent
[`children`]: struct.Value.html#structfield.children
[`iterator`]: struct.Entry.html#method.into_iter

To use this crate, add `dir_walker` as a dependency to your project's
`Cargo.toml`:

```toml
[dependencies]
dir_walker = "0.1"
```

## Example: Print the nested structure
```
# use dir_walker::{Walker, EntryItem};
# use std::path::Path;
let skip = ["./target"];
let max_entries = 8;
let max_depth = 3;
let entries = Walker::new("./")
    .max_entries(max_entries)  // optional
    .max_depth(max_depth)  // optional
    .skip_directories(&skip)  // optional
    .skip_dotted()  // optional
    .walk_dir()
    .unwrap();

// print the directory tree as nested objects
println!("entries:\n{:?}", &entries);

# let items = entries
#   .into_iter()
#   .inspect(|e| println!("{e:?}"))
#   .collect::<Vec<EntryItem>>();
# assert_eq!(items.len(), max_entries);
```
## Example: Get a flat list of entries
```
# use dir_walker::{Walker, EntryItem};
# use std::path::Path;
let max_entries = 4;
let entries = Walker::new("./")
    .max_entries(max_entries)
    .walk_dir()
    .unwrap();

// into_iter() iterates over a flat "list" of entries.
// Print a depth first representation of the root directory
let items = entries.into_iter().inspect(|e| println!("{e:?}")).collect::<Vec<EntryItem>>();

# assert_eq!(items.len(), max_entries);
```

*/

use std::collections::VecDeque;
use std::fmt::Debug;
use std::fs::{read_dir, DirEntry};
use std::path::PathBuf;

/// Configure this builder and use the method [`walk_dir`] to traverse
/// the root path. See [`Walker::new()`] for examples.
///
/// [`walk_dir`]: struct.Walker.html#method.walk_dir
pub struct Walker {
    /// root path to start the traversal
    root: PathBuf,
    /// IF true, skip all paths starting with a dot (dot files and directories)
    skip_dotted: bool,
    /// A vec of paths to skip
    skip_directories: Vec<std::path::PathBuf>,
    /// Maximum depth for the traversal
    max_depth: usize,
    /// Maximum number of traversed entries
    max_entries: usize,
    _counter: usize,
}

/// A builder to traverse the file system.
impl Walker {
    /// Returns an instance of a builder that allows to traverse the file system
    /// starting from the input `root` path with the method `walk_dir`.
    /// If `root` is a directory, `walk_dir` returns the entries (of type `Entry`)
    /// inside that directory, and all the nested entries.
    /// If `root` is a file, then it is the only returned entry.
    ///
    /// # Arguments
    ///
    /// `root` - The starting path of the traversal
    ///
    /// # Examples
    /// ```
    /// # use dir_walker::Walker;
    /// let mut walker = Walker::new("./");
    /// let entries = walker.walk_dir().unwrap();
    /// ```
    pub fn new(root: impl AsRef<std::path::Path>) -> Walker {
        Walker {
            root: root.as_ref().to_path_buf(),
            skip_dotted: Default::default(),
            skip_directories: Default::default(),
            max_entries: 10_000,
            max_depth: 100,
            _counter: 0,
        }
    }

    /// Skip dotted paths (files and directories) while traversing the file system
    ///
    ///  # Example
    /// ```
    /// # use dir_walker::Walker;
    /// # use std::path::Path;
    /// let root = "./";
    /// let entries = Walker::new(root)
    ///     .skip_dotted()
    ///     .walk_dir()
    ///     .unwrap();
    ///
    /// # let p = Path::new("./.git").canonicalize().unwrap();
    /// # entries.into_iter().for_each(|e| assert_ne!(e.dirent.path(), p))
    /// ```
    pub fn skip_dotted(mut self) -> Walker {
        self.skip_dotted = true;
        self
    }

    /// Skip `directories` while traversing the file system.
    ///
    /// # Arguments
    ///
    /// `directories` - array of directories to skip during traversal
    ///
    /// # Example
    /// ```
    /// # use dir_walker::Walker;
    /// # use std::path::Path;
    /// let root = "./";
    /// let skip = ["./target"];
    /// let entries = Walker::new(root)
    ///     .skip_directories(&skip)
    ///     .walk_dir()
    ///     .unwrap();
    ///
    /// # let p = Path::new("./target").canonicalize().unwrap();
    /// # entries.into_iter().for_each(|e| assert_ne!(e.dirent.path(), p))
    /// ```
    pub fn skip_directories(mut self, directories: &[impl AsRef<std::path::Path>]) -> Walker {
        self.skip_directories = directories
            .iter()
            .map(|d| d.as_ref().canonicalize().unwrap())
            .collect();
        self
    }

    /// Limit the traversal of the file system up to this maximum depth of nesting.
    ///
    /// # Arguments
    ///
    /// * `max_depth` - maximum level of nesting before stopping the traversal
    ///
    /// # Example
    /// ```
    /// # use dir_walker::Walker;
    /// let mut walker = Walker::new("./").max_depth(2);
    /// let entries = walker.walk_dir().unwrap();
    /// ````
    pub fn max_depth(mut self, depth: usize) -> Walker {
        self.max_depth = depth;
        self
    }

    /// Limit the number of visited entries
    ///
    /// # Arguments
    ///
    /// * `max` - maximum number of entries visited during traversal
    pub fn max_entries(mut self, max: usize) -> Walker {
        self.max_entries = max;
        self
    }

    /// Returns a recursive structure that represents the entries inside the `root` directory
    /// and its sub-directories in a depth first order, directories first and files last.
    /// Symbolic links are skipped.
    ///
    /// # Arguments
    ///
    ///  `path` - root path to walk into
    ///
    /// # Example
    /// ```
    /// # use dir_walker::Walker;
    /// # use std::path::Path;
    /// let entries = Walker::new("./src").walk_dir().unwrap();
    /// # let dirent = entries.dirent.unwrap();
    /// # let p = Path::new("./src").canonicalize().unwrap();
    /// # assert_eq!(dirent.path(), p);
    /// ```
    pub fn walk_dir(&mut self) -> Result<Entry, std::io::Error> {
        let root = self.root.canonicalize()?;
        let root_entry = get_parent_entry(&root)?;

        let children = self.walk_dir_inner(&root, 0)?;
        let entries = Entry::new(children, Some(root_entry), 0);

        Ok(entries)
    }

    /// Returns a recursive structure that represents the children of the input path
    /// and its sub-directories. The structure is computed visiting directories and their
    /// sub-directories.
    fn walk_dir_inner(
        &mut self,
        path: impl AsRef<std::path::Path>,
        depth: usize,
    ) -> Result<Vec<Entry>, std::io::Error> {
        let mut children: Vec<Entry> = Vec::new();
        let entries = self.read_entries(&path)?;

        for entry in entries.into_iter() {
            self._counter += 1;

            if self._counter == self.max_entries {
                return Ok(children);
            }

            if depth <= self.max_depth {
                children.push(Entry::new(
                    self.walk_dir_inner(entry.path().as_path(), depth + 1)?,
                    Some(entry),
                    depth,
                ));

                if self._counter >= self.max_entries {
                    return Ok(children);
                }
            }
        }
        Ok(children)
    }

    /// Returns a vector of directories and files in alphabetic order (directories first)
    /// found in the given path.
    fn read_entries(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Vec<DirEntry>, std::io::Error> {
        let mut paths: Vec<DirEntry> = Vec::new();

        let mut dirs = self.get_entries(&path, true)?;
        let mut files = self.get_entries(&path, false)?;

        paths.append(&mut dirs);
        paths.append(&mut files);

        Ok(paths)
    }

    /// Returns a vector of entries (as `DirEnt`) representing entries found inside the input `entry`.
    /// If `dirs_only` is true, this function returns directories, if false, it returns files.
    /// Symblic links are skipped.
    fn get_entries(
        &self,
        entry: impl AsRef<std::path::Path>,
        dirs_only: bool,
    ) -> Result<Vec<DirEntry>, std::io::Error> {
        let mut entries: Vec<DirEntry> = Vec::new();
        if entry.as_ref().is_dir() {
            read_dir(entry)?
                .filter_map(|e| e.ok())
                .filter(|e| self.should_skip(e.path()))
                .filter(|e| !e.path().is_symlink())
                .filter(|e| {
                    if dirs_only {
                        e.path().is_dir()
                    } else {
                        e.path().is_file()
                    }
                })
                .for_each(|e| entries.push(e));

            entries.sort_by_key(|f| f.path());
        }

        Ok(entries)
    }

    fn should_skip(&self, path: impl AsRef<std::path::Path>) -> bool {
        let path_str = path.as_ref().display().to_string();
        !((self.skip_dotted & (path_str.contains("/.") | path_str.contains("\\.")))
            | self.skip_directories.contains(&path.as_ref().to_path_buf()))
    }
}

/// Represents a directory with its sub-directories and files. The depth
/// field represents the depth of this entry with respect to the root path.
#[derive(Debug)]
pub struct Entry {
    /// The `std::fs::DirEnt` corresponding to this entry.
    pub dirent: Option<DirEntry>,
    /// Directories and files inside this directory.
    pub children: Vec<Entry>,
    /// The depth of this entry with respect to the root.
    pub depth: usize,
}

impl Entry {
    pub(crate) fn new(children: Vec<Entry>, dirent: Option<DirEntry>, depth: usize) -> Entry {
        Entry {
            children,
            dirent,
            depth,
        }
    }

    /// Find a file by its name and extension. If the file
    /// that is sought for has no valid name or the file is not found, `None`
    /// is returned. If there are multiple files with the same name in different
    /// directories, the first occurrence of the file is returned.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the file sought for
    ///
    /// # Example
    /// ```
    /// # use dir_walker::Walker;
    /// # use std::path::Path;
    /// let mut walker = Walker::new("./src");
    /// let entries = walker.walk_dir().unwrap();
    /// let found = entries.find("lib.rs").unwrap();
    /// # let p = Path::new("./src/lib.rs").canonicalize().unwrap();
    /// assert_eq!(found.dirent.unwrap().path(), p)
    /// ```
    pub fn find(self, name: &str) -> Option<Entry> {
        let mut queue: VecDeque<Entry> = VecDeque::new();

        queue.push_back(self);

        while let Some(mut node) = queue.pop_front() {
            if let Some(ref dirent) = node.dirent {
                if let Some(label) = dirent.file_name().to_str() {
                    if label == name {
                        return Some(node);
                    }
                }
            }

            node.children.reverse();
            let children = VecDeque::from(node.children);
            children.into_iter().for_each(|c| queue.push_front(c));
        }
        None
    }
}

/// Helper type that is returned when iterating over an [`Entry`].
///
/// # Example
///
/// ```
/// # use dir_walker::{Walker, EntryItem};
/// let entries = Walker::new("./").max_entries(6).walk_dir().unwrap();
/// let items = entries.into_iter().inspect(|e| println!("{e:?}")).collect::<Vec<EntryItem>>();
/// # assert_eq!(items.len(), 6)
/// ```
#[derive(Debug)]
pub struct EntryItem {
    /// `std::fs::DirEntry` object with directory information
    pub dirent: DirEntry,
    /// depth of this entry in the file system
    pub depth: usize,
}

impl EntryItem {
    pub fn new(dirent: DirEntry, depth: usize) -> EntryItem {
        EntryItem { dirent, depth }
    }
}

/// An implementation of IntoIterator iterates over a flat list
/// of entries in depth first order, directories first, files last, entering
/// each directory
impl IntoIterator for Entry {
    type Item = EntryItem;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut queue: VecDeque<Entry> = VecDeque::new();
        let mut flat_vec: Vec<EntryItem> = Vec::new();

        queue.push_back(self);

        while let Some(mut node) = queue.pop_front() {
            if let Some(dirent) = node.dirent {
                flat_vec.push(EntryItem::new(dirent, node.depth));
            }

            node.children.reverse();
            let children = VecDeque::from(node.children);
            children.into_iter().for_each(|c| queue.push_front(c));
        }

        flat_vec.into_iter()
    }
}

/// Returns the parent entry (as `DirEntry`) of self.root
fn get_parent_entry(path: &PathBuf) -> Result<DirEntry, std::io::Error> {
    let invalid_input_err = |msg: &str| std::io::Error::new(std::io::ErrorKind::InvalidInput, msg);

    let parent_entry = path.parent().unwrap();

    let entry = read_dir(parent_entry)
        .expect("Error: could not get the parent directory of the root")
        .filter_map(|e| e.ok())
        .filter(|e| e.path() == path.as_path())
        .collect::<Vec<DirEntry>>();

    let root_entry = entry.into_iter().next().ok_or(invalid_input_err(
        "Error: could not find the root directory",
    ));

    root_entry
}
