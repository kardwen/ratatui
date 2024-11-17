//! # [Ratatui] Table example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

use color_eyre::Result;
use itertools::Itertools;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{self, Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    DefaultTerminal, Frame,
};
use style::palette::tailwind;

const INFO_TEXT: [&str; 3] = [
    "(Esc) quit | (k) move up | (j) move down | (h) move left | (l) move right",
    "(J) increase viewport height | (K) decrease viewport height",
    "(H) decrease content length | (L) increase content length",
];

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}
struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_column_style_fg: Color,
    selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new() -> Self {
        let palette = &tailwind::EMERALD;
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: palette.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_row_style_fg: palette.c400,
            selected_column_style_fg: palette.c400,
            selected_cell_style_fg: palette.c600,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: palette.c400,
        }
    }
}

struct Data {
    name: String,
    email: String,
}

impl Data {
    fn name(&self) -> &str {
        &self.name
    }

    fn email(&self) -> &str {
        &self.email
    }
}

struct App {
    state: TableState,
    content_length: usize,
    items: Vec<Data>,
    scroll_state: ScrollbarState,
    colors: TableColors,
    viewport_height: u16,
}

impl App {
    fn new() -> Self {
        let content_length = 8;
        let data_vec = generate_fake_names(content_length);
        Self {
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new(data_vec.len()),
            colors: TableColors::new(),
            content_length,
            items: data_vec,
            viewport_height: 8,
        }
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1).min(self.items.len() - 1),
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }

    pub fn next_column(&mut self) {
        self.state.select_next_column();
    }

    pub fn previous_column(&mut self) {
        self.state.select_previous_column();
    }

    pub fn increase_viewport_height(&mut self) {
        self.viewport_height += 1;
    }

    pub fn decrease_viewport_height(&mut self) {
        self.viewport_height = self.viewport_height.saturating_sub(1);
    }

    pub fn increase_content_length(&mut self) {
        self.content_length += 1;
        self.items = generate_fake_names(self.content_length);
        self.scroll_state = ScrollbarState::new(self.items.len());
    }

    pub fn decrease_content_length(&mut self) {
        self.content_length = self.content_length.saturating_sub(1);
        self.items = generate_fake_names(self.content_length);
        self.scroll_state = ScrollbarState::new(self.items.len());
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                        KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
                        KeyCode::Char('J') => self.increase_viewport_height(),
                        KeyCode::Char('K') => self.decrease_viewport_height(),
                        KeyCode::Char('l') | KeyCode::Right => self.next_column(),
                        KeyCode::Char('h') | KeyCode::Left => self.previous_column(),
                        KeyCode::Char('H') => self.decrease_content_length(),
                        KeyCode::Char('L') => self.increase_content_length(),
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let areas = &Layout::vertical([
            Constraint::Length(self.viewport_height + 1),
            Constraint::Length(1),
            Constraint::Length(5),
        ])
        .flex(Flex::Start)
        .split(frame.area());
        let [left, right] =
            Layout::horizontal([Constraint::Min(1), Constraint::Length(1)]).areas(areas[0]);

        self.render_table(frame, left);
        self.render_scrollbar(frame, right);
        self.render_status(frame, areas[1]);
        self.render_footer(frame, areas[2]);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
        let selected_col_style = Style::default().fg(self.colors.selected_column_style_fg);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_cell_style_fg);

        let header = ["Name", "Email"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };
            Row::new(vec![Cell::from(data.name()), Cell::from(data.email())])
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(1)
        });
        let t = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Min(1),
                Constraint::Min(1),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        .column_highlight_style(selected_col_style)
        .cell_highlight_style(selected_cell_style)
        .highlight_symbol(Text::from(vec!["".into(), "█".into()]))
        .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        if area.height < 1 {
            return;
        }
        // account for table header
        let area = Rect {
            x: area.x,
            y: area.y + 1,
            height: area.height.saturating_sub(1),
            width: area.width,
        };
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area,
            &mut self.scroll_state,
        );
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let status = Paragraph::new(Line::from(format!(
            "viewport height: {}, content length: {}",
            self.viewport_height, self.content_length
        )));
        frame.render_widget(status, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from_iter(INFO_TEXT))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(info_footer, area);
    }
}

fn generate_fake_names(length: usize) -> Vec<Data> {
    use fakeit::{contact, name};

    (0..length)
        .map(|_| {
            let name = name::full();
            let email = contact::email();

            Data { name, email }
        })
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect()
}
