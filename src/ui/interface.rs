use crate::ui::backend::{get_groups, load_record, ActionState, App, StatefulList, TabsState};
use crate::ui::handler::{Event, Events};
use std::{error::Error, io, vec::Vec};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, Row, Table, Tabs},
    Terminal,
};

pub fn render() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = MouseTerminal::from(io::stdout().into_raw_mode()?);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::new();

    // declare headers
    let headers = vec![
        "filesystem",
        "block_usage",
        "block_soft",
        "block_hard",
        "block_grace",
        "inode_usage",
        "inode_soft",
        "inode_hard",
        "inode_grace",
    ];

    // App
    let groups = get_groups()?;
    let mut app = App {
        items: StatefulList::new(groups),
        action: ActionState::new(vec!["Hit", "Kick", "Delete"]),
        tabs: TabsState::new(vec!["Search", "Overview"]),
    };

    // main render loop
    loop {
        terminal.draw(|f| {
            // create layout
            let (main, actions) = if app.action.is_visible {
                (86, 7)
            } else {
                (93, 0)
            };
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(7),
                        Constraint::Percentage(main),
                        Constraint::Percentage(actions),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // create tabs
            let titles = app
                .tabs
                .titles
                .iter()
                .map(|t| Spans::from(t.to_owned()))
                .collect();
            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Modes"))
                .select(app.tabs.index)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            f.render_widget(tabs, chunks[0]);

            match app.tabs.index {
                0_usize => {
                    let sub_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(chunks[1]);

                    let record = load_record(
                        &headers,
                        &app.items.items[app.items.state.selected().unwrap_or(0_usize)],
                    )
                    .expect("Quota could not be loaded");
                    let rows = record.1.iter().map(|i| Row::Data(i.iter()));
                    let table = Table::new(headers.iter(), rows)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Group Data")
                                .border_type(BorderType::Rounded),
                        )
                        .header_style(Style::default().fg(Color::Green))
                        .widths(&[
                            Constraint::Length(10),
                            Constraint::Length(5),
                            Constraint::Length(5),
                            Constraint::Length(5),
                            Constraint::Length(5),
                            Constraint::Length(5),
                            Constraint::Length(5),
                            Constraint::Length(5),
                            Constraint::Length(5),
                        ]);

                    f.render_widget(table, sub_chunks[0]);

                    // render groups
                    let items: Vec<ListItem> = app
                        .items
                        .items
                        .iter()
                        .map(|i| ListItem::new(Span::from(i.to_owned())))
                        .collect();
                    let items = List::new(items)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Groups")
                                .border_type(BorderType::Rounded),
                        )
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(items, sub_chunks[1], &mut app.items.state);
                }
                1_usize => {
                    // placeholder block
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("Quota Control")
                        .border_type(BorderType::Rounded);
                    f.render_widget(block, chunks[1]);
                }
                _ => panic!("Entered not implemented mode"),
            }

            if app.action.is_visible {
                // create action tabs
                let titles = app
                    .action
                    .state
                    .titles
                    .iter()
                    .map(|t| Spans::from(t.to_owned()))
                    .collect();
                let actions = Tabs::new(titles)
                    .block(Block::default().borders(Borders::ALL).title("Actions"))
                    .select(app.action.state.index)
                    .highlight_style(Style::default().add_modifier(Modifier::BOLD));
                f.render_widget(actions, chunks[2]);
            }
        })?;

        if let Event::Input(input) = events.next()? {
            match input {
                Key::Char('q') => {
                    break;
                }
                Key::Char('\t') => app.tabs.next(),
                Key::Right => app.action.state.next(),
                Key::Left => app.action.state.next(),
                Key::Down => app.items.next(),
                Key::Up => app.items.previous(),
                Key::PageUp => app.items.select(0_usize),
                Key::PageDown => app.items.select(app.items.items.len() - 1_usize),
                _ => {}
            }
        }
    }

    Ok(())
}
