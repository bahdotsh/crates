use reqwest::blocking::Client;
use serde::Deserialize;

const CRATES_API: &str = "https://crates.io/api/v1";
const GITHUB_API: &str = "https://api.github.com";

#[derive(Debug, Deserialize)]
pub struct Crate {
    pub name: String,
    pub description: Option<String>,
    pub downloads: u64,
    pub created_at: String,
    pub updated_at: String,
    pub documentation: Option<String>,
    pub repository: Option<String>,
    pub max_version: String,
}

#[derive(Debug, Deserialize)]
struct CratesResponse {
    crates: Vec<Crate>,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub language: Option<String>,
}

pub fn search_crates(query: &str, limit: usize) -> Result<Vec<Crate>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "{}/crates?q={}&sort=downloads&per_page={}",
        CRATES_API, query, limit
    );

    let response = client
        .get(&url)
        .header("User-Agent", "crates cli app")
        .send()?
        .json::<CratesResponse>()?;

    Ok(response.crates)
}

pub fn recent_crates(limit: usize) -> Result<Vec<Crate>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "{}/crates?sort=recent-updates&per_page={}",
        CRATES_API, limit
    );

    let response = client
        .get(&url)
        .header("User-Agent", "crates cli app")
        .send()?
        .json::<CratesResponse>()?;

    Ok(response.crates)
}

pub fn trending_repos(
    period: &str,
    limit: usize,
) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
    let client = Client::new();

    // GitHub API doesn't directly provide "trending" repositories,
    // so we need to search for popular Rust repos created in the recent period
    let since = match period {
        "daily" => "2023-01-01",   // This would need to be calculated dynamically
        "weekly" => "2023-01-01",  // This would need to be calculated dynamically
        "monthly" => "2023-01-01", // This would need to be calculated dynamically
        _ => "2023-01-01",
    };

    let url = format!(
        "{}/search/repositories?q=language:rust+created:>{}&sort=stars&order=desc&per_page={}",
        GITHUB_API, since, limit
    );

    let response = client
        .get(&url)
        .header("User-Agent", "crates cli app")
        .send()?
        .json::<serde_json::Value>()?;

    let items = response["items"].as_array().unwrap();
    let mut repos = Vec::new();

    for item in items {
        let repo = serde_json::from_value::<Repository>(item.clone())?;
        repos.push(repo);
    }

    Ok(repos)
}
