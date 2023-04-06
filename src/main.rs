use aktBrowser::cli;
use structopt::StructOpt;
fn main() {
    let opts = cli::opts::Command::from_args();

    let exit_code = match opts {
        cli::opts::Command::Open(opt) => cli::open::run(opt),
    };

    // std::process::exit(exit_code)
}
