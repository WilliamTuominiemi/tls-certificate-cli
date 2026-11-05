use ratatui::layout::{Constraint, Margin, Offset, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Clear, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Tabs, Wrap,
};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
};
use std::{format, io};

use crate::certificate::Certificate;

pub struct Ui {
    should_quit: bool,
    selected_tab: usize,
    cert_amount: usize,
    certificates: Vec<Certificate>,
    scroll: usize,
    max_scroll: usize,
    scrollbar_state: ScrollbarState,
    show_help: bool,
}

impl Ui {
    pub fn new(certificates: Vec<Certificate>) -> Self {
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
            100
        } else {
            (self.cert_amount as u16 + 1) * 18
        };

        let text = self.build_text();
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        let inner_width = area.width.saturating_sub(4);
        let total_lines = paragraph.line_count(inner_width);

        let visible = area.height.saturating_sub(3) as usize;
        self.max_scroll = total_lines.saturating_sub(visible);
        self.scroll = self.scroll.min(self.max_scroll);

        self.scrollbar_state = self
            .scrollbar_state
            .content_length(self.max_scroll + 1)
            .viewport_content_length(visible)
            .position(self.scroll);

        self.render_content(frame, area, paragraph);
        self.render_tabs(frame, area + Offset::new(1, 0));
        if total_lines >= visible {
            self.render_vertical_scrollbar(frame, area);
        }
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
        let titles = (0..self.cert_amount).map(|i| {
            if i == 0 {
                format!(" {}. Leaf ", i)
            } else if i == self.cert_amount - 1 {
                format!(" {}. Root ", i)
            } else {
                format!(" {}. Intermediate ", i)
            }
        });

        let tabs = Tabs::new(titles).select(self.selected_tab);
        frame.render_widget(tabs, area);
    }

    fn render_content(&mut self, frame: &mut Frame, area: Rect, paragraph: Paragraph) {
        let block = paragraph.wrap(Wrap { trim: false }).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1))
                .border_style(Style::new().light_magenta())
                .title_bottom("'h' - toggle help"),
        );
        frame.render_widget(block.scroll((self.scroll as u16, 0)), area);

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
        let paragraph = Paragraph::new(
            "← / → - switch certificate \n↑ / ↓ - scroll certificate \nq - quit
        ",
        )
        .block(popup_block);
        frame.render_widget(paragraph, centered_area);
    }

    fn heading(&self, title: &str) -> Line<'static> {
        Line::from(Span::styled(
            title.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
    }

    fn field(&self, label: &str, value: impl Into<String>) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                format!("{label}: "),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(value.into()),
        ])
    }

    fn build_text(&self) -> Vec<Line<'static>> {
        let c = &self.certificates[self.selected_tab];
        let mut text = Vec::new();

        text.push(self.heading("Identity"));
        text.push(self.field("Subject", &c.subject));
        if let Some(san) = &c.subject_alternative_name {
            text.push(self.field("SAN", san.to_string()));
        }
        text.push(self.field("Issuer", &c.issuer));
        text.push(Line::default());

        text.push(self.heading("Validity"));
        text.push(self.field("Period", &c.validity_period));
        text.push(Line::default());

        text.push(self.heading("Key & Signature"));
        text.push(self.field("Public key", &c.public_key_and_algorithm));
        text.push(self.field("Signature algorithm", &c.signature_algorithm));
        text.push(Line::default());

        text.push(self.heading("Identifiers"));
        text.push(self.field("Serial", &c.serial_number));
        text.push(self.field("Subject Key Identifier", &c.key_identifiers.ski));
        text.push(self.field("Authority Key Identifier", &c.key_identifiers.aki));
        text.push(Line::default());

        text.push(self.heading("Usage"));
        text.push(self.field("CA", &c.basic_constraints));
        text.push(self.field("Key usage", &c.key_usage));
        text.push(self.field("Extended key usage", &c.extended_key_usage));
        text.push(Line::default());

        text.push(self.heading("Fingerprints"));
        text.push(self.field(
            "SHA-256 (certificate)",
            &c.fingerprint.certificate_fingerprint,
        ));
        text.push(self.field("SPKI pin (SHA-256, base64)", &c.fingerprint.spki_pin));

        text
    }
}
