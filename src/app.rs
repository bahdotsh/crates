use crate::api::{self, Crate, Repository};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::error;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(PartialEq)]
pub enum Tab {
    Search,
    Recent,
    Trending,
    Compare,
    Help,
}

pub enum LoadingState {
    NotLoading,
    Loading,
    Loaded,
    Error(String),
}

pub struct SecurityInfo {
    pub warnings: Vec<String>,
    pub safe: bool,
}

#[allow(dead_code)]
pub struct ComparedCrate {
    pub details: Crate,
    pub security: SecurityInfo,
    pub selected: bool,
}

pub struct App {
    pub running: bool,
    pub current_tab: Tab,
    pub crates: Vec<Crate>,
    pub repos: Vec<Repository>,
    pub search_query: String,
    pub selected_index: usize,
    pub loading_state: LoadingState,
    pub trend_period: String,
    pub show_detail: bool,
    pub input_mode: bool,
    pub detail_scroll: usize,
    pub compared_crates: Vec<ComparedCrate>,
    pub compare_search_query: String,
    pub compare_input_mode: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            running: true,
            current_tab: Tab::Search,
            crates: Vec::new(),
            repos: Vec::new(),
            search_query: String::new(),
            selected_index: 0,
            loading_state: LoadingState::NotLoading,
            trend_period: "weekly".to_string(),
            show_detail: false,
            input_mode: false,
            detail_scroll: 0,
            compared_crates: Vec::new(),
            compare_search_query: String::new(),
            compare_input_mode: false,
        };

        // Load initial data
        app.load_recent_crates();

        app
    }

    pub fn tick(&mut self) {
        // Update app state on tick
        match self.loading_state {
            LoadingState::Loading => match self.current_tab {
                Tab::Recent => self.load_recent_crates(),
                Tab::Trending => self.load_trending_repos(),
                Tab::Search => {
                    if !self.search_query.is_empty() {
                        self.search_crates();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn add_to_comparison(&mut self) {
        if self.current_tab == Tab::Recent || self.current_tab == Tab::Search {
            if !self.crates.is_empty() && self.selected_index < self.crates.len() {
                let current_crate = &self.crates[self.selected_index];

                // Check if already in comparison
                if self
                    .compared_crates
                    .iter()
                    .any(|c| c.details.name == current_crate.name)
                {
                    return; // Already added
                }

                // Fetch full details for the crate
                match api::get_crate_details(&current_crate.name) {
                    Ok(details) => {
                        let security_warnings = api::security_check(&details);
                        self.compared_crates.push(ComparedCrate {
                            details,
                            security: SecurityInfo {
                                warnings: security_warnings.clone(),
                                safe: security_warnings.is_empty(),
                            },
                            selected: false,
                        });
                    }
                    Err(_) => {
                        // If we can't get details, use the basic info we have
                        let security_warnings = api::security_check(current_crate);
                        self.compared_crates.push(ComparedCrate {
                            details: current_crate.clone(),
                            security: SecurityInfo {
                                warnings: security_warnings.clone(),
                                safe: security_warnings.is_empty(),
                            },
                            selected: false,
                        });
                    }
                }
            }
        }
    }

    pub fn remove_from_comparison(&mut self) {
        if self.current_tab == Tab::Compare && !self.compared_crates.is_empty() {
            self.compared_crates.remove(self.selected_index);
            if self.selected_index >= self.compared_crates.len() && !self.compared_crates.is_empty()
            {
                self.selected_index = self.compared_crates.len() - 1;
            }
        }
    }

    pub fn add_crate_to_comparison_by_name(&mut self, name: &str) {
        match api::get_crate_details(name) {
            Ok(details) => {
                // Check if already in comparison
                if self
                    .compared_crates
                    .iter()
                    .any(|c| c.details.name == details.name)
                {
                    return; // Already added
                }

                let security_warnings = api::security_check(&details);
                self.compared_crates.push(ComparedCrate {
                    details,
                    security: SecurityInfo {
                        warnings: security_warnings.clone(),
                        safe: security_warnings.is_empty(),
                    },
                    selected: false,
                });
            }
            Err(_) => {
                // Handle error - perhaps show a message to the user
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Handle quit event in any mode
        if key.code == KeyCode::Char('q') && !self.input_mode && !self.compare_input_mode {
            self.running = false;
            return;
        }

        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        // Handle detail view mode
        if self.show_detail {
            self.handle_detail_mode(key);
            return;
        }

        // Handle input mode separately
        if self.input_mode {
            self.handle_input_mode(key);
            return;
        }

        // Handle compare input mode separately
        if self.compare_input_mode {
            match key.code {
                KeyCode::Enter => {
                    self.compare_input_mode = false;
                    if !self.compare_search_query.is_empty() {
                        let query = self.compare_search_query.clone();
                        self.add_crate_to_comparison_by_name(&query);
                        self.compare_search_query.clear();
                    }
                }
                KeyCode::Esc => {
                    self.compare_input_mode = false;
                }
                KeyCode::Char(c) => {
                    self.compare_search_query.push(c);
                }
                KeyCode::Backspace => {
                    self.compare_search_query.pop();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Tab => {
                self.next_tab();
            }
            KeyCode::BackTab => {
                self.prev_tab();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.next_item();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.prev_item();
            }
            KeyCode::Enter => {
                self.show_detail = true;
                self.detail_scroll = 0;
            }
            KeyCode::Char('1') => {
                self.current_tab = Tab::Search;
            }
            KeyCode::Char('2') => {
                self.current_tab = Tab::Recent;
                self.load_recent_crates();
            }
            KeyCode::Char('3') => {
                self.current_tab = Tab::Trending;
                self.load_trending_repos();
            }
            KeyCode::Char('4') => {
                self.current_tab = Tab::Help;
            }
            KeyCode::Char('5') => {
                self.current_tab = Tab::Compare;
            }
            KeyCode::Char('/') => {
                if matches!(self.current_tab, Tab::Search) {
                    self.input_mode = true;
                    self.search_query.clear(); // Clear previous query when starting new search
                }
            }
            KeyCode::Char('a') => {
                if matches!(self.current_tab, Tab::Search)
                    || matches!(self.current_tab, Tab::Recent)
                {
                    self.add_to_comparison();
                } else if matches!(self.current_tab, Tab::Compare) {
                    self.compare_input_mode = true;
                }
            }
            KeyCode::Char('d') => {
                if matches!(self.current_tab, Tab::Compare) {
                    self.remove_from_comparison();
                }
            }
            _ => {}
        }
    }

    fn handle_detail_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.show_detail = false;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.detail_scroll = self.detail_scroll.saturating_add(10);
            }
            KeyCode::PageUp => {
                self.detail_scroll = self.detail_scroll.saturating_sub(10);
            }
            _ => {}
        }
    }

    fn handle_input_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.input_mode = false;
                if !self.search_query.is_empty() {
                    self.search_crates();
                    self.selected_index = 0; // Reset selection to the top result
                }
            }
            KeyCode::Esc => {
                self.input_mode = false;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Tab => {
                // Auto-complete functionality could be added here
            }
            _ => {}
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Search => Tab::Recent,
            Tab::Recent => Tab::Trending,
            Tab::Trending => Tab::Compare,
            Tab::Compare => Tab::Help,
            Tab::Help => Tab::Search,
        };
        self.selected_index = 0;
        self.show_detail = false;

        // Just set loading state but don't actually load
        match self.current_tab {
            Tab::Recent => {
                if self.crates.is_empty() {
                    self.loading_state = LoadingState::Loading;
                }
            }
            Tab::Trending => {
                if self.repos.is_empty() {
                    self.loading_state = LoadingState::Loading;
                }
            }
            _ => {}
        }
    }

    fn prev_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Search => Tab::Help,
            Tab::Recent => Tab::Search,
            Tab::Trending => Tab::Recent,
            Tab::Compare => Tab::Trending,
            Tab::Help => Tab::Compare,
        };
        self.selected_index = 0;
        self.show_detail = false;

        // Just set loading state but don't actually load
        match self.current_tab {
            Tab::Recent => {
                if self.crates.is_empty() {
                    self.loading_state = LoadingState::Loading;
                }
            }
            Tab::Trending => {
                if self.repos.is_empty() {
                    self.loading_state = LoadingState::Loading;
                }
            }
            _ => {}
        }
    }

    fn next_item(&mut self) {
        let max = match self.current_tab {
            Tab::Recent | Tab::Search => self.crates.len(),
            Tab::Trending => self.repos.len(),
            Tab::Compare => self.compared_crates.len(),
            Tab::Help => 0,
        };

        if max > 0 {
            self.selected_index = (self.selected_index + 1) % max;
        }
    }

    fn prev_item(&mut self) {
        let max = match self.current_tab {
            Tab::Recent | Tab::Search => self.crates.len(),
            Tab::Trending => self.repos.len(),
            Tab::Compare => self.compared_crates.len(),
            Tab::Help => 0,
        };

        if max > 0 {
            self.selected_index = if self.selected_index > 0 {
                self.selected_index - 1
            } else {
                max - 1
            };
        }
    }

    fn load_recent_crates(&mut self) {
        self.loading_state = LoadingState::Loading;

        // Fetch data
        let app_result = api::recent_crates(20);
        match app_result {
            Ok(crates) => {
                self.crates = crates;
                self.loading_state = LoadingState::Loaded;
            }
            Err(e) => {
                self.loading_state = LoadingState::Error(e.to_string());
            }
        }
    }

    fn load_trending_repos(&mut self) {
        self.loading_state = LoadingState::Loading;

        // Fetch data
        let app_result = api::trending_repos(&self.trend_period, 20);
        match app_result {
            Ok(repos) => {
                self.repos = repos;
                self.loading_state = LoadingState::Loaded;
            }
            Err(e) => {
                self.loading_state = LoadingState::Error(e.to_string());
            }
        }
    }

    pub fn search_crates(&mut self) {
        if self.search_query.is_empty() {
            return;
        }

        self.loading_state = LoadingState::Loading;

        match api::search_crates(&self.search_query, 20) {
            Ok(crates) => {
                self.crates = crates;
                self.loading_state = LoadingState::Loaded;
            }
            Err(e) => {
                self.loading_state = LoadingState::Error(e.to_string());
            }
        }
    }
    pub fn search_crates_silently(&mut self, query: &str) {
        self.loading_state = LoadingState::Loading;

        match api::search_crates(query, 20) {
            Ok(crates) => {
                self.crates = crates;
                self.loading_state = LoadingState::Loaded;
            }
            Err(e) => {
                self.loading_state = LoadingState::Error(e.to_string());
            }
        }
    }
}
