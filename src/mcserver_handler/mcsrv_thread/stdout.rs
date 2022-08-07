use std::{thread::{self, JoinHandle}, process::{ChildStdout, ChildStdin}, io::Read, sync::{Mutex, Arc}, time::Duration};

use crate::mcserver_handler::armut;

pub struct Stdout {
    thread: Option<JoinHandle<()>>,
    pub full_output: Arc<Mutex<Vec<u8>>>,
    quit: Arc<Mutex<bool>>,
    stopped: Arc<Mutex<bool>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    pub other_data: Arc<Mutex<OtherData>>,

}
impl Stdout {
    pub fn new(mut stream: ChildStdout, stdin: Arc<Mutex<Option<ChildStdin>>>) -> Self {
        let quit = Arc::new(Mutex::new(false));
        let stopped = Arc::new(Mutex::new(false));
        let data = Arc::new(Mutex::new(Vec::<u8>::new()));
        let other_data = Arc::new(Mutex::new(OtherData::new()));

        let thread = {
            let t_quit = quit.clone();
            let t_stopped = stopped.clone();
            let t_data = data.clone();
            let t_other_data = other_data.clone();
            thread::spawn(move || {
                let mut buf = [0u8];
                let mut line_start = 0;
                loop {
                    let quit = *t_quit.lock().unwrap(); // other thread requests that i stop
                    if quit { break; }; // quit
                    match stream.read(&mut buf) { // read from stream
                        Ok(v) => match v { // should always receive 1 byte, as the buffer is only 1 byte long.
                            1 => { // push byte onto data
                                let data = &mut *t_data.lock().unwrap();
                                data.push(buf[0]);
                                if buf[0] == 10 {
                                    // line including \n
                                    let line = String::from_utf8_lossy(&data[line_start..]).to_string();
                                    line_start = data.len();
                                    Self::parse_line(line, &t_other_data);
                                    println!("Definitively finished parsing line.");
                                }
                            },
                            _ => { println!("Read EOF, stopping thread."); break; }, // eof
                        },
                        Err(err) => { println!("Failed to read stdout, stopping thread: {}", err); break; }, // error reading from stream
                    };
                };
                println!("Stopped thread.");
                *t_stopped.lock().unwrap() = true; // tell the other threads that i have stopped
            })
        };

        Self {
            thread: Some(thread),
            full_output: data,
            quit,
            stopped,
            stdin,
            other_data,
        }
    }

    // - - - - -

    fn parse_line(line: String, other_data: &Arc<Mutex<OtherData>>) {
        println!("Parsing line");
        /*
        Possible:
            [06:46:32] [Server thread/INFO]: players_online has 1 [dcbot_temp]
            [06:46:01] [Server thread/INFO]: [Dummi26: Added 1 to [dcbot_temp] for players_online (now 1)]
         */
        let mut chars = Vec::from_iter(line.chars());
        let mut char_iter = chars.iter();
        // move forward until you hit ": "
        let mut pchar = false;
        loop { if let Some(char) = char_iter.next()  { if pchar && *char == ' ' { break; } pchar = *char == ':'; } else { return; } };
        let line = String::from_iter(char_iter);
        let line_chars = Vec::from_iter(line.chars());
        if line_chars.len() > 0 && line_chars[0] != '[' { // otherwise it is most likely a player running some sort of command
            let identifiers = ["players_online", "entities"];
            for index in 0..identifiers.len() {
                let str = identifiers[index].to_string() + " has ";
                if line.starts_with(str.as_str()) {
                    let rest = &line[str.len()..];
                    let mut rest_iter = rest.chars();
                    let mut value = String::new();
                    loop {
                        match rest_iter.next() {
                            Some(c) => { if c == ' ' { break; }; value.push(c); },
                            None => { return; /* there is always a space behind the value, if not, then this message isn't something we are interested in. */},
                        };
                    };
                    match str::parse::<u32>(&value) {
                        Ok(value) => {
                            match index {
                                0 => other_data.lock().unwrap().players_online = Some(value),
                                1 => other_data.lock().unwrap().entities = Some(value),
                                _ => {},
                            }
                        },
                        Err(_) => {},
                    };
                };
            };
        };
    }

    // - - - - -

    /// Ask the thread to quit.
    pub fn quit_req(&mut self) {
        *self.quit.lock().unwrap() = true;
    }
    /// Ask the thread to quit and block until it does so.
    pub fn quit_blocks(&mut self) {
        self.quit_req();
        self.quit_wait();
    }
    /// Wait for the thread to stop.
    pub fn quit_wait(&mut self) {
        while self.is_running() { thread::sleep(Duration::from_millis(250)); }
    }
    pub fn is_running(&self) -> bool { !*self.stopped.lock().unwrap() }
}

pub struct OtherData {
    pub players_online: Option<u32>,
    pub entities: Option<u32>,
}
impl OtherData {
    pub fn new() -> Self {
        Self {
            players_online: None,
            entities: None,
        }
    }
    pub fn clone(&self) -> Self {
        Self {
            players_online: self.players_online.clone(),
            entities: self.entities.clone(),
        }
    }
}