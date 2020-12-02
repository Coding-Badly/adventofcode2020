use std::io::BufRead;

use regex::{Captures, Regex};

#[derive(Debug)]
enum Error {
    InvalidInput(String),
    InvalidNumber(std::num::ParseIntError)
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Error::InvalidNumber(error)
    }
}

enum ParserCommand {
    Parse(String),
    Stop,
}

struct ScanResult {
    password: String,
    valid: bool,
}

impl std::fmt::Display for ScanResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.valid {
            write!(f, "The password \"{}\" IS VALID.", self.password)
        }
        else {
            write!(f, "The password \"{}\" is NOT valid.", self.password)
        }
    }
}

impl Into<bool> for ScanResult {
    fn into(self) -> bool {
        self.valid
    }
}

struct ScanRequest {
    n1: usize,
    n2: usize,
    letter: char,
    password: String,
}

impl ScanRequest {
    fn from_captures(captures: &Captures) -> Result<Self, Error> {
        let n1 = captures.name("n1").unwrap().as_str().parse()?;
        let n2 = captures.name("n2").unwrap().as_str().parse()?;
        let letter = captures.name("l").unwrap().as_str().chars().next().unwrap();
        let password = captures.name("pw").unwrap().as_str().to_string();
        Ok(Self {
            n1,
            n2,
            letter,
            password,
        })
    }
    fn scan_by_count(self) -> ScanResult {
        let mut count = 0;
        for rover in self.password.chars() {
            if rover == self.letter {
                count += 1;
            }
        }
        let valid = count >= self.n1 && count <= self.n2;
        ScanResult {
            password: self.password,
            valid,
        }
    }
    fn scan_by_position(self) -> ScanResult {
        let mut count = 0;
        for (index, rover) in self.password.chars().enumerate() {
            let starts_at_one = index + 1;
            if (starts_at_one == self.n1 || starts_at_one == self.n2) && (rover == self.letter) {
                count += 1;
            }
        }
        let valid = count == 1;
        ScanResult {
            password: self.password,
            valid,
        }
    }
}

enum ScannerCommand {
    Scan(ScanRequest),
    Stop,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Create a channel (funnel) for errors that occur in the context of a thread
    let (error_tx, error_rx): (std::sync::mpsc::Sender<Error>, std::sync::mpsc::Receiver<Error>)
        = std::sync::mpsc::channel();

    // Create a channel for the main thread to send commands to the Parser
    let (parser_tx, parser_rx): (std::sync::mpsc::Sender<ParserCommand>, std::sync::mpsc::Receiver<ParserCommand>)
        = std::sync::mpsc::channel();

    // Create a channel for the Parser to send commands to the Scanner
    let (scanner_tx, scanner_rx): (std::sync::mpsc::Sender<ScannerCommand>, std::sync::mpsc::Receiver<ScannerCommand>)
        = std::sync::mpsc::channel();

    // Create a channel for the Scanner to output results
    let (result_tx, result_rx): (std::sync::mpsc::Sender<ScanResult>, std::sync::mpsc::Receiver<ScanResult>)
        = std::sync::mpsc::channel();

    // The Parser thread
    let parser_errors = error_tx.clone();
    let scan_requests = scanner_tx.clone();
    let parser = std::thread::spawn(move || {
        let re = Regex::new(r"\s*(?P<n1>[1-9][0-9]*)\s*-\s*(?P<n2>[1-9][0-9]*)\s*(?P<l>.)\s*:\s*(?P<pw>.*)").unwrap();
        loop {
            let command = parser_rx.recv().expect("Parser unable to receive the next command");
            match command {
                ParserCommand::Parse(line) => {
                    if let Some(captures) = re.captures(&line) {
                        match ScanRequest::from_captures(&captures) {
                            Ok(scan_request) =>
                                scan_requests.send(ScannerCommand::Scan(scan_request)).expect("Unable to send scan request"),
                            Err(error) =>
                                parser_errors.send(error).expect("Unable to send an error"),
                        }
                    }
                    else {
                        parser_errors.send(Error::InvalidInput(line)).expect("Unable to send an error");
                    }
                }
                ParserCommand::Stop => break,
            }
        }
    });

    //let scanner_errors = error_tx.clone();
    let scanner = std::thread::spawn(move || {
        loop {
            let command = scanner_rx.recv().expect("Scanner unable to receive the next command");
            match command {
                ScannerCommand::Scan(scan_request) =>
                    result_tx.send(scan_request.scan_by_position()).expect("Scanner unable to send results"),
                ScannerCommand::Stop => break,
            }
        }
    });

    // Drop the left over channel reference
    drop(error_tx);

    // Feed STDIN to the Parser
    for line in std::io::stdin().lock().lines() {
        let line = line?.trim().to_string();
        if line == "" {
            break;
        }
        parser_tx.send(ParserCommand::Parse(line)).expect("Unable to send the next command to Parser");
    }

    // Tell the two threads to stop
    parser_tx.send(ParserCommand::Stop).expect("Unable to send the next command to Parser");
    scanner_tx.send(ScannerCommand::Stop).expect("Unable to send the next command to Scanner");

    // Wait for them to stop
    parser.join().expect("The Parser panicked");
    scanner.join().expect("The Scanner panicked");

    // Output any errors
    while let Some(error) = error_rx.recv().ok() {
        println!("Error: {:?}", error);
    }

    // Output the results
    let mut valid = 0;
    let mut total = 0;
    while let Some(result) = result_rx.recv().ok() {
        println!("{}", result);
        if result.into() {
            valid += 1;
        }
        total += 1;
    }
    println!("Of the {} passwords {} are valid.", total, valid);

    Ok(())
}
