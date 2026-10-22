use ratatui::layout::{Offset, Rect};
use ratatui::widgets::{Block, Paragraph, Tabs};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
};
use std::io;

pub struct Ui {
    should_quit: bool,
    selected_tab: usize,
    cert_amount: usize,
    certificates: Vec<String>,
}

impl Ui {
    pub fn new(certificates: Vec<String>) -> Self {
        Self {
            should_quit: false,
            selected_tab: 0,
            cert_amount: certificates.len(),
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
        self.render_content(frame, frame.area());
        self.render_tabs(frame, frame.area() + Offset::new(1, 0));
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Right => self.selected_tab = (self.selected_tab + 1) % self.cert_amount,
                KeyCode::Left => {
                    self.selected_tab =
                        (self.selected_tab + self.cert_amount - 1) % self.cert_amount
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let titles = (0..self.cert_amount).map(|i| format!("Certificate {}", i));

        let tabs = Tabs::new(titles).select(self.selected_tab);
        frame.render_widget(tabs, area);
    }

    fn render_content(&self, frame: &mut Frame, area: Rect) {
        let text = self.certificates[self.selected_tab as usize].clone();
        let block = Paragraph::new(text).block(Block::bordered());
        frame.render_widget(block, area);
    }
}
