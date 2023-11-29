/*!
This crate provides a convenient way to traverse a directory recursively.
The objects in this crate can be used seamlessly with the standard library
types (`std::fs::*`) since [`Entry`] is based on `std::fs::DirEntry`.
The entry point of this crate is the [`Walker`] (builder) struct. Use the [`new`] function
passing the entry point of the traversal as input to configure the `Walker`. Several
options can be specified:
- use the method [`skip_dotted`] to skip dotted files
or directories during traversal.
- The method [`skip_directories`] allows to skip directories.
- Use [`max_depth`] to stop the traversal at a fixed depth.

All of the above are optional. After setting the options use [`walk_dir`]
to traverse the file system starting from the `root`.

The result of the traversal is a recursively built [`Entry`] object that
exposes its information in its [`dirent`] field and lists its dependencies
in the [`children`] field.

[`new`]: struct.Walker.html#method.new
[`skip_dotted`]: struct.Walker.html#method.skip_dotted
[`skip_directories`]: struct.Walker.html#method.skip_directories
[`max_depth`]: struct.Walker.html#method.max_depth
[`walk_dir`]: struct.Walker.html#method.walk_dir
[`dirent`]: struct.Value.html#structfield.dirent
[`children`]: struct.Value.html#structfield.children

To use this crate, add `dir_walker` as a dependency to your project's
`Cargo.toml`:

```toml
[dependencies]
dir_walker = "0.1"
```

# Example
```
# use dir_walker::Walker;
let entries = Walker::new("./src")
    .max_depth(2)  // optional
    .skip_dotted()  // optional
    .walk_dir()
    .unwrap();

// print the directory tree as nested objects
println!("entries:\n{entries:?}");

// into_iter() iterates over a flat "list" of entries.
// Print a depth first representation of the root directory
entries.into_iter().for_each(|e| println!("{e:?}"));

// output:
// (DirEntry("./src"), 0)
// (DirEntry("./src/lib.rs"), 1)
// (DirEntry("./tests"), 0)
// (DirEntry("./tests/walkdir.rs"), 1)
// (DirEntry("./Cargo.lock"), 0)
// (DirEntry("./Cargo.toml"), 0)
```

```

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
    skip_directories: Vec<String>,
    /// Max depth for the traversal
    max_depth: usize,
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
    /// let walker = Walker::new("./");
    /// let entries = walker.walk_dir().unwrap();
    /// ```
    pub fn new(root: impl AsRef<std::path::Path>) -> Walker {
        Walker {
            root: root.as_ref().to_path_buf(),
            skip_dotted: Default::default(),
            skip_directories: Default::default(),
            max_depth: std::usize::MAX,
        }
    }

    /// Skip dotted paths (files and directories) while traversing the file system
    ///
    ///  # Example
    /// ```
    /// # use dir_walker::Walker;
    /// let root = "./";
    /// let skip = ["./target"];
    /// let entries = Walker::new(root)
    ///     .skip_dotted()
    ///     .walk_dir()
    ///     .unwrap();
    /// ```
    pub fn skip_dotted(mut self) -> Walker {
        self.skip_dotted = true;
        self
    }

    /// Skip `directories` while traversing the file system
    ///
    /// # Arguments
    ///
    /// `directories` - array of directories to skip during traversal
    pub fn skip_directories(mut self, directories: &[impl AsRef<std::path::Path>]) -> Walker {
        self.skip_directories = directories
            .iter()
            .map(|d| d.as_ref().display().to_string())
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
    /// let walker = Walker::new("./").max_depth(2);
    /// let entries = walker.walk_dir().unwrap();
    /// ````
    pub fn max_depth(mut self, depth: usize) -> Walker {
        self.max_depth = depth;
        self
    }

    /// Returns a nested structure that represents the entries inside the `root` directory
    /// and its sub-directories in a depth first order, directories first and files last.
    ///
    /// Arguments
    ///
    ///  `path` - root path to walk into
    ///
    /// # Example
    /// ```
    /// # use dir_walker::Walker;
    /// let walker = Walker::new("./");
    /// let entries = walker.walk_dir().unwrap();
    /// ````
    pub fn walk_dir(&self) -> Result<Entry, std::io::Error> {
        if self.root.is_file() {
            let parent = self
                .root
                .parent()
                .expect("Error: could not get the parent directory of this file");

            for entry in read_dir(parent)? {
                let entry = entry?;
                if entry.path() == self.root.as_path() {
                    let entry = Entry::new(Vec::new(), Some(entry), 0);
                    return Ok(entry);
                }
            }
        }

        let children = self.walk_dir_inner(&self.root, 0, self.max_depth)?;
        let entries = Entry::new(children, None, 0);

        Ok(entries)
    }

    /// Returns a nested structures that represents the children of the input path
    /// and its sub-directories. The structure is computed visiting directories and their
    /// sub-directories.
    fn walk_dir_inner(
        &self,
        path: impl AsRef<std::path::Path>,
        depth: usize,
        max_depth: usize,
    ) -> Result<Vec<Entry>, std::io::Error> {
        let mut children: Vec<Entry> = Vec::new();
        let entries = self.read_entries(&path)?;

        for entry in entries.into_iter() {
            if depth <= max_depth {
                children.push(Entry::new(
                    self.walk_dir_inner(entry.path().as_path(), depth + 1, max_depth)?,
                    Some(entry),
                    depth,
                ));
            }
        }
        Ok(children)
    }

    /// Returns a vector of directories and files in alphabetic order (directories first)
    /// found in the given path
    fn read_entries(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Vec<DirEntry>, std::io::Error> {
        let mut paths: Vec<DirEntry> = Vec::new();

        let mut dirs = self.get_entries(&path, true).unwrap();
        let mut files = self.get_entries(&path, false).unwrap();

        paths.append(&mut dirs);
        paths.append(&mut files);

        Ok(paths)
    }

    /// Returns a vector of `DirEnt` representing entries found inside the input `entry`.
    /// If `dirs_only` is true, this function returns directories, if false, it
    /// returns files.
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
        !((self.skip_dotted & path_str.contains("/.")) | self.skip_directories.contains(&path_str))
    }
}

/// Value of an `Entry`. See the documentation for [`Entry`].
#[derive(Debug)]
pub struct Value {
    /// The `std::fs::DirEnt` corresponding to this entry.
    pub dirent: Option<DirEntry>,
    /// Directories and files inside this directory.
    pub children: Vec<Entry>,
    /// The depth of this entry with respect to the root.
    pub depth: usize,
}

/// Represents a directory with its sub-directories and files. The depth
/// field represents the depth of this entry with respect to the root path.
#[derive(Debug)]
pub struct Entry(pub Value);

impl Entry {
    pub(crate) fn new(children: Vec<Entry>, dirent: Option<DirEntry>, depth: usize) -> Entry {
        Entry(Value {
            children,
            dirent,
            depth,
        })
    }

    /// Find a file by its name and extension. If the file
    /// that is sought for has no valid name or the file is not found, `None`
    /// is returned. If there are multiple files with the same name in different
    /// directories, the first occurrence of the file is returned.
    ///
    /// Arguments
    ///
    /// * `name` - The name of the file sought for
    ///
    /// Example
    /// ```
    /// # use dir_walker::Walker;
    /// let walker = Walker::new("./");
    /// let entries = walker.walk_dir().unwrap();
    /// let found = entries.find("lib.rs");
    /// println!("Found file: {found:?}");
    /// ```
    pub fn find(self, name: &str) -> Option<Entry> {
        let mut queue: VecDeque<Entry> = VecDeque::new();

        queue.push_back(self);

        while let Some(mut node) = queue.pop_front() {
            if let Some(ref dirent) = node.0.dirent {
                if let Some(label) = dirent.file_name().to_str() {
                    if label == name {
                        return Some(node);
                    }
                }
            }

            node.0.children.reverse();
            let children = VecDeque::from(node.0.children);
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
/// # use dir_walker::Walker;
/// let entries = Walker::new("./").walk_dir().unwrap();
/// entries.into_iter().for_each(|e| println!("{e:?}"));
/// ```
#[derive(Debug)]
pub struct EntryIterator {
    /// `std::fs::DirEntry` object with directory information
    pub dirent: DirEntry,
    /// depth of this entry in the file system
    pub depth: usize,
}

impl EntryIterator {
    pub fn new(dirent: DirEntry, depth: usize) -> EntryIterator {
        EntryIterator { dirent, depth }
    }
}

/// An implementation of IntoIterator iterates over a flat list
/// of entries in depth first order, directories first, files last, entering
/// each directory
impl IntoIterator for Entry {
    type Item = EntryIterator;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut queue: VecDeque<Entry> = VecDeque::new();
        let mut flat_vec: Vec<EntryIterator> = Vec::new();

        queue.push_back(self);

        while let Some(mut node) = queue.pop_front() {
            if let Some(dirent) = node.0.dirent {
                flat_vec.push(EntryIterator::new(dirent, node.0.depth));
            }

            node.0.children.reverse();
            let children = VecDeque::from(node.0.children);
            children.into_iter().for_each(|c| queue.push_front(c));
        }

        flat_vec.into_iter()
    }
}
