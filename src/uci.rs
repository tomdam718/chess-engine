use std::{iter::Peekable, str::SplitWhitespace};

pub trait Uci {
    /// The main loop of the UCI protocol
    fn uci_loop(&mut self) {
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let mut args = input.split_whitespace().peekable();

            match args.next() {
                Some("uci") => {
                    self.send("id name Tsunami");
                    self.send("id author github.com/thomasdamcevski");
                    self.send("uciok");
                }
                Some("isready") => {
                    self.send("readyok");
                }
                Some("ucinewgame") => {}
                Some("position") => {
                    self.set_position(&mut args);
                }
                Some("go") => {
                    self.go(&mut args);
                }
                Some("eval") => {
                    self.display_eval();
                }
                Some("d") => {
                    self.display();
                }
                Some("quit") => {
                    break;
                }
                _ => {
                    self.send("Unrecognized command");
                }
            }
        }
    }

    /// The main entry point for the search
    fn go(&mut self, args: &mut Peekable<SplitWhitespace>);

    /// Display the static NNUE evaluation of the current position
    fn display_eval(&mut self);

    /// Sets the position of the board from a list of moves, or from a FEN string
    fn set_position(&mut self, args: &mut Peekable<SplitWhitespace>);

    /// Display an ASCII representation of the board
    /// As well as some other information about the position
    /// such as the FEN
    fn display(&self);

    /// Send a message to the GUI
    fn send(&self, msg: &str) {
        println!("{}", msg);
    }

    /// Response when receiving an unknown command
    fn unknown(&self) {
        self.send("Unrecognized command");
    }
}
