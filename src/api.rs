use chrono::DateTime;
use reqwest::blocking::Client;
use serde::Deserialize;

const CRATES_API: &str = "https://crates.io/api/v1";
const GITHUB_API: &str = "https://api.github.com";

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Crate {
    pub name: String,
    pub description: Option<String>,
    pub downloads: u64,
    pub created_at: String,
    pub updated_at: String,
    pub documentation: Option<String>,
    pub repository: Option<String>,
    pub max_version: String,
    pub license: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct CratesResponse {
    crates: Vec<Crate>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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

pub fn get_crate_details(name: &str) -> Result<Crate, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/crates/{}", CRATES_API, name);

    let response = client
        .get(&url)
        .header("User-Agent", "crates cli app")
        .send()?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch crate details: {}", response.status()).into());
    }

    let json: serde_json::Value = response.json()?;
    let crate_data = &json["crate"];

    // Parse the crate data from the response
    let crate_info: Crate = serde_json::from_value(crate_data.clone())?;

    Ok(crate_info)
}

// Security check for crates - simple heuristic approach
pub fn security_check(crate_data: &Crate) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check for indicators that might suggest security issues

    // 1. No license specified
    if crate_data.license.is_none() || crate_data.license.as_deref() == Some("") {
        warnings.push("No license specified".to_string());
    }

    // 2. Recent crate with high downloads could be suspicious
    if let Ok(created) = DateTime::parse_from_rfc3339(&crate_data.created_at) {
        // Fix the DateTime conversion
        let created_utc = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            created.naive_utc(),
            chrono::Utc,
        );
        let age = chrono::Utc::now().signed_duration_since(created_utc);

        if age.num_days() < 30 && crate_data.downloads > 10000 {
            warnings.push("New crate with unusually high download count".to_string());
        }
    }

    // 3. Check name similarity to popular crates (potential typosquatting)
    let typosquatting_targets = ["serde", "tokio", "reqwest", "actix", "rocket", "diesel"];
    for target in typosquatting_targets {
        if crate_data.name != target && levenshtein_distance(&crate_data.name, target) <= 2 {
            warnings.push(format!("Name similar to popular crate '{}'", target));
        }
    }

    // 4. No repository link
    if crate_data.repository.is_none() {
        warnings.push("No repository link".to_string());
    }

    warnings
}

// Simple Levenshtein distance implementation for detecting similar crate names
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let s1_len = s1_chars.len();
    let s2_len = s2_chars.len();

    let mut matrix = vec![vec![0; s2_len + 1]; s1_len + 1];

    for i in 0..=s1_len {
        matrix[i][0] = i;
    }

    for j in 0..=s2_len {
        matrix[0][j] = j;
    }

    for j in 1..=s2_len {
        for i in 1..=s1_len {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = std::cmp::min(
                matrix[i - 1][j] + 1,
                std::cmp::min(matrix[i][j - 1] + 1, matrix[i - 1][j - 1] + cost),
            );
        }
    }

    matrix[s1_len][s2_len]
}
