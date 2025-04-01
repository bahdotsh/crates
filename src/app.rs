use crate::api::{self, Crate, Repository};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::error;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum Tab {
    Recent,
    Search,
    Trending,
    Help,
}

pub enum LoadingState {
    NotLoading,
    Loading,
    Loaded,
    Error(String),
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
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            running: true,
            current_tab: Tab::Recent,
            crates: Vec::new(),
            repos: Vec::new(),
            search_query: String::new(),
            selected_index: 0,
            loading_state: LoadingState::NotLoading,
            trend_period: "weekly".to_string(),
            show_detail: false,
            input_mode: false,
        };

        // Load initial data
        app.load_recent_crates();

        app
    }

    pub fn tick(&mut self) {
        // Update app state on tick
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Handle input mode separately
        if self.input_mode {
            self.handle_input_mode(key);
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.running = false;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
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
                self.toggle_detail();
            }
            KeyCode::Char('1') => {
                self.current_tab = Tab::Recent;
                self.load_recent_crates();
            }
            KeyCode::Char('2') => {
                self.current_tab = Tab::Search;
            }
            KeyCode::Char('3') => {
                self.current_tab = Tab::Trending;
                self.load_trending_repos();
            }
            KeyCode::Char('4') => {
                self.current_tab = Tab::Help;
            }
            KeyCode::Char('/') => {
                if matches!(self.current_tab, Tab::Search) {
                    self.input_mode = true;
                }
            }
            _ => {}
        }
    }

    fn handle_input_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.input_mode = false;
                self.search_crates();
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
            _ => {}
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Recent => Tab::Search,
            Tab::Search => Tab::Trending,
            Tab::Trending => Tab::Help,
            Tab::Help => Tab::Recent,
        };
        self.selected_index = 0;

        // Load data for the new tab
        match self.current_tab {
            Tab::Recent => self.load_recent_crates(),
            Tab::Trending => self.load_trending_repos(),
            _ => {}
        }
    }

    fn prev_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Recent => Tab::Help,
            Tab::Search => Tab::Recent,
            Tab::Trending => Tab::Search,
            Tab::Help => Tab::Trending,
        };
        self.selected_index = 0;

        // Load data for the new tab
        match self.current_tab {
            Tab::Recent => self.load_recent_crates(),
            Tab::Trending => self.load_trending_repos(),
            _ => {}
        }
    }

    fn next_item(&mut self) {
        let max = match self.current_tab {
            Tab::Recent | Tab::Search => self.crates.len(),
            Tab::Trending => self.repos.len(),
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

    fn toggle_detail(&mut self) {
        self.show_detail = !self.show_detail;
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
}
