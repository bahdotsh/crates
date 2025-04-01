use crate::app::{App, LoadingState, Tab};
use chrono::DateTime;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create the main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    draw_title(f, chunks[0]);
    draw_tabs(f, app, chunks[1]);

    // Draw content based on current tab
    match app.current_tab {
        Tab::Recent => draw_crates_list(f, app, chunks[2], "Recent Crates"),
        Tab::Search => draw_search_tab(f, app, chunks[2]),
        Tab::Trending => draw_repos_list(f, app, chunks[2], "Trending Repositories"),
        Tab::Help => draw_help(f, app, chunks[2]),
    }

    draw_status_bar(f, app, chunks[3]);
}

fn draw_title<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let title = Paragraph::new(Text::styled(
        "Crates Explorer",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
    .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(title, area);
}

fn draw_tabs<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let titles = ["Recent", "Search", "Trending", "Help"]
        .iter()
        .map(|t| Spans::from(vec![Span::styled(*t, Style::default().fg(Color::White))]))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(match app.current_tab {
            Tab::Recent => 0,
            Tab::Search => 1,
            Tab::Trending => 2,
            Tab::Help => 3,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn draw_crates_list<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect, title: &str) {
    let items: Vec<ListItem> = app
        .crates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = format!("{} v{}", c.name, c.max_version);
            let desc = c.description.clone().unwrap_or_default();
            let downloads = format!("Downloads: {}", c.downloads);

            // Parse and format date
            let updated = if let Ok(dt) = DateTime::parse_from_rfc3339(&c.updated_at) {
                format!("Updated: {}", dt.format("%Y-%m-%d"))
            } else {
                "".to_string()
            };

            let content = vec![
                Spans::from(vec![Span::styled(
                    name,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]),
                Spans::from(vec![Span::raw(desc)]),
                Spans::from(vec![
                    Span::styled(downloads, Style::default().fg(Color::Blue)),
                    Span::raw(" | "),
                    Span::styled(updated, Style::default().fg(Color::Yellow)),
                ]),
            ];

            ListItem::new(content).style(Style::default().fg(if i == app.selected_index {
                Color::Yellow
            } else {
                Color::White
            }))
        })
        .collect();

    // If we're in loading state, show a loading message
    if matches!(app.loading_state, LoadingState::Loading) {
        let loading = ListItem::new(vec![Spans::from(vec![Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )])]);

        let loading_list = List::new(vec![loading])
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(loading_list, area);
        return;
    }

    // If there's an error, show the error message
    if let LoadingState::Error(ref msg) = app.loading_state {
        let error = ListItem::new(vec![Spans::from(vec![Span::styled(
            format!("Error: {}", msg),
            Style::default().fg(Color::Red),
        )])]);

        let error_list =
            List::new(vec![error]).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(error_list, area);
        return;
    }

    // If we have no items, show a message
    if items.is_empty() {
        let empty = ListItem::new(vec![Spans::from(vec![Span::styled(
            "No items found",
            Style::default().fg(Color::Gray),
        )])]);

        let empty_list =
            List::new(vec![empty]).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(empty_list, area);
        return;
    }

    // Otherwise show the list of items
    let items_count = items.len();
    let mut list_state = ratatui::widgets::ListState::default();
    if items_count > 0 {
        list_state.select(Some(app.selected_index.min(items_count - 1)));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_repos_list<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect, title: &str) {
    let items: Vec<ListItem> = app
        .repos
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let name = &r.full_name;
            let desc = r.description.clone().unwrap_or_default();
            let stars = format!("★ {}", r.stargazers_count);
            let forks = format!("🍴 {}", r.forks_count);

            let content = vec![
                Spans::from(vec![Span::styled(
                    name,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]),
                Spans::from(vec![Span::raw(desc)]),
                Spans::from(vec![
                    Span::styled(stars, Style::default().fg(Color::Yellow)),
                    Span::raw(" | "),
                    Span::styled(forks, Style::default().fg(Color::Blue)),
                ]),
            ];

            ListItem::new(content).style(Style::default().fg(if i == app.selected_index {
                Color::Yellow
            } else {
                Color::White
            }))
        })
        .collect();

    // Handle loading, error, and empty states (similar to draw_crates_list)
    if matches!(app.loading_state, LoadingState::Loading) {
        let loading = ListItem::new(vec![Spans::from(vec![Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )])]);

        let loading_list =
            List::new(vec![loading]).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(loading_list, area);
        return;
    }

    if let LoadingState::Error(ref msg) = app.loading_state {
        let error = ListItem::new(vec![Spans::from(vec![Span::styled(
            format!("Error: {}", msg),
            Style::default().fg(Color::Red),
        )])]);

        let error_list =
            List::new(vec![error]).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(error_list, area);
        return;
    }

    if items.is_empty() {
        let empty = ListItem::new(vec![Spans::from(vec![Span::styled(
            "No items found",
            Style::default().fg(Color::Gray),
        )])]);

        let empty_list =
            List::new(vec![empty]).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(empty_list, area);
        return;
    }

    // Create and update list state
    let items_count = items.len();
    let mut list_state = ratatui::widgets::ListState::default();
    if items_count > 0 {
        list_state.select(Some(app.selected_index.min(items_count - 1)));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_search_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),    // Search results
        ])
        .split(area);

    // Draw search input
    let search_input = Paragraph::new(app.search_query.as_ref())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Search Query"));

    f.render_widget(search_input, chunks[0]);

    // Draw search results
    draw_crates_list(f, app, chunks[1], "Search Results");
}

