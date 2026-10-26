use ratatui::layout::{Constraint, Margin, Offset, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
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

pub struct Ui<'a> {
    should_quit: bool,
    selected_tab: usize,
    cert_amount: usize,
    certificates: Vec<Certificate<'a>>,
    scroll: usize,
    max_scroll: usize,
    scrollbar_state: ScrollbarState,
    show_help: bool,
}

impl<'a> Ui<'a> {
    pub fn new(certificates: Vec<Certificate<'a>>) -> Self {
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

    fn build_text(&self) -> Vec<Line<'static>> {
        let certificate = &self.certificates[self.selected_tab];
        let mut text = vec![
            Line::from(format!("Subject: {}", certificate.subject)),
            Line::from(format!("Issuer: {}", certificate.issuer)),
            Line::from(format!(
                "Validity period: {} - {} ({} days left)",
                certificate.validity_period.not_before.format("%d/%m/%Y"),
                certificate.validity_period.not_after.format("%d/%m/%Y"),
                certificate.validity_period.days_left
            )),
        ];

        if let Some(basic_constraints) = &certificate.basic_constraints {
            let line = if basic_constraints.ca {
                let path_len = match basic_constraints.path_len_constraint {
                    Some(len) => len.to_string(),
                    None => "unlimited".to_string(),
                };
                format!("CA: true, Path length: {}", path_len)
            } else {
                "CA: false".to_string()
            };
            text.push(Line::from(line));
        }

        text.push(Line::from(certificate.key_usage.to_string()));

        if let Some(san) = &certificate.san {
            let names = san
                .general_names
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            text.push(Line::from(format!(
                "SAN ({}): {}",
                san.general_names.len(),
                names
            )));
        }

        text
    }
}
