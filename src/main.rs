// Copyright (c) Jethro G. Beekman
//
// This file is part of rust-reduce.
//
// rust-reduce is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rust-reduce is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rust-reduce.  If not, see <https://www.gnu.org/licenses/>.

use std::{ffi::OsString, io::Write, process::{Command, Stdio}};

use clap::clap_app;
use quote::ToTokens;
use syn_inline_mod::{Error as InlineError, InlinerBuilder};
use tempfile::NamedTempFile;

mod output;
mod transforms;

fn main() {
    let matches = clap_app!(("rust-reduce") =>
        (version: clap::crate_version!())
        (@arg CMD: * "Command to run.")
        (@arg ARGS: * ... "Arguments to the command to run.
        
The last argument must be the path of the existing file of interest. CMD will be invoked with the last argument replaced with the path to a temporary file.

You can use `--` to separate ARGS from any arguments passed to `rust-reduce`.")
        (@arg FILE: -o --output +takes_value "Reduced output file (default is to replace input file).")
        (@arg ONCE: short("1") --("no-progress") "Only save the fully reduced output, not the intermediates.")
        (after_help: "\
`rust-reduce` will try to make the source file smaller by interpreting it as valid Rust code and intelligently removing parts of the code. After each removal, the given command will be run but passing a path to a file containing the reduced code. The command should return 0 if run on the original input, and also if the reduced code is interesting, non-0 otherwise.

The original file will be overwritten with the smallest interesting reduced version, if found. This happens while `rust-reduce` is running. The original file will be backed up with the `.orig` suffix. If `rustfmt` is found, it will be used to clean up the output.

A common way to use `rust-reduce` is to write a short shell script that runs `rustc` and greps the compiler output for a particular error message. NB. you will want to look for a specific error message because while `rust-reduce` will generate syntactically correct code, it's not guaranteed to compile.

The original file may refer to modules in different files, these will be inlined and reduced along with the main file.")
    ).get_matches();

    let mut cmd = vec![matches.value_of_os("CMD").expect("validated").to_owned()];
    let mut iter = matches.values_of_os("ARGS").expect("validated").map(ToOwned::to_owned);
    let file = iter.next_back().expect("validated");
    cmd.extend(iter);

    if !run_with_path(&cmd, &file) {
        eprintln!("rust-reduce: run with initial input did not indicate success");
        std::process::exit(1);
    }

    let mut inlined_file = match InlinerBuilder::new()
        .error_not_found(true)
        .parse_and_inline_modules(file.as_ref()) {
        Ok(f) => f,
        Err(InlineError::NotFound(missing)) => {
            eprintln!("rust-reduce: file not found");
            for (modname, loc) in missing {
                eprintln!("    mod {} @ {}:{}", modname, loc.path.display(), loc.line);
            }
            std::process::exit(1);
        },
        Err(_) => unimplemented!()
    };
    let mut output = (if matches.is_present("ONCE") {
        output::WaitGuard::new::<output::LastWriter, _>
    } else {
        output::WaitGuard::new::<output::AsyncWriter, _>
    })(
        matches.value_of_os("FILE").map(ToOwned::to_owned).unwrap_or(file),
        !matches.is_present("FILE")
    );

    let mut try_compile = |reduced_file: &_| {
        let result = run_with_path(&cmd, &write_file(reduced_file).path());

        if result {
            output.output_formatted(reduced_file)
        }

        result
    };

    eprintln!("Pruning items");
    transforms::prune_items::prune_items(&mut inlined_file, &mut try_compile);
    eprintln!("Removing #[derive] attributes");
    transforms::remove_derive_attrs::remove_derive_attrs(&mut inlined_file, &mut try_compile);
    eprintln!("Removing #[doc] attributes");
    transforms::remove_doc_attrs::remove_doc_attrs(&mut inlined_file, &mut try_compile);
    eprintln!("Clearing block bodies");
    transforms::clear_blocks::clear_blocks(&mut inlined_file, &mut try_compile);
}

fn run_with_path<P: AsRef<std::path::Path>>(cmd: &[OsString], path: &P) -> bool {
    let (cmd, args) = cmd.split_first().expect("validated");
    match Command::new(cmd)
        .args(args)
        .arg(path.as_ref())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(ref stat) if stat.success() => true,
        _ => false
    }
}

fn write_file(contents: &syn::File) -> NamedTempFile {
    let mut file = tempfile::Builder::new().prefix("test").tempfile().unwrap();
    write!(file, "{}", contents.into_token_stream()).unwrap();
    file.flush().unwrap();
    file
}
