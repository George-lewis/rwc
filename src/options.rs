use clap::ArgAction;

#[derive(Debug, clap::Parser)]
pub struct Options {
    pub files: Vec<String>,

    #[clap(short = 'b', long, default_value_t = true, action = ArgAction::SetFalse)]
    pub bytes: bool,

    #[clap(short = 'c', long, default_value_t = true, action = ArgAction::SetFalse)]
    pub chars: bool,

    #[clap(short = 'l', long, default_value_t = true, action = ArgAction::SetFalse)]
    pub lines: bool,

    #[clap(short = 'w', long, default_value_t = true, action = ArgAction::SetFalse)]
    pub words: bool,

    #[clap(short = 'L', long, default_value_t = true, action = ArgAction::SetFalse)]
    pub max_line_length: bool,

    #[clap(long, default_value_t = true, action = ArgAction::SetFalse)]
    pub filename: bool,

    #[clap(long, default_value_t = false)]
    pub no_header: bool,
}
