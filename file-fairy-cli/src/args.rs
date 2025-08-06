use clap::{Arg, ArgAction};

/// Builder for common CLI arguments used across multiple subcommands
pub struct ArgBuilder;

impl ArgBuilder {
    // File system operation arguments
    
    /// Creates a recursive scanning argument
    pub fn recursive() -> Arg {
        Arg::new("recursive")
            .long("recursive")
            .short('r')
            .help("Scan subdirectories recursively")
            .action(ArgAction::SetTrue)
    }

    /// Creates a max-depth argument for recursive scanning
    pub fn max_depth() -> Arg {
        Arg::new("max-depth")
            .long("max-depth")
            .help("Maximum depth for recursive scanning")
            .value_name("DEPTH")
            .value_parser(clap::value_parser!(usize))
    }

    /// Creates a follow-symlinks argument
    pub fn follow_symlinks() -> Arg {
        Arg::new("follow-symlinks")
            .long("follow-symlinks")
            .help("Follow symbolic links")
            .action(ArgAction::SetTrue)
    }

    /// Creates a max-size argument for file size filtering
    pub fn max_size() -> Arg {
        Arg::new("max-size")
            .long("max-size")
            .help("Maximum file size to process (in MB)")
            .value_name("SIZE_MB")
            .value_parser(clap::value_parser!(u64))
    }

    // LLM configuration arguments

    /// Creates a model path argument with default
    pub fn model_path(default: &'static str) -> Arg {
        Arg::new("model")
            .long("model")
            .short('m')
            .help("Path to the LLM model file")
            .value_name("MODEL_PATH")
            .default_value(default)
    }

    /// Creates a max-tokens argument with default
    pub fn max_tokens(default: &'static str) -> Arg {
        Arg::new("max-tokens")
            .long("max-tokens")
            .help("Maximum number of tokens to generate")
            .value_name("TOKENS")
            .default_value(default)
            .value_parser(clap::value_parser!(u32))
    }

    /// Creates a threads argument with default
    pub fn threads(default: &'static str) -> Arg {
        Arg::new("threads")
            .long("threads")
            .short('t')
            .help("Number of threads to use")
            .value_name("THREADS")
            .default_value(default)
            .value_parser(clap::value_parser!(u32))
    }

    // Interface and behavior arguments

    /// Creates a verbose argument
    pub fn verbose() -> Arg {
        Arg::new("verbose")
            .long("verbose")
            .short('v')
            .help("Enable verbose output")
            .action(ArgAction::SetTrue)
    }

    /// Creates an interactive mode argument
    pub fn interactive() -> Arg {
        Arg::new("interactive")
            .long("interactive")
            .short('i')
            .help("Prompt for each file with proposed changes")
            .action(ArgAction::SetTrue)
    }

    /// Creates an apply/execute argument
    pub fn apply() -> Arg {
        Arg::new("apply")
            .long("apply")
            .help("Actually perform the file operations (default is dry-run preview)")
            .action(ArgAction::SetTrue)
    }

    // Path arguments

    /// Creates a directory path argument (positional)
    pub fn directory_path(name: &'static str, help: &'static str, default: &'static str) -> Arg {
        Arg::new(name)
            .help(help)
            .value_name("PATH")
            .default_value(default)
            .index(1)
    }

    /// Creates a file path argument (positional, required)
    pub fn file_path(name: &'static str, help: &'static str) -> Arg {
        Arg::new(name)
            .help(help)
            .required(true)
            .value_name("FILE")
            .index(1)
    }

    /// Creates a target directory argument (optional)
    pub fn target_dir() -> Arg {
        Arg::new("target-dir")
            .long("target-dir")
            .help("Target directory for organized files (default: organize in place)")
            .value_name("DIR")
    }
}
