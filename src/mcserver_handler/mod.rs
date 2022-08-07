use std::{thread::{self, JoinHandle}, time::Duration, process::{ChildStdout, ChildStderr, ChildStdin, Child}, io::{Read, self, Write}, sync::{mpsc::Sender, Mutex, Arc}, fmt};

use serenity::{model::prelude::Message, prelude::Context};

mod mcsrv_thread;

pub struct McsrvHandler {
    process: Child,
    stderr: Option<ChildStderr>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,

    pub stderr_complete_output: Mutex<Vec<u8>>,
    pub stdout_complete_output: Mutex<Vec<u8>>,

    thread_stdout: Arc<Mutex<Option<mcsrv_thread::stdout::Stdout>>>,
    thread_dcmsg: Arc<Mutex<mcsrv_thread::dcmsg::Dcmsg>>,
}
impl McsrvHandler {
    pub fn new(mut process: Child, ctx: Context, msg: Message) -> Self {
        let stdin = Arc::new(Mutex::new(process.stdin.take()));
        let thread_stdout = armut(match process.stdout.take() {
            Some(stdout) => Some(mcsrv_thread::stdout::Stdout::new(stdout, stdin.clone())),
            None => None,
        });
        let thread_dcmsg = mcsrv_thread::dcmsg::Dcmsg::new(ctx, msg, thread_stdout.clone(), stdin.clone());
        Self {
            stderr: process.stderr.take(),
            stdin,
            stderr_complete_output: Mutex::new(Vec::new()),
            stdout_complete_output: Mutex::new(Vec::new()),
            thread_stdout: thread_stdout,
            thread_dcmsg: armut(thread_dcmsg),
            process: process,
        }
    }
    pub fn kill(&mut self) -> io::Result<()> {
        self.process.kill()?;

        let mut thread_dcmsg = self.thread_dcmsg.lock().unwrap();
        thread_dcmsg.quit_req();
        let mut thread_stdout = self.thread_stdout.lock().unwrap();
        if let Some(thread_stdout) = thread_stdout.as_mut() { thread_stdout.quit_req(); };
        Ok(())
    }
    pub fn write_to_stdin(&mut self, data: &[u8]) -> io::Result<bool> {
        if let Some(stdin) = &mut *self.stdin.lock().unwrap() {
            stdin.write_all(data)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Same as Arc::new(Mutex::new(v))
fn armut<T>(v: T) -> Arc<Mutex<T>> { Arc::new(Mutex::new(v)) }