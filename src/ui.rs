use crate::api;
use crate::app::{App, LoadingState, Tab};
use chrono::DateTime;

use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
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
            Tab::Compare => {
                if !app.compared_crates.is_empty() && app.selected_index < app.compared_crates.len()
                {
                    draw_compared_crate_detail(f, app, chunks[2]);
                }
            }
            _ => {}
        }
    } else {
        match app.current_tab {
            Tab::Search => draw_search_tab(f, app, chunks[2]),
            Tab::Recent => draw_crates_list(f, app, chunks[2], "Recent Crates"),
            Tab::Trending => draw_repos_list(f, app, chunks[2], "Trending Repositories"),
            Tab::Compare => draw_compare_tab(f, app, chunks[2]),
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
    let titles = ["Search", "Recent", "Trending", "Compare", "Help"]
        .iter()
        .map(|t| Line::from(vec![Span::styled(*t, Style::default().fg(Color::White))]))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(match app.current_tab {
            Tab::Search => 0,
            Tab::Recent => 1,
            Tab::Trending => 2,
            Tab::Compare => 3,
            Tab::Help => 4,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn draw_compare_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input for adding crates
            Constraint::Min(0),    // Comparison table
        ])
        .split(area);

    // Draw input for adding crates
    let input_style = if app.compare_input_mode {
        Style::default().fg(Color::Blue)
    } else {
        Style::default()
    };

    let cursor_position = if app.compare_input_mode {
        Some(app.compare_search_query.len())
    } else {
        None
    };

    let search_prompt = if app.compare_search_query.is_empty() && !app.compare_input_mode {
        "Add a crate by name..."
    } else {
        ""
    };

    let search_input = Paragraph::new(
        if app.compare_search_query.is_empty() && !app.compare_input_mode {
            Text::styled(search_prompt, Style::default().fg(Color::DarkGray))
        } else {
            Text::raw(&app.compare_search_query)
        },
    )
    .style(input_style)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if app.compare_input_mode {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            })
            .title(if app.compare_input_mode {
                "Adding crate..."
            } else {
                "Press 'a' to add a crate | 'd' to remove selected"
            }),
    );

    f.render_widget(search_input, chunks[0]);

    // Render cursor position when in input mode
    if app.compare_input_mode && cursor_position.is_some() {
        f.set_cursor(
            chunks[0].x + 1 + cursor_position.unwrap() as u16,
            chunks[0].y + 1,
        );
    }

    // Draw comparison table if there are crates to compare
    if app.compared_crates.is_empty() {
        let no_crates = Paragraph::new("No crates added for comparison. Press 'a' to add crates.")
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Comparison"));

        f.render_widget(no_crates, chunks[1]);
        return;
    }

    // Create a layout for the comparison table
    // The first column is for crate names, the rest for metrics
    let column_constraints = vec![
        Constraint::Percentage(25), // Name
        Constraint::Percentage(15), // Downloads
        Constraint::Percentage(15), // License
        Constraint::Percentage(15), // Security
        Constraint::Percentage(15), // Updated
        Constraint::Percentage(15), // Version
    ];

    let header_cells = [
        "Crate",
        "Downloads",
        "License",
        "Security",
        "Updated",
        "Version",
    ]
    .iter()
    .map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let mut rows = vec![];
    for (i, compared) in app.compared_crates.iter().enumerate() {
        let crate_data = &compared.details;

        // Style for highlighting the selected row
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Security status indicator
        let security_status = if compared.security.safe {
            "✓ Safe"
        } else {
            "⚠ Warning"
        };

        // License display
        let license_display = match &crate_data.license {
            Some(license) if !license.is_empty() => license,
            _ => "Unknown",
        };

        // Format the updated date
        let updated = if let Ok(dt) = DateTime::parse_from_rfc3339(&crate_data.updated_at) {
            dt.format("%Y-%m-%d").to_string()
        } else {
            "Unknown".to_string()
        };

        // Create the row
        let cells = vec![
            Cell::from(crate_data.name.clone()),
            Cell::from(format!("{}", crate_data.downloads)),
            Cell::from(license_display),
            Cell::from(security_status).style(if compared.security.safe {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            }),
            Cell::from(updated),
            Cell::from(crate_data.max_version.clone()),
        ];

        rows.push(Row::new(cells).style(style));
    }

    let table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Comparison"))
        .widths(&column_constraints)
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(table, chunks[1]);
}

