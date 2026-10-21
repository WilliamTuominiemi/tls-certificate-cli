use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
};
use std::io;

pub struct Ui {
    should_quit: bool,
}

impl Ui {
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub fn run(mut self) -> io::Result<()> {
        ratatui::run(|mut terminal| self.main_loop(terminal))
    }

    fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget("TLS certificate", frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let event::Event::Key(key) = event::read()? {
            if key.is_press() && key.code == KeyCode::Char('q') {
                self.should_quit = true;
            }
        }
        Ok(())
    }
}
