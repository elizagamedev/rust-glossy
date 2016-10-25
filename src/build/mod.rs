extern crate glob;
extern crate regex;
#[cfg(feature = "optimizer")]
mod optimize;

use std::env;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use self::glob::glob;
use self::regex::Regex;

#[cfg(feature = "optimizer")]
use self::optimize::Optimizer;
#[cfg(not(feature = "optimizer"))]
type Optimizer = ();

/// The language that glossy will target when optimizing shaders.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Language {
    OpenGl,
    OpenGlEs20,
    OpenGlEs30,
}

/// Configuration for the glossy shader preprocessor.
pub struct Config {
    lang: Language,
    sources: Vec<Source>,
    includes: HashMap<String, String>,
    optimizer: Option<Optimizer>,
    preserve_line_info: bool,
    allow_untested: bool,
}

impl Config {
    /// Create a new blank configuration for loading GLSL shaders.
    pub fn new(lang: Language) -> Config {
        Config {
            lang: lang,
            sources: Vec::new(),
            includes: HashMap::new(),
            optimizer: None,
            preserve_line_info: true,
            allow_untested: false,
        }
    }

    /// Specify a glob pattern which adds the matching files to the list of shader sources as
    /// vertex shader sources.
    pub fn vertex(mut self, pattern: &str) -> Config {
        for entry in glob(pattern).unwrap() {
            if let Ok(entry) = entry {
                self.sources.push(Source::new(entry, SourceKind::Vertex));
            }
        }
        self
    }

    /// Specify a glob pattern which adds the matching files to the list of shader sources as
    /// fragment shader sources.
    pub fn fragment(mut self, pattern: &str) -> Config {
        for entry in glob(pattern).unwrap() {
            if let Ok(entry) = entry {
                self.sources.push(Source::new(entry, SourceKind::Fragment));
            }
        }
        self
    }

    /// Specify a glob pattern which adds the matching files to the list of shader sources.
    /// If the extension is "vert" or "frag", they will be loaded as vertex and fragment
    /// shaders respectively. If not, they will be added to the list of generic shader files.
    /// Thus this function should really only be used to add non-vertex or fragment shader sources,
    /// as these are the only kinds that glsl-optimizer can optimize.
    pub fn source(mut self, pattern: &str) -> Config {
        for entry in glob(pattern).unwrap() {
            if let Ok(entry) = entry {
                let kind = if let Some(ext) = entry.extension() {
                    match ext.to_str().unwrap() {
                        "vert" => SourceKind::Vertex,
                        "frag" => SourceKind::Fragment,
                        _ => SourceKind::Unknown,
                    }
                } else {
                    SourceKind::Unknown
                };
                self.sources.push(Source::new(entry, kind));
            }
        }
        self
    }

    /// Specify a glob pattern which adds the matching files to the list of include files.
    pub fn include(mut self, pattern: &str) -> Config {
        for entry in glob(pattern).unwrap() {
            if let Ok(entry) = entry {
                let fname = entry.file_name().unwrap().to_str().unwrap().to_string();
                self.includes.insert(fname, read_file(entry));
            }
        }
        self
    }

    /// Specify that the shaders should be optimized with glsl-optimizer. This is only available
    /// when the 'optimizer' feature is enabled (which is the default option).
    ///
    /// This also implicitly sets the `discard_line_info()` option.
    #[cfg(feature = "optimizer")]
    pub fn optimize(mut self) -> Config {
        self.preserve_line_info = false;
        self.optimizer = Some(Optimizer::new(self.lang));
        self
    }

    /// Do not preserve the line and file information in the shader.
    ///
    /// This option, when set, will strip all comments and empty lines, but will not modify
    /// whitespace or minify the shader otherwise. Most of the time, `optimize()` makes more sense
    /// than this option, though it might be useful for the very slight minification it can provide
    /// for shader sources that aren't supported by glsl-optimizer.
    pub fn discard_line_info(mut self) -> Config {
        self.preserve_line_info = false;
        self
    }

    /// Allow the optimizer to work on untested language versions.
    ///
    /// According to the glsl-optimizer readme, versions of GLSL beyond 1.20 are untested. This flag
    /// allows glsl-optimizer to try optimizing them regardless.
    #[cfg(feature = "optimizer")]
    pub fn allow_untested_versions(mut self) -> Config {
        self.allow_untested = true;
        self
    }

    /// Process the specified GLSL shaders.
    ///
    /// This function will panic if the source files cannot be written, a recursive inclusion is
    /// detected, or if an error occurs during optimization. The optimizer can thus serve as a
    /// compile-time shader validator to a certain extent. Note that the error messages generated
    /// by the compiler will have inaccurate file line information, as it does not account for the
    /// #included files or stripped comments (the optimizer does not support reading comments).
    pub fn build(self) {
        use std::io::Write;

        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir);

