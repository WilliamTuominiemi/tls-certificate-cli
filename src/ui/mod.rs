use ratatui::layout::{Constraint, Margin, Offset, Rect};
use ratatui::style::Style;
use ratatui::widgets::{
    Block, BorderType, Clear, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Tabs, Wrap,
};
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
    show_help: bool,
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
            show_help: false,
        }
    }

    pub fn run(mut self) -> io::Result<()> {
        ratatui::run(|terminal| self.main_loop(terminal))
    }

    fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let mut area = frame.area();
        area.width = if self.cert_amount < 6 {
            90
        } else {
            (self.cert_amount as u16 + 1) * 15
        };

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
                KeyCode::Char('h') => self.show_help = !self.show_help,
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
        let text = self.certificates[self.selected_tab].clone();
        let block = Paragraph::new(text).wrap(Wrap { trim: false }).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1))
                .border_style(Style::new().light_magenta())
                .title_bottom("'h' - toggle help"),
        );
        frame.render_widget(
            block.scroll((self.scrollbar_state.get_position() as u16, 0)),
            area,
        );

        if self.show_help {
            self.render_help(frame, area);
        }
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

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let popup_block = Block::bordered()
            .title("Help")
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(1))
            .border_style(Style::new().light_cyan());
        let centered_area = area.centered(Constraint::Length(60), Constraint::Length(5));

        frame.render_widget(Clear, centered_area);
        let paragraph = Paragraph::new("Arrow keys left/right - switch certificate \nArrow keys up/down - scroll certificate \nq - quit
        ")
            .block(popup_block);
        frame.render_widget(paragraph, centered_area);
    }
}
