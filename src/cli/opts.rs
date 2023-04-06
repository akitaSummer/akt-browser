use super::open;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Command {
    Open(open::Opts),
}