        // A map of include names to file IDs
        let mut include_map = HashMap::new();

        // Write each shader file
        for shader_source in self.sources.iter() {
            let name = shader_source.path.file_name().unwrap().to_str().unwrap();
            let source = read_file(&shader_source.path);

            // #include
            let (source, version) = self.process(&mut include_map, &name, source.trim_right(), Vec::new(), 0, None);

            // optimize
            let source = if self.version_supported(version) {
                match self.do_optimize(source, shader_source.kind) {
                    Ok(source) => source,
                    Err(err) => {
                        panic!("optimization error for shader source \"{}\": {}", name, err);
                    }
                }
            } else {
                source
            };

            // write to file
            let mut file = File::create(out_path.join(name)).unwrap();
            file.write_all(source.as_bytes()).unwrap();
        }

        // Write the map source file
        let mut file = File::create(out_path.join("glossy_map.rs")).unwrap();
        write!(&mut file, "pub fn id_to_file_name(id: u32) -> &'static str {{ match id {{\n").unwrap();
        let mut include_map: Vec<(String, usize)> = include_map.into_iter().collect();
        include_map.sort_by_key(|&(_, ref v)| *v);
        for (name, id) in include_map.into_iter() {
            write!(&mut file, "{} => Some({:?}),\n", id, name).unwrap();
        }
        write!(&mut file, "_ => None }} }}").unwrap();

