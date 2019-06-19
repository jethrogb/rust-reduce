// Copyright (c) Jethro G. Beekman
//
// This file is part of rust-reduce.
//
// rust-reduce is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Foobar is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rust-reduce.  If not, see <https://www.gnu.org/licenses/>.

use std::{ffi::OsStr, fs, io::Write, path::PathBuf, process::{Command, Stdio}, sync::mpsc, thread::{self, JoinHandle}};

use quote::ToTokens;

/// A type that will wait during `Drop` for all output operations to complete.
pub struct WaitGuard {
    path: Option<PathBuf>,
    constructor: fn(PathBuf, bool) -> Box<OutputType>,
    inner: Option<Box<OutputType>>,
    need_backup: bool,
}

impl WaitGuard {
    pub fn new<T: OutputType, P: Into<PathBuf>>(path: P, need_backup: bool) -> Self {
        WaitGuard {
            path: Some(path.into()),
            constructor: T::new,
            inner: None,
            need_backup,
        }
    }

    pub fn output_formatted(&mut self, reduced_file: &syn::File) {
        let path = self.path.take();
        let WaitGuard { constructor, need_backup, .. } = *self;
        self.inner.get_or_insert_with(|| constructor(path.unwrap(), need_backup))
            .output(reduced_file.into_token_stream().to_string())
    }
}

pub trait OutputType {
    fn new(path: PathBuf, need_backup: bool) -> Box<OutputType> where Self: Sized;
    fn output(&mut self, reduced_file: String);
}

pub struct AsyncWriter {
    chan: mpsc::Sender<Message>,
    thread: Option<JoinHandle<()>>,
}

impl OutputType for AsyncWriter {
    fn new(file: PathBuf, need_backup: bool) -> Box<OutputType> {
        let (send, recv) = mpsc::channel();

        let thread = thread::spawn(move || {
            TargetFileWorker {
                backed_up: !need_backup,
                file,
                chan: recv,
            }.run()
        });

        Box::new(AsyncWriter {
            chan: send,
            thread: Some(thread),
        })
    }

    fn output(&mut self, reduced_file: String) {
        self.chan.send(Message::Work(reduced_file)).unwrap()
    }
}

impl Drop for AsyncWriter {
    fn drop(&mut self) {
        self.chan.send(Message::Quit).unwrap();
        self.thread.take().unwrap().join().unwrap()
    }
}

pub struct LastWriter {
    worker: TargetFileWorker,
    last: Option<String>,
}

impl OutputType for LastWriter {
    fn new(file: PathBuf, need_backup: bool) -> Box<OutputType> {
        Box::new(LastWriter {
            worker: TargetFileWorker {
                backed_up: !need_backup,
                file,
                chan: mpsc::channel().1,
            },
            last: None,
        })
    }

    fn output(&mut self, reduced_file: String) {
        self.last = Some(reduced_file);
    }
}

impl Drop for LastWriter {
    fn drop(&mut self) {
        // unwrap ok: `new` is always immediately followed by `output`
        self.worker.emit_formatted_file(self.last.take().unwrap());
    }
}

enum Message {
    Work(String),
    Quit
}

struct TargetFileWorker {
    backed_up: bool,
    file: PathBuf,
    chan: mpsc::Receiver<Message>,
}

impl TargetFileWorker {
    fn backup(&mut self) {
        if !self.backed_up {
            let mut orig = self.file.clone().into_os_string();
            orig.push(".orig");
            if let Err(e) = fs::copy(&self.file, orig) {
                eprintln!("Failed to backup input file: {}", e);
                std::process::exit(1);
            }
            self.backed_up = true;
        }
    }

    fn emit_formatted_file(&mut self, reduced_file: String) {
        self.backup();

        eprintln!("{} bytes...", reduced_file.len());

        match Command::new("rustfmt")
            .stdout(if self.file == OsStr::new("-") {
                    Stdio::inherit()
                } else {
                    Stdio::from(fs::File::create(&self.file).unwrap())
                })
            .stdin(Stdio::piped())
            .spawn() {
            Ok(mut child) => {
                child.stdin.take().unwrap().write_all(reduced_file.as_bytes()).unwrap();
                child.wait().unwrap();
            }
            Err(_) => {
                if self.file == OsStr::new("-") {
                    Box::new(std::io::stdout()) as Box<Write>
                } else {
                    Box::new(fs::File::create(&self.file).unwrap())
                }.write_all(reduced_file.as_bytes()).unwrap();
            }
        }
    }

    fn run(&mut self) {
        let mut next = None;
        while let Message::Work(mut tokens) = next.unwrap_or_else(|| self.chan.recv().unwrap()) {
            next = None;

            // skip ahead if there's more work in the queue
            while let Ok(msg) = self.chan.try_recv() {
                match msg {
                    Message::Work(new_toks) => tokens = new_toks,
                    other => next = Some(other),
                }
            }

            self.emit_formatted_file(tokens)
        }
    }
}
