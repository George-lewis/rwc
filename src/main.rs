use std::{
    fmt::{format, Arguments},
    fs::File,
    io::{BufRead, BufReader, Write},
    ops::Add,
    process::ExitCode,
};

use clap::Parser;
use options::Options;

mod options;

// custom `write!()` macro that writes to a `FixedString`
// it has a special case for writing string literals
// to avoid the overhead of formatting
//
// the downside is that it doesn't support inline format arguments
macro_rules! write_fixed {
    ($dst:expr, $argzero:tt, $($arg:tt)+) => {
        $dst.write_fmt(format_args!($argzero, $($arg)*))
    };
    ($dst:expr, $lit:literal) => {
        $dst.write_lit($lit)
    };
}

/// A fixed-size string that can be used to build a string without heap allocation
pub struct FixedString<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> FixedString<N> {
    pub fn new() -> Self {
        Self {
            data: [0; N],
            len: 0,
        }
    }

    /// this function creates a `&str` from the data
    /// you must ensure that the data is valid utf-8
    /// note: zero cost, as this is a mem::transmute
    pub unsafe fn as_str_unchecked(&self) -> &str {
        // note: zero cost, this is a mem::transmute
        std::str::from_utf8_unchecked(&self.data[..self.len])
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// this will panic if there's not enough space
    pub fn push_str_unchecked(&mut self, s: &str) {
        self.data[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());

        self.len += s.len();
    }

    pub fn write_lit(&mut self, lit: &str) {
        self.push_str_unchecked(lit);
    }

    // compatibility with the `write!()` macro
    pub fn write_fmt(&mut self, args: Arguments) {
        // actually format the string
        // we could optimize for the no-args case
        // but we have a custom `write!()` macro
        // that calls into `write_fmt_lit()` instead for that case
        let fmt = format(args);

        self.push_str_unchecked(&fmt);
    }
}

#[derive(Debug)]
struct Statistics {
    bytes: u32,
    chars: u32,
    lines: u32,
    words: u32,
    max_line_length: u16,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            bytes: 0,
            chars: 0,
            lines: 0,
            words: 0,
            max_line_length: 0,
        }
    }

    pub fn print(&self, opts: &Options, filename: &str) {
        let mut s = FixedString::<64>::new();

        if opts.bytes {
            write_fixed!(&mut s, "{:8} ", self.bytes);
        }

        if opts.chars {
            write_fixed!(&mut s, "{:8} ", self.chars);
        }

        if opts.lines {
            write_fixed!(&mut s, "{:8} ", self.lines);
        }

        if opts.words {
            write_fixed!(&mut s, "{:8} ", self.words);
        }

        if opts.max_line_length {
            write_fixed!(&mut s, "{:8} ", self.max_line_length);
        }

        // note: zero cost, this is a mem::transmute
        // safety: we're sure that the string is valid utf-8
        // because we're building it from valid utf-8 strings
        let s = unsafe { s.as_str_unchecked() };

        // `filename` is an unknown length
        // so we can't add it to the `FixedString`
        println!("{s} {filename}");
    }
}

impl Add for Statistics {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            bytes: self.bytes + other.bytes,
            chars: self.chars + other.chars,
            lines: self.lines + other.lines,
            words: self.words + other.words,
            max_line_length: self.max_line_length.max(other.max_line_length),
        }
    }
}

fn read_file<R: BufRead>(reader: &mut R, buffer: &mut String) -> anyhow::Result<Statistics> {
    let mut statistics = Statistics::new();

    while reader.read_line(buffer)? > 0 {
        statistics.lines += 1;
        statistics.bytes += buffer.len() as u32;
        statistics.chars += buffer.chars().count() as u32;
        statistics.words += buffer.split_whitespace().count() as u32;
        statistics.max_line_length = statistics.max_line_length.max(buffer.len() as u16);

        buffer.clear();
    }

    Ok(statistics)
}

fn print_header(options: &Options) -> std::io::Result<()> {
    let mut s = FixedString::<54>::new();

    if options.bytes {
        write_fixed!(&mut s, "   bytes")
    }

    if options.chars {
        write_fixed!(&mut s, "    chars")
    }

    if options.lines {
        write_fixed!(&mut s, "    lines")
    }

    if options.words {
        write_fixed!(&mut s, "    words")
    }

    if options.max_line_length {
        write_fixed!(&mut s, "      max")
    }

    if options.filename {
        write_fixed!(&mut s, " filename")
    }

    write_fixed!(s, "\n");

    // note: `s` is valid utf-8
    // because we're building it from valid utf-8 strings
    std::io::stdout().write_all(s.as_bytes())
}

fn main() -> anyhow::Result<ExitCode> {
    let options = Options::try_parse()?;

    let mut total = Statistics::new();
    let mut buffer = String::new();

    if options.files.is_empty() {
        let reader = std::io::stdin().lock();

        let statistics = read_file(&mut BufReader::new(reader), &mut buffer)?;

        if !options.no_header {
            print_header(&options)?;
        }

        statistics.print(&options, "-");
    } else {
        if !options.no_header {
            print_header(&options)?;
        }

        for filename in &options.files {
            let statistics = if filename == "-" {
                let mut reader = std::io::stdin().lock();

                read_file(&mut reader, &mut buffer)
            } else {
                let file = File::open(&filename)?;
                let mut reader = BufReader::new(file);

                read_file(&mut reader, &mut buffer)
            }?;

            statistics.print(&options, &filename);

            total = total + statistics;
        }
    }

    if options.files.len() > 1 {
        total.print(&options, "total");
    }

    Ok(ExitCode::SUCCESS)
}
