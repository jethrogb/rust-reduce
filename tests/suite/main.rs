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

use std::{env, fs, path::PathBuf, process::Command};

fn run_test(dir: &str) {
    let mut path = tests_dir();
    path.push(dir);

    let out = Command::new(find_rust_reduce())
        .args(&["-1", "-o", "-"])
        .args(&[path.join("test.sh"), path.join("input.rs")])
        .output()
        .unwrap();

    if !out.status.success() {
        eprintln!("`rust-reduce` failed with {}", out.status);
        eprintln!("{}", String::from_utf8(out.stderr).unwrap());
        panic!("Test failed");
    }
    let expected = fs::read_to_string(path.join("output.rs")).unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), expected);
}

macro_rules! tests {
    ($($s:ident),* $(,)*) => {
        $(
            #[test]
            fn $s() {
                run_test(stringify!($s));
            }
        )*
    }
}

tests!(futures_core);

fn find_rust_reduce() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("rust-reduce");
    path.set_extension(env::consts::EXE_EXTENSION);
    path
}

fn tests_dir() -> PathBuf {
    let mut path = PathBuf::from(file!());
    path.pop();
    path
}
