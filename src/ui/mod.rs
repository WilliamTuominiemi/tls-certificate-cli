use ratatui::widgets::{Block, Paragraph};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
};
use std::io;

pub struct Ui {
    should_quit: bool,
    certificates: String,
}

impl Ui {
    pub fn new(certificates: String) -> Self {
        Self {
            should_quit: false,
            certificates,
        }
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
        let block = Block::bordered().title("TLS certificate");
        let inner = block.inner(frame.area());

        frame.render_widget(block, frame.area());
        frame.render_widget(Paragraph::new(self.certificates.clone()), inner);
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
