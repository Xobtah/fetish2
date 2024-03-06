use std::{error::Error, io};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListDirection, ListItem, ListState, Paragraph},
};

enum InputMode {
    Select,
    Query,
}

struct App {
    conn: rusqlite::Connection,
    input: String,
    cursor_position: usize,
    input_mode: InputMode,
    users: Vec<(bool, bool, i64, String)>,
    selected_user: Option<usize>,
}

impl App {
    fn new() -> Result<Self, rusqlite::Error> {
        Ok(Self {
            conn: rusqlite::Connection::open("db.sqlite")?,
            input: String::new(),
            cursor_position: 0,
            input_mode: InputMode::Select,
            users: Default::default(),
            selected_user: None,
        })
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    // fn reset_cursor(&mut self) {
    //     self.cursor_position = 0;
    // }

    fn submit_search_query(&mut self) {
        self.users = self
            .conn
            .prepare(
                r"
SELECT s.user_id AS is_scammer, u.user_id, u.first_name, u.last_name
FROM USERS u
LEFT JOIN SCAMMERS s
ON u.user_id = s.user_id
WHERE first_name LIKE :first_name
",
            )
            .unwrap()
            .query_map(
                rusqlite::named_params! {
                    ":first_name": &self.input,
                },
                |row| {
                    Ok((
                        false,
                        row.get::<_, i64>("is_scammer").is_ok(),
                        row.get::<_, i64>("user_id")?,
                        format!(
                            "{} {}",
                            row.get::<_, String>("first_name")?,
                            row.get::<_, String>("last_name")?
                        ),
                    ))
                },
            )
            .unwrap()
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<(bool, bool, i64, String)>>();
        // self.input.clear();
        // self.reset_cursor();
    }

    fn submit_update_query(&mut self) {
        for (is_in_selection, is_scammer, user_id, _) in &self.users {
            if *is_in_selection {
                self.conn
                    .execute(
                        if *is_scammer {
                            "DELETE FROM SCAMMERS WHERE user_id = :user_id"
                        } else {
                            "INSERT INTO SCAMMERS (user_id) VALUES (:user_id)"
                        },
                        rusqlite::named_params! {
                            ":user_id": user_id,
                        },
                    )
                    .unwrap();
            }
        }
        self.submit_search_query();
    }

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            self.draw(terminal)?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Select => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Query;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Down if self.users.len() > 0 => {
                            self.selected_user = Some(match self.selected_user {
                                Some(i) => {
                                    if i >= self.users.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            })
                        }
                        KeyCode::Up if self.users.len() > 0 => {
                            self.selected_user = Some(match self.selected_user {
                                Some(i) => {
                                    if i == 0 {
                                        self.users.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => self.users.len() - 1,
                            })
                        }
                        KeyCode::Char(' ') if self.selected_user.is_some() => {
                            if let Some(i) = self.selected_user {
                                self.users[i].0 = !self.users[i].0;
                            }
                        }
                        KeyCode::Enter => self.submit_update_query(),
                        _ => {}
                    },
                    InputMode::Query if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => {
                            self.submit_search_query();
                            self.input_mode = InputMode::Select;
                        }
                        KeyCode::Char(to_insert) => {
                            self.enter_char(to_insert);
                        }
                        KeyCode::Backspace => {
                            self.delete_char();
                        }
                        KeyCode::Left => {
                            self.move_cursor_left();
                        }
                        KeyCode::Right => {
                            self.move_cursor_right();
                        }
                        KeyCode::Esc => {
                            self.input_mode = InputMode::Select;
                        }
                        _ => {}
                    },
                    InputMode::Query => {}
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ]);
        let [help_area, input_area, messages_area, footer_area] = vertical.areas(area);

        let (msg, style) = match self.input_mode {
            InputMode::Select => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Query => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                ],
                Style::default(),
            ),
        };
        Paragraph::new(Text::from(Line::from(msg)).patch_style(style)).render(help_area, buf);

        Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Select => Style::default(),
                InputMode::Query => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"))
            .render(input_area, buf);

        match self.input_mode {
            InputMode::Select =>
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                {}

            InputMode::Query => {
                // Make the cursor visible and ask ratatui to put it at the specified coordinates after
                // rendering
                // #[allow(clippy::cast_possible_truncation)]
                // f.set_cursor(
                //     // Draw the cursor at the current position in the input field.
                //     // This position is can be controlled via the left and right arrow key
                //     input_area.x + self.cursor_position as u16 + 1,
                //     // Move one line down, from the border to the input line
                //     input_area.y + 1,
                // );
            }
        }

        let users: Vec<ListItem> = self
            .users
            .iter()
            .map(|(is_in_selection, is_scammer, _, u)| {
                let list_item = ListItem::new(Line::from(Span::raw(format!(
                    "{} {u}",
                    if *is_scammer { " ✓ " } else { " ☐ " }
                ))));
                if *is_in_selection {
                    list_item.bg(Color::Red)
                } else {
                    list_item
                }
            })
            .collect();
        let users = List::new::<Vec<ListItem>>(users)
            .block(Block::default().title("Users").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(style::palette::tailwind::BLUE.c300),
            )
            .highlight_symbol(">")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);
        StatefulWidget::render(
            &users,
            messages_area,
            buf,
            &mut ListState::default().with_selected(self.selected_user),
        );

        match self.input_mode {
            InputMode::Select if self.users.len() > 0 => {
                let (msg, style) = (
                    vec![
                        "Use ".into(),
                        "↓↑".bold(),
                        " to move, ".into(),
                        "Space".bold(),
                        " to change status, ".into(),
                        "Enter".bold(),
                        " to commit changes.".into(),
                    ],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                )
                    .into();
                Paragraph::new(Text::from(Line::from(msg)).patch_style(style))
                    .render(footer_area, buf)
            }
            InputMode::Select => {}
            InputMode::Query => {}
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let res = App::new()?.run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err:#?}");
    }

    Ok(())
}
