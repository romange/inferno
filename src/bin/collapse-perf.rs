use std::io;
use std::path::PathBuf;

use env_logger::Env;
use inferno::collapse::perf::{Folder, Options};
use inferno::collapse::{Collapse, DEFAULT_NTHREADS};
use lazy_static::lazy_static;
use structopt::StructOpt;

lazy_static! {
    static ref NTHREADS: String = format!("{}", *DEFAULT_NTHREADS);
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "inferno-collapse-perf",
    about,
    after_help = "\
[1] perf script must emit both PID and TIDs for these to work; eg, Linux < 4.1:
        perf script -f comm,pid,tid,cpu,time,event,ip,sym,dso,trace
    for Linux >= 4.1:
        perf script -F comm,pid,tid,cpu,time,event,ip,sym,dso,trace
    If you save this output add --header on Linux >= 3.14 to include perf info."
)]
struct Opt {
    // ************* //
    // *** FLAGS *** //
    // ************* //
    /// Include raw addresses where symbols can't be found
    #[structopt(long = "addrs")]
    addrs: bool,

    /// All annotations (--kernel --jit)
    #[structopt(long = "all")]
    all: bool,

    /// Annotate jit functions with a `_[j]`
    #[structopt(long = "jit")]
    jit: bool,

    /// Annotate kernel functions with a `_[k]`
    #[structopt(long = "kernel")]
    kernel: bool,

    /// Include PID with process names
    #[structopt(long = "pid")]
    pid: bool,

    /// Include TID and PID with process names
    #[structopt(long = "tid")]
    tid: bool,

    /// Silence all log output
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// Verbose logging mode (-v, -vv, -vvv)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    // *************** //
    // *** OPTIONS *** //
    // *************** //
    /// Event filter [default: first encountered event]
    #[structopt(long = "event-filter", value_name = "STRING")]
    event_filter: Option<String>,

    /// Number of threads to use
    #[structopt(
        short = "n",
        long = "nthreads",
        default_value = &NTHREADS,
        value_name = "UINT"
    )]
    nthreads: usize,

    // ************ //
    // *** ARGS *** //
    // ************ //
    #[structopt(value_name = "PATH")]
    /// Perf script output file, or STDIN if not specified
    infile: Option<PathBuf>,

    #[structopt(long = "skip-after", value_name = "STRING")]
    /// If set, will omit all the parent stack frames of the frame with matched function name.
    ///
    /// Has no effect on the stack trace if no function is matched.
    skip_after: Option<String>,
}

impl Opt {
    fn into_parts(self) -> (Option<PathBuf>, Options) {
        let mut options = Options::default();
        options.include_pid = self.pid;
        options.include_tid = self.tid;
        options.include_addrs = self.addrs;
        options.annotate_jit = self.jit || self.all;
        options.annotate_kernel = self.kernel || self.all;
        options.event_filter = self.event_filter;
        options.nthreads = self.nthreads;
        options.skip_after = self.skip_after;
        (self.infile, options)
    }
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    // Initialize logger
    if !opt.quiet {
        env_logger::Builder::from_env(Env::default().default_filter_or(match opt.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }))
        .format_timestamp(None)
        .init();
    }

    let (infile, options) = opt.into_parts();
    Folder::from(options).collapse_file_to_stdout(infile.as_ref())
}
