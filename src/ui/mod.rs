use ratatui::layout::{Margin, Offset, Rect};
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
};
use std::{cmp, io};

pub struct Ui {
    should_quit: bool,
    selected_tab: usize,
    cert_amount: usize,
    certificates: Vec<String>,
    scroll: usize,
    max_scroll: usize,
    scrollbar_state: ScrollbarState,
}

impl Ui {
    pub fn new(certificates: Vec<String>) -> Self {
        let mut longest_certificate = 0;
        for certificate in &certificates {
            let lines: Vec<&str> = certificate.split("\n").collect();
            longest_certificate = cmp::max(longest_certificate, lines.len());
        }

        Self {
            should_quit: false,
            selected_tab: 0,
            cert_amount: certificates.len(),
            certificates,
            scroll: 0,
            max_scroll: 0,
            scrollbar_state: ScrollbarState::default(),
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

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let total_lines = self.certificates[self.selected_tab].lines().count();
        let visible = area.height.saturating_sub(2) as usize;
        self.max_scroll = total_lines.saturating_sub(visible);
        self.scroll = self.scroll.min(self.max_scroll);

        self.scrollbar_state = self
            .scrollbar_state
            .content_length(self.max_scroll + 1)
            .viewport_content_length(visible)
            .position(self.scroll);

        self.render_content(frame, area);
        self.render_tabs(frame, area + Offset::new(1, 0));
        self.render_vertical_scrollbar(frame, area);
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
                KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                KeyCode::Down => self.scroll = (self.scroll + 1).min(self.max_scroll),
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
        frame.render_widget(
            block.scroll((self.scrollbar_state.get_position() as u16, 0)),
            area,
        );
    }

    fn render_vertical_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.scrollbar_state,
        );
    }
}