fn draw_help<B: Backend>(f: &mut Frame<B>, _app: &App, area: Rect) {
    let text = vec![
        Spans::from(Span::styled(
            "Crates Explorer - Help",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            "Keyboard Controls:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("Shift+Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" - Switch between tabs"),
        ]),
        Spans::from(vec![
            Span::styled("j", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("Down", Style::default().fg(Color::Cyan)),
            Span::raw(" - Move down"),
        ]),
        Spans::from(vec![
            Span::styled("k", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("Up", Style::default().fg(Color::Cyan)),
            Span::raw(" - Move up"),
        ]),
        Spans::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" - Show details"),
        ]),
        Spans::from(vec![
            Span::styled("/", Style::default().fg(Color::Cyan)),
            Span::raw(" - Search"),
        ]),
        Spans::from(vec![
            Span::styled("1-4", Style::default().fg(Color::Cyan)),
            Span::raw(" - Switch tabs directly"),
        ]),
        Spans::from(vec![
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Cyan)),
            Span::raw(" - Quit"),
        ]),
        Spans::from(""),
        Spans::from(Span::styled(
            "Tab Guide:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("Recent", Style::default().fg(Color::Green)),
            Span::raw(" - Recently updated crates"),
        ]),
        Spans::from(vec![
            Span::styled("Search", Style::default().fg(Color::Green)),
            Span::raw(" - Search for crates by name"),
        ]),
        Spans::from(vec![
            Span::styled("Trending", Style::default().fg(Color::Green)),
            Span::raw(" - Trending Rust repositories on GitHub"),
        ]),
        Spans::from(vec![
            Span::styled("Help", Style::default().fg(Color::Green)),
            Span::raw(" - This help screen"),
        ]),
    ];

    let help = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

fn draw_status_bar<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let current_mode = match app.current_tab {
        Tab::Recent => "Recent",
        Tab::Search => "Search",
        Tab::Trending => "Trending",
        Tab::Help => "Help",
    };

    let status = format!("Mode: {} | Press 'q' to quit", current_mode);
    let status_bar = Paragraph::new(Span::styled(
        status,
        Style::default().fg(Color::White).bg(Color::DarkGray),
    ))
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(status_bar, area);
}