fn draw_compared_crate_detail<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let compared = &app.compared_crates[app.selected_index];
    let crate_data = &compared.details;

    let title = format!("{} v{}", crate_data.name, crate_data.max_version);

    let mut content = vec![
        Line::from(vec![Span::styled(
            "Description:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw(
            crate_data
                .description
                .clone()
                .unwrap_or_else(|| "No description available.".to_string()),
        )]),
        Line::from(vec![]),
        // License information
        Line::from(vec![
            Span::styled(
                "License: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                crate_data
                    .license
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![]),
        // Security information
        Line::from(vec![Span::styled(
            "Security Check:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    if compared.security.warnings.is_empty() {
        content.push(Line::from(vec![Span::styled(
            "✓ No security issues detected",
            Style::default().fg(Color::Green),
        )]));
    } else {
        content.push(Line::from(vec![Span::styled(
            "⚠ Security warnings:",
            Style::default().fg(Color::Red),
        )]));

        for warning in &compared.security.warnings {
            content.push(Line::from(vec![Span::styled(
                format!("  • {}", warning),
                Style::default().fg(Color::Red),
            )]));
        }
    }

    content.extend_from_slice(&[
        Line::from(vec![]),
        Line::from(vec![
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
        Line::from(vec![]),
        Line::from(vec![
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
        Line::from(vec![
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
        Line::from(vec![]),
    ]);

    if let Some(ref docs) = crate_data.documentation {
        content.push(Line::from(vec![
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
        content.push(Line::from(vec![
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
        Line::from(vec![]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
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
fn draw_crates_list<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect, title: &str) {
    let items: Vec<ListItem> = app
        .crates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = format!("{} v{}", c.name, c.max_version);
            let desc = c.description.clone().unwrap_or_default();
            let downloads = format!("{} downloads", c.downloads);

            // Parse and format date
            let updated = if let Ok(dt) = DateTime::parse_from_rfc3339(&c.updated_at) {
                dt.format("%Y-%m-%d").to_string()
            } else {
                "".to_string()
            };

            let mut content = vec![];

            // Name with version
            content.push(Line::from(vec![Span::styled(
                name,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(if i == app.selected_index {
                        Modifier::BOLD | Modifier::UNDERLINED
                    } else {
                        Modifier::BOLD
                    }),
            )]));

            // Repository URL in green (if available)
            if let Some(repo) = &c.repository {
                content.push(Line::from(vec![Span::styled(
                    truncate_str(repo, 60),
                    Style::default().fg(Color::Green),
                )]));
            }

            // Description
            if !desc.is_empty() {
                content.push(Line::from(vec![Span::raw(truncate_str(&desc, 80))]));
            }

            // Stats line
            content.push(Line::from(vec![
                Span::styled(downloads, Style::default().fg(Color::Yellow)),
                Span::raw(" · Updated: "),
                Span::styled(updated, Style::default().fg(Color::Gray)),
            ]));

            // Add a blank line between results for better readability
            content.push(Line::from(vec![Span::raw("")]));

            ListItem::new(content).style(if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            })
        })
        .collect();

    // If we're in loading state, show a loading message
    if matches!(app.loading_state, LoadingState::Loading) {
        let loading = ListItem::new(vec![Line::from(vec![Span::styled(
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
        let error = ListItem::new(vec![Line::from(vec![Span::styled(
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
        let empty = ListItem::new(vec![Line::from(vec![Span::styled(
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
                Line::from(vec![Span::styled(
                    name,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![Span::raw(truncate_str(&desc, 60))]),
                Line::from(vec![
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
        let loading = ListItem::new(vec![Line::from(vec![Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )])]);

        let loading_list =
            List::new(vec![loading]).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(loading_list, area);
        return;
    }

    if let LoadingState::Error(ref msg) = app.loading_state {
        let error = ListItem::new(vec![Line::from(vec![Span::styled(
            format!("Error: {}", msg),
            Style::default().fg(Color::Red),
        )])]);

        let error_list =
            List::new(vec![error]).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(error_list, area);
        return;
    }

    if items.is_empty() {
        let empty = ListItem::new(vec![Line::from(vec![Span::styled(
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
            Constraint::Length(1), // Small padding
            Constraint::Min(0),    // Search results
        ])
        .split(area);

    // Create a Google-like search input
    let input_style = if app.input_mode {
        Style::default().fg(Color::Blue)
    } else {
        Style::default()
    };

    let cursor_position = if app.input_mode {
        Some(app.search_query.len())
    } else {
        None
    };

    let search_prompt = if app.search_query.is_empty() && !app.input_mode {
        "Search for crates..."
    } else {
        ""
    };

    let search_input = Paragraph::new(if app.search_query.is_empty() && !app.input_mode {
        Text::styled(search_prompt, Style::default().fg(Color::DarkGray))
    } else {
        Text::raw(&app.search_query)
    })
    .style(input_style)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if app.input_mode {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            })
            .title(if app.input_mode {
                "🔍 Type to search"
            } else {
                "🔍 Press / to search"
            }),
    );

    f.render_widget(search_input, chunks[0]);

    // Render cursor position when in input mode
    if app.input_mode && cursor_position.is_some() {
        f.set_cursor(
            chunks[0].x + 1 + cursor_position.unwrap() as u16,
            chunks[0].y + 1,
        );
    }

    // Add stats about results if we have searched - use String instead of &str
    let stats_text = if !app.crates.is_empty() && !app.search_query.is_empty() {
        format!(
            "Found {} results for \"{}\"",
            app.crates.len(),
            app.search_query
        )
    } else {
        String::new()
    };

    let stats = Paragraph::new(stats_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Left);

    f.render_widget(stats, chunks[1]);

    // Draw search results with a simple title
    let title = if app.search_query.is_empty() {
        "Popular Crates"
    } else {
        "Search Results"
    };

    // Update the app to show popular crates when no search is active
    if app.search_query.is_empty()
        && app.crates.is_empty()
        && matches!(app.loading_state, LoadingState::NotLoading)
    {
        // Pretend we're searching for a broadly popular term to show useful results
        app.loading_state = LoadingState::Loading;
        app.search_crates_silently("rust");
    }

    draw_crates_list(f, app, chunks[2], title);
}

fn draw_crate_detail<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let crate_data = &app.crates[app.selected_index];

    let title = format!("{} v{}", crate_data.name, crate_data.max_version);

    // Run security check
    let security_warnings = api::security_check(crate_data);
    let is_safe = security_warnings.is_empty();

    let mut content = vec![
        Line::from(vec![Span::styled(
            "Description:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw(
            crate_data
                .description
                .clone()
                .unwrap_or_else(|| "No description available.".to_string()),
        )]),
        Line::from(vec![]),
        // Add license information
        Line::from(vec![
            Span::styled(
                "License: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                crate_data
                    .license
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
                if crate_data.license.is_some() {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ]),
        Line::from(vec![]),
        // Add security information
        Line::from(vec![Span::styled(
            "Security Check:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    if is_safe {
        content.push(Line::from(vec![Span::styled(
            "✓ No security issues detected",
            Style::default().fg(Color::Green),
        )]));
    } else {
        content.push(Line::from(vec![Span::styled(
            "⚠ Security warnings:",
            Style::default().fg(Color::Red),
        )]));

        for warning in &security_warnings {
            content.push(Line::from(vec![Span::styled(
                format!("  • {}", warning),
                Style::default().fg(Color::Red),
            )]));
        }
    }

    content.extend_from_slice(&[
        Line::from(vec![]),
        Line::from(vec![
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
        Line::from(vec![]),
        Line::from(vec![
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
        Line::from(vec![
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
        Line::from(vec![]),
    ]);

    if let Some(ref docs) = crate_data.documentation {
        content.push(Line::from(vec![
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
        content.push(Line::from(vec![
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

    // Add option to add to comparison
    content.extend_from_slice(&[
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Press 'a' to add to comparison",
            Style::default().fg(Color::Blue),
        )]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
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
        Line::from(vec![Span::styled(
            "Description:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw(
            repo_data
                .description
                .clone()
                .unwrap_or_else(|| "No description available.".to_string()),
        )]),
        Line::from(vec![]),
        Line::from(vec![
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
        Line::from(vec![
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
        Line::from(vec![]),
        Line::from(vec![
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
        Line::from(vec![]),
        Line::from(vec![
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
        Line::from(vec![]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
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
        Line::from(Span::styled(
            "Crates Explorer - Help",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Keyboard Controls:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("Shift+Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" - Switch between tabs"),
        ]),
        Line::from(vec![
            Span::styled("j", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("Down", Style::default().fg(Color::Cyan)),
            Span::raw(" - Move down"),
        ]),
        Line::from(vec![
            Span::styled("k", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("Up", Style::default().fg(Color::Cyan)),
            Span::raw(" - Move up"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" - Show details"),
        ]),
        Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Cyan)),
            Span::raw(" - Search (in Search tab)"),
        ]),
        Line::from(vec![
            Span::styled("1-4", Style::default().fg(Color::Cyan)),
            Span::raw(" - Switch tabs directly"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Cyan)),
            Span::raw(" - Quit"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" - Exit detail view or search input"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "In Detail View:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Cyan)),
            Span::raw(" - Scroll up/down"),
        ]),
        Line::from(vec![
            Span::styled("PageUp/PageDown", Style::default().fg(Color::Cyan)),
            Span::raw(" - Scroll by page"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Tab Guide:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Search", Style::default().fg(Color::Green)),
            Span::raw(" - Search for crates by name"),
        ]),
        Line::from(vec![
            Span::styled("Recent", Style::default().fg(Color::Green)),
            Span::raw(" - Recently updated crates"),
        ]),
        Line::from(vec![
            Span::styled("Trending", Style::default().fg(Color::Green)),
            Span::raw(" - Trending Rust repositories on GitHub"),
        ]),
        Line::from(vec![
            Span::styled("Help", Style::default().fg(Color::Green)),
            Span::raw(" - This help screen"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "License & Security Features:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("• Crate details now include "),
            Span::styled("license information", Style::default().fg(Color::Green)),
            Span::raw(" and "),
            Span::styled("security checks", Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![Span::raw(
            "• Security warnings highlight potential issues with crates",
        )]),
        Line::from(""),
        Line::from(Span::styled(
            "Compare Tab:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("5", Style::default().fg(Color::Cyan)),
            Span::raw(" - Switch to Compare tab"),
        ]),
        Line::from(vec![
            Span::styled("a", Style::default().fg(Color::Cyan)),
            Span::raw(" - Add current crate to comparison or add new crate by name"),
        ]),
        Line::from(vec![
            Span::styled("d", Style::default().fg(Color::Cyan)),
            Span::raw(" - Remove selected crate from comparison"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" - View detailed security and license info"),
        ]),
        Line::from(vec![Span::raw(
            "Compare key metrics across multiple crates side by side",
        )]),
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
        Tab::Compare => {
            if app.show_detail {
                "Compare > Crate Detail"
            } else if app.compare_input_mode {
                "Compare > Adding Crate"
            } else {
                "Compare"
            }
        }
        Tab::Help => "Help",
    };

    let navigation_help = if app.show_detail {
        "ESC to go back | j/k to scroll"
    } else if app.input_mode || app.compare_input_mode {
        "ESC to cancel | Enter to confirm"
    } else if matches!(app.current_tab, Tab::Search) {
        "/ to search | Enter to view details | a to add to comparison | q to quit"
    } else if matches!(app.current_tab, Tab::Recent) {
        "Enter to view details | a to add to comparison | q to quit"
    } else if matches!(app.current_tab, Tab::Compare) {
        "a to add crate | d to remove | Enter to view details | q to quit"
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