        // Set the glossy_macros_only cfg
        println!("cargo:rustc-cfg=glossy_macros_only");
    }

    /// Helper function which returns a newly-generated shader source with inlined #includes
    fn process(&self,
               include_map: &mut HashMap<String, usize>,
               name: &str,
               source: &str,
               include_stack: Vec<&str>,
               file_id: usize,
               version: Option<&str>) -> (String, String) {
        use std::fmt::Write;

        lazy_static! {
            static ref LINE_COMMENT_RE: Regex = Regex::new(r"/\*.*?\*/|//.*").unwrap();
            static ref INCLUDE_ANGLE_RE: Regex = Regex::new("^\\s*#\\s*include\\s+<([:print:]+)>\\s*$").unwrap();
            static ref INCLUDE_QUOTE_RE: Regex = Regex::new("^\\s*#\\s*include\\s+\"([:print:]+)\"\\s*$").unwrap();
            static ref VERSION_RE: Regex = Regex::new(r"^\s*#\s*version\s+(\d+)").unwrap();
            static ref EMPTY_RE: Regex = Regex::new(r"^\s*$").unwrap();
        }

        // The processed source
        let mut output = String::new();

        // true if we're in a block comment
        let mut block_comment = false;
        // true if we've yet to parse either the first line or the #version directive
        let mut first_line = true;
        // the version parsed in the source
        let mut parsed_version = String::new();
        // the first line num (for the line directive)
        let mut first_line_num = 1;

        for (line_num, line) in source.lines().enumerate() {
            let line_after_block_comment = if block_comment {
                // If we're in a block comment, look for the first terminator we can find
                match line.find("*/") {
                    Some(idx) => {
                        block_comment = false;
                        &line[idx + 2..]
                    }
                    None => {
                        // If there's no terminator, continue along.
                        if self.preserve_line_info {
                            write!(&mut output, "{}\n", line).unwrap();
                        }
                        continue;
                    }
                }
            } else {
                &line
            };

            // Remove all single-line comments from this line for testing
            let line_after_line_comments = LINE_COMMENT_RE.replace(line_after_block_comment, "");

            // Deal with the start of multi-line block comment starters
            let line_no_comments = match line_after_line_comments.find("/*") {
                Some(idx) => {
                    block_comment = true;
                    &line_after_line_comments[..idx]
                }
                None => {
                    &line_after_line_comments
                }
            };

            if EMPTY_RE.is_match(line_no_comments) {
                // Skip empty lines if we're not preserving line info
                if !self.preserve_line_info {
                    continue;
                }
            } else if first_line {
                first_line = false;
                // If this is the first non-empty line, check if it's a #version directive.
                if let Some(cap) = VERSION_RE.captures(line_no_comments) {
                    parsed_version = cap.at(1).unwrap().to_string();
                    // If the version is different than the one passed in, panic
                    if let Some(version) = version {
                        if parsed_version != version {
                            panic!("included file \"{}\" specifies version {}, but parent specifies \"{}\"",
                                       name, parsed_version, version);
                        }
                    }
                    // The first line is 2
                    first_line_num = 2;
                    // Write a compensatory blank line if necessary.
                    if self.preserve_line_info && line_num > 0 {
                        if block_comment {
                            write!(&mut output, "/*\n").unwrap();
                        } else {
                            write!(&mut output, "\n").unwrap();
                        }
                    }
                    // Since we know this is a version directive, skip the rest of the loop.
                    continue;
                } else {
                    // The first line is 1, so don't touch it

                    // Default to our parent's version; if None, then the default.
                    parsed_version = match version {
                        Some(v) => v.to_string(),
                        None => match self.lang {
                            Language::OpenGl => "110".to_string(),
                            Language::OpenGlEs20 | Language::OpenGlEs30 => "100".to_string(),
                        }
                    };
                }
            }

            if self.preserve_line_info {
                // If the line is not empty, and we've not yet inserted a line directive, we can now
                // do so.
                if !first_line && EMPTY_RE.is_match(line_no_comments) {

                }
            } else {
                // If we don't care about line info, skip this line if it's empty
                // OR if we're an included file and it's a #version directive
                if EMPTY_RE.is_match(line_no_comments) {
                    continue;
                }
                if !include_stack.is_empty() && VERSION_RE.is_match(line_no_comments) {
                    continue;
                }
            }

            // Now actually figure out if this is an #include directive.
            let include_name;
            if let Some(cap) = INCLUDE_QUOTE_RE.captures(line_no_comments) {
                include_name = cap.at(1).unwrap();
            } else if let Some(cap) = INCLUDE_ANGLE_RE.captures(line_no_comments) {
                include_name = cap.at(1).unwrap();
            } else {
                // Write the line and move on
                if self.preserve_line_info {
                    write!(&mut output, "{}\n", line).unwrap();
                } else {
                    write!(&mut output, "{}\n", line_no_comments).unwrap();
                }
                continue;
            };

            // This is an #include! So #include it.
            let include_source = self.includes.get(include_name).unwrap_or_else(|| {
                panic!("shader file \"{}\" includes non-existent file \"{}\"", name, include_name);
            });
            // But first, see if we've already got it in our #include stack, and complain about
            // recursive includes.
            if include_stack.contains(&include_name) {
                panic!("recursive inclusion of \"{}\" in shader file \"{}\"", include_name, name);
            }
            // Process the included file
            let mut sub_include_stack = include_stack.clone();
            sub_include_stack.push(include_name);
            let new_id = include_map.len() + 1;
            let include_file_id = *include_map.entry(include_name.to_string()).or_insert(new_id);
            let (include_source, _) = self.process(include_map,
                                                    include_name,
                                                    include_source,
                                                    sub_include_stack,
                                                    include_file_id,
                                                    Some(&parsed_version));
            if self.preserve_line_info {
                write!(&mut output, "{}\n#line {} {}\n", include_source, line_num + 2, file_id).unwrap();
                if block_comment {
                    // If there was a start of a block comment on this line, we need to insert that here
                    write!(&mut output, "/*\n").unwrap();
                }
            } else {
                if !include_source.is_empty() {
                    write!(&mut output, "{}\n", include_source).unwrap();
                }
            }
        }
        // Return
        if include_stack.is_empty() {
            (format!("#version {}\n#line {} {}\n{}", parsed_version, first_line_num, file_id, output.trim_right()), parsed_version)
        } else {
            (format!("#line {} {}\n{}", first_line_num, file_id, output.trim_right()), parsed_version)
        }
    }

    /// Helper function which determines if the optimizer supports the version of the
    /// shader-to-be-optimized.
    fn version_supported(&self, version: String) -> bool {
        match self.lang {
            Language::OpenGl => (self.allow_untested || version == "110" || version == "120"),
            Language::OpenGlEs20 | Language::OpenGlEs30 => (version == "100" || version == "300"),
        }
    }

    /// Helper function which optimizes the shader source with glsl-optimizer
    #[cfg(feature = "optimizer")]
    fn do_optimize(&self, source: String, kind: SourceKind) -> Result<String, String> {
        if let Some(ref optimizer) = self.optimizer {
            optimizer.optimize(source, kind)
        } else {
            Ok(source)
        }
    }

    #[cfg(not(feature = "optimizer"))]
    fn do_optimize(&self, source: String) -> String {
        // dummy
        source
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SourceKind {
    Vertex,
    Fragment,
    Unknown,
}

struct Source {
    path: PathBuf,
    kind: SourceKind,
}

impl Source {
    pub fn new(path: PathBuf, kind: SourceKind) -> Source {
        Source {
            path: path,
            kind: kind,
        }
    }
}

/// Helper function which reads the contents of a file as a string.
fn read_file<P: AsRef<Path>>(p: P) -> String {
    let mut file = File::open(p).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    s
}
