use std::convert::TryFrom;

use super::{raw, Error, Format, Result};
use crate::{Index, LineNumber};

impl Format<'_> {
    /// Looks up an instruction address in the SymCache, yielding an iterator of [`SourceLocation`]s.
    ///
    /// This always returns an iterator, however that iterator might be empty in case no [`SourceLocation`]
    /// was found for the given `addr`.
    pub fn lookup(&self, addr: u64) -> SourceLocationIter<'_> {
        let source_location_start = self.source_locations.len() - self.ranges.len();
        let relative_addr = self.offset_addr(addr);
        let source_location_idx = relative_addr.and_then(|relative_addr| {
            // Ranges are sorted in ascending order, so consider the two cases:
            // Address is out of bounds (first simple test assert)
            // Ranges: [4344, 4336]      Desired address: [4335]
            // binary_search_by_key returns Err(2) because 4335 can be inserted after 4336, so if
            // the index is close to the end of the list it is above and out of bounds
            //
            // Address is between bounds (second simple test assert)
            // Ranges: [4344, 4336]      Desired address: [4338]
            // binary_search_by_key returns Err(1) because 4338 can be inserted after 4344, so just
            // add 1 to the start to get the correct fn of interest: 4336 at index 1
            //
            //
            // Something to consider: When does 4344 end? Previously there was an additional range
            // with addr = u32::MAX which is currently not being pushed. Is that needed to indicate
            // the end of the final range?
            match self.ranges.binary_search_by_key(&relative_addr, |r| r.0) {
                Ok(idx) => {
                    Some(Index::try_from(source_location_start + idx).unwrap())
                }
                Err(idx) if idx == self.ranges.len() => {
                    None
                }
                Err(idx) => {
                    Some(Index::try_from(source_location_start + idx).unwrap())
                }
            }
        });
        SourceLocationIter {
            format: self,
            source_location_idx,
        }
    }

    fn get_file(&self, file_idx: Index) -> Result<File<'_>> {
        match self.files.get::<usize>(file_idx.into()) {
            Some(file) => Ok(File { format: self, file }),
            None => Err(Error::InvalidFileReference(file_idx)),
        }
    }
}

/// A source File included in the SymCache.
///
/// Source files can have up to three path prefixes/fragments.
/// They are in the order of `comp_dir`, `directory`, `path_name`.
/// If a later fragment is an absolute path, it overrides the previous fragment.
///
/// The [`File::full_path`] method yields the final concatenated and resolved path.
///
/// # Examples
///
/// Considering that a C project is being compiled inside the `/home/XXX/sentry-native/` directory,
/// - The `/home/XXX/sentry-native/src/sentry_core.c` may have the following fragments:
///   - comp_dir: /home/XXX/sentry-native/
///   - directory: -
///   - path_name: src/sentry_core.c
/// - The included file `/usr/include/pthread.h` may have the following fragments:
///   - comp_dir: /home/XXX/sentry-native/ <- The comp_dir is defined, but overrided by the dir below
///   - directory: /usr/include/
///   - path_name: pthread.h
#[derive(Debug)]
pub struct File<'data> {
    format: &'data Format<'data>,
    file: &'data raw::File,
}

impl<'data> File<'data> {
    /// Resolves the compilation directory of this source file.
    pub fn comp_dir(&self) -> Option<Result<&'data str>> {
        self.file
            .comp_dir_idx
            .map(|idx| self.format.get_string(idx))
    }

    /// Resolves the parent directory of this source file.
    pub fn directory(&self) -> Option<Result<&'data str>> {
        self.file
            .directory_idx
            .map(|idx| self.format.get_string(idx))
    }

    /// Resolves the final path name fragment of this source file.
    pub fn path_name(&self) -> Result<&'data str> {
        if let Some(idx) = self.file.path_name_idx {
            self.format.get_string(idx)
        } else {
            Ok("")
        }
    }

    /// Resolves and concatenates the full path based on its individual fragments.
    pub fn full_path(&self) -> Result<String> {
        let comp_dir = self.comp_dir().unwrap_or(Ok(""))?;
        let directory = self.directory().unwrap_or(Ok(""))?;
        let path_name = self.path_name()?;

        let prefix = symbolic_common::join_path(comp_dir, directory);
        let full_path = symbolic_common::join_path(&prefix, path_name);
        Ok(full_path)
    }
}

/// A Function definition as included in the SymCache.
#[derive(Debug)]
pub struct Function<'data> {
    format: &'data Format<'data>,
    function: &'data raw::Function,
}

impl<'data> Function<'data> {
    /// The possibly mangled name/symbol of this function.
    pub fn name(&self) -> Result<&'data str> {
        self.format.get_string(self.function.name_idx)
    }
}

/// An Iterator that yields [`SourceLocation`]s, representing an inlining hierarchy.
#[derive(Debug)]
pub struct SourceLocationIter<'data> {
    format: &'data Format<'data>,
    source_location_idx: Option<Index>,
}

impl<'data> SourceLocationIter<'data> {
    /// Yields the next [`SourceLocation`] in the inlining hierarchy.
    // We return a `Result` here, so its not a *real* `Iterator`
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<SourceLocation<'data>>> {
        match self.source_location_idx {
            None => Ok(None),
            Some(idx) => match self.format.source_locations.get::<usize>(idx.into()) {
                Some(source_location) => {
                    self.source_location_idx = source_location.inlined_into_idx;
                    Ok(Some(SourceLocation {
                        format: self.format,
                        source_location,
                    }))
                }
                None => Err(Error::InvalidSourceLocationReference(idx)),
            },
        }
    }
}

/// A Source Location as included in the SymCache.
///
/// The source location represents a `(function, file, line, inlined_into)` tuple corresponding to
/// an instruction in the executable.
#[derive(Debug)]
pub struct SourceLocation<'data> {
    format: &'data Format<'data>,
    source_location: &'data raw::SourceLocation,
}

impl SourceLocation<'_> {
    /// The source line corresponding to the instruction.
    ///
    /// This might return `0` when no line information can be found.
    pub fn line(&self) -> Option<LineNumber> {
        self.source_location.line
    }

    /// The source file corresponding to the instruction.
    pub fn file(&self) -> Option<Result<File<'_>>> {
        self.source_location
            .file_idx
            .map(|idx| self.format.get_file(idx))
    }

    /// The function corresponding to the instruction.
    pub fn function(&self) -> Result<Option<Function<'_>>> {
        let function_idx = if let Some(idx) = self.source_location.function_idx {
            idx
        } else {
            return Ok(None);
        };
        match self.format.functions.get::<usize>(function_idx.into()) {
            Some(function) => Ok(Some(Function {
                format: self.format,
                function,
            })),
            None => Err(Error::InvalidFunctionReference(function_idx)),
        }
    }

    // TODO: maybe forward some of the `File` and `Function` accessors, such as:
    // `function_name` or `full_path` for convenience.
}
