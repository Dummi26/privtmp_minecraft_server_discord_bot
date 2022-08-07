use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex}, time::Duration, borrow::Cow, process::ChildStdin, io::Write};

use futures::executor;
use serenity::{prelude::Context, model::prelude::Message};

use super::stdout::OtherData;

pub struct Dcmsg {
    thread: JoinHandle<()>,
    quit: Arc<Mutex<bool>>,
    stopped: Arc<Mutex<bool>>,

    ctx: Context,
    msg: Message,
    stdout: Arc<Mutex<Option<super::stdout::Stdout>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
}
impl Dcmsg {
    pub fn new(ctx: Context, msg: Message, stdout: Arc<Mutex<Option<super::stdout::Stdout>>>, stdin: Arc<Mutex<Option<ChildStdin>>>) -> Self {
        let quit = Arc::new(Mutex::new(false));
        let stopped = Arc::new(Mutex::new(false));

        let thread = {
            let t_quit = quit.clone();
            let t_stopped = stopped.clone();
            let t_stdin = stdin.clone();
            let t_stdout = stdout.clone();
            let mut t_msg = msg.clone();
            let t_http = ctx.http.clone();
            thread::spawn(move || {
                let sleep_duration_before = Duration::from_secs(1);
                let sleep_duration_after = Duration::from_secs(2);
                match executor::block_on(t_msg.reply(&t_http, "Starting server...")) {
                    Ok(mut associated_message) => {
                        loop {
                            let quit = *t_quit.lock().unwrap(); // other thread requests that i stop
                            if quit { break; } // stop
                            else { // do not stop
                                if let Some(stdin) = t_stdin.lock().unwrap().as_mut() {
                    stdin.write_all("\
scoreboard players get players_online dcbot_temp
scoreboard players get entities dcbot_temp
".as_bytes());
                                };
                                thread::sleep(sleep_duration_before);
                                match executor::block_on(associated_message.edit(&t_http, |m| m.content(
                                    format!(
                                        "{}Server log:\n```\n{}\n```",
                                        match Self::get_count1(&t_stdout) { Some(v) => format!(
                                            "Players online: {}\nEntities: {}\n",
                                            match v.players_online { Some(v) => v.to_string(), None => "?".to_string(), },
                                            match v.entities { Some(v) => v.to_string(), None => "?".to_string(), },
                                        ), None => String::new(), },
                                        get_stdout_last_n_chars(&t_stdout)
                                    )
                                ))) {
                                    Ok(_) => {},
                                    Err(err) => { println!("Could not edit message. Error: {}", err); },
                                };
                                thread::sleep(sleep_duration_after);
                            };
                        };
                    },
                    Err(err) => { println!("Could not send associated message. Not starting server. Error: {}", err); },
                };
                println!("Stopped thread.");
                *t_stopped.lock().unwrap() = true; // tell the other threads that i have stopped
            })
        };

        fn get_stdout_last_n_chars(stdout: &Arc<Mutex<Option<super::stdout::Stdout>>>) -> String {
            let stdout = &*stdout.lock().unwrap();
            if let Some(stdout) = stdout {
                if stdout.is_running() {
                    let str = match String::from_utf8_lossy(stdout.full_output.lock().unwrap().as_slice()) { Cow::Owned(v) => v, Cow::Borrowed(v) => v.to_string(), };
                    let mut len = 0;
                    for char in str.chars().rev() {
                        len += char.len_utf8();
                        if len > 500 {
                            break;
                        }
                    };
                    if len == 0 {
                        ""
                    } else {
                        &str[str.len()-len..]
                    }.to_string()
                } else {
                    String::from("Server shut down.")
                }
            } else {
                String::from("Server not running.")
            }
        }

        Self {
            thread,
            quit,
            stopped,
            ctx,
            msg,
            stdout,
            stdin,
        }
    }

    // - - - - -
    
    pub fn get_count1(stdout: &Arc<Mutex<Option<super::stdout::Stdout>>>) -> Option<OtherData> {
        if let Some(stdout) = stdout.lock().unwrap().as_ref() {
            Some(stdout.other_data.lock().unwrap().clone())
        } else {
            None
        }
    }

    // - - - - -

    /// Ask the thread to quit.
    pub fn quit_req(&mut self) {
        *self.quit.lock().unwrap() = true;
    }
    /// Ask the thread to quit and block until it does so.
    pub fn quit_block(&mut self) {
        self.quit_req();
        self.quit_wait();
    }
    /// Wait for the thread to stop.
    pub fn quit_wait(&self) {
        while self.is_running() {}
    }
    pub fn is_running(&self) -> bool { !*self.stopped.lock().unwrap() }
}