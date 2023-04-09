use aktBrowser::cli;
use structopt::StructOpt;
fn main() {
    let opts = cli::opts::Command::from_args();

    match opts {
        cli::opts::Command::Open(opt) => cli::open::run(opt),
    };
}
