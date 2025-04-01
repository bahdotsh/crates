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

    // Draw content based on current tab and detail view
    if app.show_detail {
        match app.current_tab {
            Tab::Recent | Tab::Search => {
                if !app.crates.is_empty() && app.selected_index < app.crates.len() {
                    draw_crate_detail(f, app, chunks[2]);
                }
            }
            Tab::Trending => {
                if !app.repos.is_empty() && app.selected_index < app.repos.len() {
                    draw_repo_detail(f, app, chunks[2]);
                }
            }
            _ => {}
        }
    } else {
        match app.current_tab {
            Tab::Search => draw_search_tab(f, app, chunks[2]),
            Tab::Recent => draw_crates_list(f, app, chunks[2], "Recent Crates"),
            Tab::Trending => draw_repos_list(f, app, chunks[2], "Trending Repositories"),
            Tab::Help => draw_help(f, app, chunks[2]),
        }
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
    let titles = ["Search", "Recent", "Trending", "Help"]
        .iter()
        .map(|t| Spans::from(vec![Span::styled(*t, Style::default().fg(Color::White))]))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(match app.current_tab {
            Tab::Search => 0,
            Tab::Recent => 1,
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
                Spans::from(vec![Span::raw(truncate_str(&desc, 60))]),
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
            let language = r.language.clone().unwrap_or_else(|| "Unknown".to_string());

            let content = vec![
                Spans::from(vec![Span::styled(
                    name,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]),
                Spans::from(vec![Span::raw(truncate_str(&desc, 60))]),
                Spans::from(vec![
                    Span::styled(stars, Style::default().fg(Color::Yellow)),
                    Span::raw(" | "),
                    Span::styled(forks, Style::default().fg(Color::Blue)),
                    Span::raw(" | Language: "),
                    Span::styled(language, Style::default().fg(Color::Magenta)),
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

    // Draw search input - highlight when in input mode
    let search_input = Paragraph::new(app.search_query.as_ref())
        .style(if app.input_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if app.input_mode {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                })
                .title(if app.input_mode {
                    "Enter search query (press Enter to search)"
                } else {
                    "Search Query (press / to search)"
                }),
        );

    f.render_widget(search_input, chunks[0]);

    // Draw search results with a better title
    let title = if app.search_query.is_empty() {
        "Type / to search for crates"
    } else {
        "Search Results"
    };

    draw_crates_list(f, app, chunks[1], title);
}

fn draw_crate_detail<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let crate_data = &app.crates[app.selected_index];

    let title = format!("{} v{}", crate_data.name, crate_data.max_version);

    let mut content = vec![
        Spans::from(vec![Span::styled(
            "Description:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw(
            crate_data
                .description
                .clone()
                .unwrap_or_else(|| "No description available.".to_string()),
        )]),
        Spans::from(vec![]),
        Spans::from(vec![
            Span::styled(
                "Downloads: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", crate_data.downloads),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Spans::from(vec![]),
        Spans::from(vec![
            Span::styled(
                "Created: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format_date(&crate_data.created_at),
                Style::default().fg(Color::White),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                "Updated: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format_date(&crate_data.updated_at),
                Style::default().fg(Color::White),
            ),
        ]),
        Spans::from(vec![]),
    ];

    if let Some(ref docs) = crate_data.documentation {
        content.push(Spans::from(vec![
            Span::styled(
                "Documentation: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                docs,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]));
    }

    if let Some(ref repo) = crate_data.repository {
        content.push(Spans::from(vec![
            Span::styled(
                "Repository: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                repo,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]));
    }

    // Add navigation help
    content.extend_from_slice(&[
        Spans::from(vec![]),
        Spans::from(vec![]),
        Spans::from(vec![Span::styled(
            "Press ESC or q to go back",
            Style::default().fg(Color::Gray),
        )]),
    ]);

    let detail = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true })
        .scroll((app.detail_scroll as u16, 0));

    f.render_widget(detail, area);
}

fn draw_repo_detail<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let repo_data = &app.repos[app.selected_index];

    let title = &repo_data.full_name;

    let content = vec![
        Spans::from(vec![Span::styled(
            "Description:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw(
            repo_data
                .description
                .clone()
                .unwrap_or_else(|| "No description available.".to_string()),
        )]),
        Spans::from(vec![]),
        Spans::from(vec![
            Span::styled(
                "Stars: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("★ {}", repo_data.stargazers_count),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                "Forks: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("🍴 {}", repo_data.forks_count),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Spans::from(vec![]),
        Spans::from(vec![
            Span::styled(
                "Language: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                repo_data
                    .language
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Spans::from(vec![]),
        Spans::from(vec![
            Span::styled(
                "URL: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &repo_data.html_url,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]),
        // Add navigation help
        Spans::from(vec![]),
        Spans::from(vec![]),
        Spans::from(vec![Span::styled(
            "Press ESC or q to go back",
            Style::default().fg(Color::Gray),
        )]),
    ];

    let detail = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title.as_str()))
        .wrap(Wrap { trim: true })
        .scroll((app.detail_scroll as u16, 0));

    f.render_widget(detail, area);
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
            Span::raw(" - Search (in Search tab)"),
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
        Spans::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" - Exit detail view or search input"),
        ]),
        Spans::from(""),
        Spans::from(Span::styled(
            "In Detail View:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Spans::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Cyan)),
            Span::raw(" - Scroll up/down"),
        ]),
        Spans::from(vec![
            Span::styled("PageUp/PageDown", Style::default().fg(Color::Cyan)),
            Span::raw(" - Scroll by page"),
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
            Span::styled("Search", Style::default().fg(Color::Green)),
            Span::raw(" - Search for crates by name"),
        ]),
        Spans::from(vec![
            Span::styled("Recent", Style::default().fg(Color::Green)),
            Span::raw(" - Recently updated crates"),
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
    let mode_text = match app.current_tab {
        Tab::Search => {
            if app.show_detail {
                "Search > Crate Detail"
            } else if app.input_mode {
                "Search > Input Mode"
            } else {
                "Search"
            }
        }
        Tab::Recent => {
            if app.show_detail {
                "Recent > Crate Detail"
            } else {
                "Recent"
            }
        }
        Tab::Trending => {
            if app.show_detail {
                "Trending > Repository Detail"
            } else {
                "Trending"
            }
        }
        Tab::Help => "Help",
    };

    let navigation_help = if app.show_detail {
        "ESC to go back | j/k to scroll"
    } else if app.input_mode {
        "ESC to cancel | Enter to search"
    } else if matches!(app.current_tab, Tab::Search) {
        "/ to search | Enter to view details | q to quit"
    } else {
        "Enter to view details | q to quit"
    };

    let status = format!("{} | {}", mode_text, navigation_help);

    let status_bar = Paragraph::new(Span::styled(
        status,
        Style::default().fg(Color::White).bg(Color::DarkGray),
    ))
    .block(Block::default().borders(Borders::ALL))
    .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(status_bar, area);
}

// Helper function to format dates nicely
fn format_date(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        date_str.to_string()
    }
}

// Helper function to truncate strings to a maximum length
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
