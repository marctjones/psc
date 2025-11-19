//! Sovereign Reader - Personal news aggregator CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use sovereign_reader::{
    digest, llm, sources, storage::Storage, wikipedia::WikipediaClient, DigestFormat, Interests,
    Source, SourceType,
};
use std::process;

#[derive(Parser)]
#[command(name = "sovereign-reader")]
#[command(about = "Personal news aggregator and daily digest generator", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage news sources
    Source {
        #[command(subcommand)]
        command: SourceCommands,
    },
    /// Manage your interests for recommendations
    Interest {
        #[command(subcommand)]
        command: InterestCommands,
    },
    /// Fetch new articles from all enabled sources
    Fetch {
        /// Fetch from a specific source only
        #[arg(short, long)]
        source: Option<String>,
    },
    /// Generate daily digest
    Digest {
        /// Output format (markdown, html, plain, json)
        #[arg(short, long, default_value = "markdown")]
        format: String,
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show unread articles
    List {
        /// Maximum number of articles to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Mark an article as read
    Read {
        /// Article ID
        article_id: String,
    },
    /// Configure reader settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Discover RSS feeds on a website
    Discover {
        /// Website URL to analyze
        url: String,
        /// Automatically add discovered feeds
        #[arg(short, long)]
        add: bool,
    },
    /// Track a website for changes (for sites without RSS)
    Track {
        /// Website URL to track
        url: String,
        /// Name for the source
        name: String,
    },
    /// Check LLM assistant integration status
    LlmStatus,
    /// Score articles using LLM (requires LLM assistant)
    Score {
        /// Re-score all unread articles
        #[arg(long)]
        all: bool,
    },
    /// Summarize an article using LLM
    Summarize {
        /// Article ID to summarize
        article_id: String,
    },
    /// Analyze reading patterns and suggest interests
    Analyze,
}

#[derive(Subcommand)]
enum SourceCommands {
    /// List all sources
    List,
    /// Add a new source
    Add {
        /// Source name
        name: String,
        /// RSS/Atom feed URL
        url: String,
        /// Source ID (auto-generated if not provided)
        #[arg(short, long)]
        id: Option<String>,
    },
    /// Enable a source
    Enable {
        /// Source ID
        id: String,
    },
    /// Disable a source
    Disable {
        /// Source ID
        id: String,
    },
    /// Remove a source
    Remove {
        /// Source ID
        id: String,
    },
    /// Show available pre-configured sources
    Available,
    /// Add a pre-configured source by ID
    AddKnown {
        /// Source ID (e.g., lwn, ars, nytimes)
        id: String,
    },
}

#[derive(Subcommand)]
enum InterestCommands {
    /// Show current interests
    Show,
    /// Add a topic interest
    AddTopic {
        /// Topic name
        topic: String,
    },
    /// Remove a topic interest
    RemoveTopic {
        /// Topic name
        topic: String,
    },
    /// Add a keyword to boost
    AddKeyword {
        /// Keyword
        keyword: String,
    },
    /// Add a keyword to block
    Block {
        /// Keyword to block
        keyword: String,
    },
    /// Clear all interests
    Clear,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set articles per source in digest
    ArticlesPerSource {
        /// Number of articles
        count: usize,
    },
    /// Set number of Wikipedia articles in digest
    WikipediaArticles {
        /// Number of articles
        count: usize,
    },
    /// Set fetch interval in hours
    FetchInterval {
        /// Hours between fetches
        hours: u32,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let storage = Storage::open(None)?;

    match cli.command {
        Commands::Source { command } => handle_source(command, &storage).await,
        Commands::Interest { command } => handle_interest(command, &storage),
        Commands::Fetch { source } => handle_fetch(source, &storage).await,
        Commands::Digest { format, output } => handle_digest(format, output, &storage).await,
        Commands::List { limit } => handle_list(limit, &storage),
        Commands::Read { article_id } => handle_read(article_id, &storage),
        Commands::Config { command } => handle_config(command, &storage),
        Commands::Discover { url, add } => handle_discover(url, add, &storage).await,
        Commands::Track { url, name } => handle_track(url, name, &storage).await,
        Commands::LlmStatus => handle_llm_status().await,
        Commands::Score { all } => handle_score(all, &storage).await,
        Commands::Summarize { article_id } => handle_summarize(article_id, &storage).await,
        Commands::Analyze => handle_analyze(&storage).await,
    }
}

async fn handle_source(command: SourceCommands, storage: &Storage) -> Result<()> {
    match command {
        SourceCommands::List => {
            let sources = storage.get_sources()?;
            if sources.is_empty() {
                println!("No sources configured. Use 'sovereign-reader source add' or 'sovereign-reader source available' to get started.");
                return Ok(());
            }

            println!("{}", "Configured Sources:".cyan().bold());
            println!("{}", "=".repeat(60));

            for source in sources {
                let status = if source.enabled {
                    "enabled".green()
                } else {
                    "disabled".yellow()
                };

                println!(
                    "\n{} ({})",
                    source.name.bold(),
                    source.id.dimmed()
                );
                println!("  Status: {}", status);
                println!("  URL: {}", source.url);
                if !source.sections.is_empty() {
                    println!("  Sections: {}", source.sections.join(", "));
                }
                if let Some(last) = source.last_fetched {
                    println!("  Last fetched: {}", last.format("%Y-%m-%d %H:%M"));
                }
            }
            Ok(())
        }

        SourceCommands::Add { name, url, id } => {
            let id = id.unwrap_or_else(|| {
                name.to_lowercase()
                    .replace(' ', "-")
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-')
                    .collect()
            });

            let source = Source {
                id: id.clone(),
                name,
                source_type: SourceType::Feed,
                url,
                sections: vec![],
                enabled: true,
                last_fetched: None,
            };

            storage.save_source(&source)?;
            println!("{} Added source '{}' (enabled)", "OK".green().bold(), id);
            Ok(())
        }

        SourceCommands::Enable { id } => {
            let sources = storage.get_sources()?;
            if let Some(mut source) = sources.into_iter().find(|s| s.id == id) {
                source.enabled = true;
                storage.save_source(&source)?;
                println!("{} Enabled source '{}'", "OK".green().bold(), id);
            } else {
                println!("{} Source '{}' not found", "Error:".red().bold(), id);
            }
            Ok(())
        }

        SourceCommands::Disable { id } => {
            let sources = storage.get_sources()?;
            if let Some(mut source) = sources.into_iter().find(|s| s.id == id) {
                source.enabled = false;
                storage.save_source(&source)?;
                println!("{} Disabled source '{}'", "OK".green().bold(), id);
            } else {
                println!("{} Source '{}' not found", "Error:".red().bold(), id);
            }
            Ok(())
        }

        SourceCommands::Remove { id } => {
            storage.delete_source(&id)?;
            println!("{} Removed source '{}'", "OK".green().bold(), id);
            Ok(())
        }

        SourceCommands::Available => {
            println!("{}", "Available Pre-configured Sources:".cyan().bold());
            println!("{}", "=".repeat(60));

            for source in sources::get_known_sources() {
                println!("\n{} ({})", source.name.bold(), source.id.dimmed());
                println!("  {}", source.url);
            }

            println!("\nUse 'sovereign-reader source add-known <id>' to add a source.");
            Ok(())
        }

        SourceCommands::AddKnown { id } => {
            let known = sources::get_known_sources();
            if let Some(mut source) = known.into_iter().find(|s| s.id == id) {
                source.enabled = true;
                storage.save_source(&source)?;
                println!(
                    "{} Added '{}' ({})",
                    "OK".green().bold(),
                    source.name,
                    source.id
                );
            } else {
                println!(
                    "{} Unknown source '{}'. Use 'sovereign-reader source available' to see options.",
                    "Error:".red().bold(),
                    id
                );
            }
            Ok(())
        }
    }
}

fn handle_interest(command: InterestCommands, storage: &Storage) -> Result<()> {
    let mut interests = storage.get_interests()?;

    match command {
        InterestCommands::Show => {
            println!("{}", "Your Interests:".cyan().bold());
            println!("{}", "=".repeat(40));

            if interests.topics.is_empty()
                && interests.keywords.is_empty()
                && interests.blocked_keywords.is_empty()
            {
                println!("No interests configured yet.");
                println!("\nUse these commands to set up your interests:");
                println!("  sovereign-reader interest add-topic <topic>");
                println!("  sovereign-reader interest add-keyword <keyword>");
                println!("  sovereign-reader interest block <keyword>");
                return Ok(());
            }

            if !interests.topics.is_empty() {
                println!("\n{}", "Topics:".bold());
                for topic in &interests.topics {
                    println!("  - {}", topic);
                }
            }

            if !interests.keywords.is_empty() {
                println!("\n{}", "Keywords (boost):".bold());
                for kw in &interests.keywords {
                    println!("  + {}", kw.green());
                }
            }

            if !interests.blocked_keywords.is_empty() {
                println!("\n{}", "Blocked:".bold());
                for kw in &interests.blocked_keywords {
                    println!("  - {}", kw.red());
                }
            }
            Ok(())
        }

        InterestCommands::AddTopic { topic } => {
            if !interests.topics.contains(&topic) {
                interests.topics.push(topic.clone());
                storage.save_interests(&interests)?;
            }
            println!("{} Added topic '{}'", "OK".green().bold(), topic);
            Ok(())
        }

        InterestCommands::RemoveTopic { topic } => {
            interests.topics.retain(|t| t != &topic);
            storage.save_interests(&interests)?;
            println!("{} Removed topic '{}'", "OK".green().bold(), topic);
            Ok(())
        }

        InterestCommands::AddKeyword { keyword } => {
            if !interests.keywords.contains(&keyword) {
                interests.keywords.push(keyword.clone());
                storage.save_interests(&interests)?;
            }
            println!("{} Added keyword '{}'", "OK".green().bold(), keyword);
            Ok(())
        }

        InterestCommands::Block { keyword } => {
            if !interests.blocked_keywords.contains(&keyword) {
                interests.blocked_keywords.push(keyword.clone());
                storage.save_interests(&interests)?;
            }
            println!("{} Blocked keyword '{}'", "OK".green().bold(), keyword);
            Ok(())
        }

        InterestCommands::Clear => {
            storage.save_interests(&Interests::default())?;
            println!("{} Cleared all interests", "OK".green().bold());
            Ok(())
        }
    }
}

async fn handle_fetch(source_filter: Option<String>, storage: &Storage) -> Result<()> {
    let sources = storage.get_enabled_sources()?;

    if sources.is_empty() {
        println!("No enabled sources. Use 'sovereign-reader source add-known <id>' to add some.");
        return Ok(());
    }

    let sources_to_fetch: Vec<_> = if let Some(ref filter) = source_filter {
        sources.into_iter().filter(|s| s.id == *filter).collect()
    } else {
        sources
    };

    if sources_to_fetch.is_empty() {
        println!("No matching sources found.");
        return Ok(());
    }

    println!("{}", "Fetching articles...".cyan().bold());

    let mut total_articles = 0;
    let mut total_errors = 0;

    for source in sources_to_fetch {
        print!("  {} ... ", source.name);

        let result = sources::fetch_source(&source).await?;

        if result.errors.is_empty() {
            println!(
                "{} ({} articles)",
                "OK".green(),
                result.articles.len()
            );

            for article in result.articles {
                storage.save_article(&article)?;
                total_articles += 1;
            }
        } else {
            println!("{} ({})", "WARN".yellow(), result.errors.join(", "));
            total_errors += result.errors.len();
        }
    }

    // Also fetch Wikipedia content
    println!("  Wikipedia ... ");
    let wiki = WikipediaClient::new();
    let interests = storage.get_interests()?;
    let read_ids = storage.get_read_article_ids()?;

    // Get featured article
    if let Ok(Some(featured)) = wiki.get_featured_article().await {
        storage.save_article(&featured)?;
        total_articles += 1;
    }

    // Get On This Day
    if let Ok(otd_articles) = wiki.get_on_this_day().await {
        for article in otd_articles {
            storage.save_article(&article)?;
            total_articles += 1;
        }
    }

    // Get articles based on interests
    let config = storage.get_reader_config()?;
    if let Ok(interest_articles) = wiki
        .search_by_interests(&interests, config.wikipedia_articles, &read_ids)
        .await
    {
        for article in interest_articles {
            storage.save_article(&article)?;
            total_articles += 1;
        }
    }

    println!("{}", "OK".green());

    println!("\n{}", "=".repeat(40));
    println!(
        "Fetched {} articles ({} errors)",
        total_articles.to_string().green(),
        total_errors.to_string().yellow()
    );

    Ok(())
}

async fn handle_digest(
    format: String,
    output: Option<String>,
    storage: &Storage,
) -> Result<()> {
    let config = storage.get_reader_config()?;

    // Get recent unread articles
    let articles = storage.get_recent_articles(config.max_article_age_days, 100)?;

    if articles.is_empty() {
        println!("No articles available for digest. Run 'sovereign-reader fetch' first.");
        return Ok(());
    }

    let digest = digest::generate_digest(articles, &config);

    let format = match format.to_lowercase().as_str() {
        "markdown" | "md" => DigestFormat::Markdown,
        "html" => DigestFormat::Html,
        "plain" | "text" | "txt" => DigestFormat::Plain,
        "json" => DigestFormat::Json,
        _ => {
            println!("Unknown format '{}', using markdown", format);
            DigestFormat::Markdown
        }
    };

    let rendered = digest::render_digest(&digest, &format)?;

    if let Some(path) = output {
        std::fs::write(&path, &rendered)?;
        println!(
            "{} Digest written to {}",
            "OK".green().bold(),
            path
        );
    } else {
        println!("{}", rendered);
    }

    Ok(())
}

fn handle_list(limit: usize, storage: &Storage) -> Result<()> {
    let articles = storage.get_unread_articles(limit)?;

    if articles.is_empty() {
        println!("No unread articles. Run 'sovereign-reader fetch' to get new content.");
        return Ok(());
    }

    println!("{}", "Unread Articles:".cyan().bold());
    println!("{}", "=".repeat(60));

    for article in articles {
        println!("\n{}", article.title.bold());
        println!("  ID: {}", article.id.dimmed());
        println!("  Source: {}", article.source_id);
        println!("  URL: {}", article.url);

        if let Some(summary) = &article.summary {
            let truncated: String = summary.chars().take(100).collect();
            print!("  {}", truncated.dimmed());
            if summary.len() > 100 {
                print!("...");
            }
            println!();
        }
    }

    println!("\nUse 'sovereign-reader read <id>' to mark an article as read.");
    Ok(())
}

fn handle_read(article_id: String, storage: &Storage) -> Result<()> {
    storage.mark_read(&article_id)?;
    println!("{} Marked article as read", "OK".green().bold());
    Ok(())
}

fn handle_config(command: ConfigCommands, storage: &Storage) -> Result<()> {
    let mut config = storage.get_reader_config()?;

    match command {
        ConfigCommands::Show => {
            println!("{}", "Reader Configuration:".cyan().bold());
            println!("{}", "=".repeat(40));
            println!("Articles per source: {}", config.articles_per_source);
            println!("Wikipedia articles: {}", config.wikipedia_articles);
            println!("Fetch interval: {} hours", config.fetch_interval_hours);
            println!("Max article age: {} days", config.max_article_age_days);
            println!("Digest format: {:?}", config.digest_format);
            Ok(())
        }

        ConfigCommands::ArticlesPerSource { count } => {
            config.articles_per_source = count;
            storage.save_reader_config(&config)?;
            println!("{} Set articles per source to {}", "OK".green().bold(), count);
            Ok(())
        }

        ConfigCommands::WikipediaArticles { count } => {
            config.wikipedia_articles = count;
            storage.save_reader_config(&config)?;
            println!(
                "{} Set Wikipedia articles to {}",
                "OK".green().bold(),
                count
            );
            Ok(())
        }

        ConfigCommands::FetchInterval { hours } => {
            config.fetch_interval_hours = hours;
            storage.save_reader_config(&config)?;
            println!(
                "{} Set fetch interval to {} hours",
                "OK".green().bold(),
                hours
            );
            Ok(())
        }
    }
}

async fn handle_discover(url: String, add: bool, storage: &Storage) -> Result<()> {
    println!("{}", "Discovering feeds...".cyan().bold());
    println!("URL: {}\n", url);

    // Analyze the site
    let analysis = sources::analyze_site(&url).await?;

    if analysis.feeds.is_empty() {
        println!("{}", "No RSS/Atom feeds found.".yellow());

        if analysis.supports_scraping {
            println!(
                "\nThis site can be tracked for changes using:\n  {}",
                format!("sovereign-reader track \"{}\" \"Site Name\"", url).dimmed()
            );

            if let Some(selector) = analysis.recommended_selector {
                println!("  Recommended CSS selector: {}", selector);
            }
        }

        return Ok(());
    }

    println!("{}", "Discovered feeds:".green().bold());
    println!("{}", "=".repeat(60));

    for (i, feed) in analysis.feeds.iter().enumerate() {
        let feed_type = match feed.feed_type {
            sources::FeedType::Rss => "RSS",
            sources::FeedType::Atom => "Atom",
            sources::FeedType::Json => "JSON",
            sources::FeedType::Unknown => "Feed",
        };

        println!("\n{}. {}", i + 1, feed.url);
        if let Some(title) = &feed.title {
            println!("   Title: {}", title);
        }
        println!("   Type: {}", feed_type);
    }

    if add {
        println!("\n{}", "Adding discovered feeds...".cyan());

        for feed in &analysis.feeds {
            let id = feed
                .title
                .as_ref()
                .map(|t| {
                    t.to_lowercase()
                        .replace(' ', "-")
                        .chars()
                        .filter(|c| c.is_alphanumeric() || *c == '-')
                        .collect()
                })
                .unwrap_or_else(|| format!("feed-{}", uuid_simple()));

            let name = feed
                .title
                .clone()
                .unwrap_or_else(|| format!("Feed from {}", url));

            let source = Source {
                id: id.clone(),
                name,
                source_type: SourceType::Feed,
                url: feed.url.clone(),
                sections: vec![],
                enabled: true,
                last_fetched: None,
            };

            storage.save_source(&source)?;
            println!("  {} Added '{}'", "OK".green(), id);
        }
    } else {
        println!(
            "\nUse {} to automatically add these feeds.",
            "sovereign-reader discover <url> --add".dimmed()
        );
    }

    Ok(())
}

async fn handle_track(url: String, name: String, storage: &Storage) -> Result<()> {
    println!("{}", "Setting up site tracking...".cyan().bold());

    // Analyze the site first
    let analysis = sources::analyze_site(&url).await?;

    // Warn if site has feeds
    if !analysis.feeds.is_empty() {
        println!(
            "{} This site has RSS feeds available. Consider using those instead:",
            "Note:".yellow().bold()
        );
        for feed in &analysis.feeds {
            println!("  - {}", feed.url);
        }
        println!();
    }

    let id = name
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let source = Source {
        id: id.clone(),
        name,
        source_type: SourceType::Track,
        url,
        sections: vec![],
        enabled: true,
        last_fetched: None,
    };

    storage.save_source(&source)?;

    println!("{} Now tracking '{}' for changes", "OK".green().bold(), id);

    if let Some(selector) = analysis.recommended_selector {
        println!("  Using selector: {}", selector);
    }

    println!(
        "\nRun {} to fetch articles.",
        "sovereign-reader fetch".dimmed()
    );

    Ok(())
}

/// Generate a simple unique ID
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", time)
}

async fn handle_llm_status() -> Result<()> {
    println!("{}", "LLM Integration Status".cyan().bold());
    println!("{}", "=".repeat(60));

    let available = llm::is_llm_available();
    let ipc_dir = llm::get_ipc_dir();

    if available {
        println!("\n{} LLM assistant environment detected", "OK".green().bold());
    } else {
        println!("\n{} LLM assistant not detected", "INFO".yellow().bold());
    }

    println!("\nIPC Directory: {}", ipc_dir.display());

    if ipc_dir.exists() {
        println!("Status: {}", "Directory exists".green());
    } else {
        println!("Status: {}", "Directory will be created on first use".dimmed());
    }

    println!("\n{}", "How LLM integration works:".bold());
    println!("1. sovereign-reader writes requests to the IPC directory");
    println!("2. Your LLM assistant monitors for 'pending' file");
    println!("3. Assistant processes request-*.json and writes response-*.json");
    println!("4. sovereign-reader reads response and continues");

    println!("\n{}", "To enable, set environment variable:".dimmed());
    println!("  export CLAUDE_CODE=1");

    println!("\n{}", "Or have your LLM assistant monitor:".dimmed());
    println!("  {}", ipc_dir.display());

    // Show assistant instructions
    println!("\n{}", "Assistant Instructions:".cyan().bold());
    println!("{}", llm::get_assistant_instructions());

    Ok(())
}

async fn handle_score(all: bool, storage: &Storage) -> Result<()> {
    if !llm::is_llm_available() {
        println!(
            "{} LLM assistant not detected. Run 'sovereign-reader llm-status' for setup info.",
            "Error:".red().bold()
        );
        return Ok(());
    }

    println!("{}", "Scoring articles with LLM...".cyan().bold());

    let articles = if all {
        storage.get_unread_articles(100)?
    } else {
        // Get recently fetched, unscored articles
        storage.get_unread_articles(20)?
    };

    if articles.is_empty() {
        println!("No articles to score.");
        return Ok(());
    }

    // Convert to scoring format
    let articles_for_scoring: Vec<llm::ArticleForScoring> = articles
        .iter()
        .map(|a| llm::ArticleForScoring {
            id: a.id.clone(),
            title: a.title.clone(),
            summary: a.summary.clone(),
            source: a.source_id.clone(),
            categories: a.categories.clone(),
        })
        .collect();

    // Get user interests
    let interests = storage.get_interests()?;
    let user_interests = llm::UserInterests {
        topics: interests.topics,
        keywords: interests.keywords,
        blocked_keywords: interests.blocked_keywords,
    };

    // Create LLM client and score
    let client = llm::LlmClient::new()?;

    println!(
        "Sending {} articles to LLM for scoring...",
        articles_for_scoring.len()
    );
    println!("Waiting for response from: {}\n", client.ipc_dir().display());

    match client.score_articles(articles_for_scoring, user_interests).await {
        Ok(scores) => {
            println!("{}", "Scores received:".green().bold());
            println!("{}", "=".repeat(60));

            for score in &scores {
                // Update article in storage
                // For now, just display the scores
                let score_color = if score.score >= 70 {
                    score.score.to_string().green()
                } else if score.score >= 40 {
                    score.score.to_string().yellow()
                } else {
                    score.score.to_string().dimmed()
                };

                // Find article title
                let title = articles
                    .iter()
                    .find(|a| a.id == score.id)
                    .map(|a| a.title.as_str())
                    .unwrap_or("Unknown");

                println!("\n[{}] {}", score_color, title);
                println!("     {}", score.reason.dimmed());
            }

            println!("\n{}", "=".repeat(60));
            println!("Scored {} articles", scores.len());
        }
        Err(e) => {
            println!("{} {}", "Error:".red().bold(), e);
            println!(
                "\nMake sure your LLM assistant is monitoring: {}",
                client.ipc_dir().display()
            );
        }
    }

    Ok(())
}

async fn handle_summarize(article_id: String, storage: &Storage) -> Result<()> {
    if !llm::is_llm_available() {
        println!(
            "{} LLM assistant not detected. Run 'sovereign-reader llm-status' for setup info.",
            "Error:".red().bold()
        );
        return Ok(());
    }

    // Get the article
    let articles = storage.get_unread_articles(1000)?;
    let article = articles.iter().find(|a| a.id == article_id);

    let article = match article {
        Some(a) => a,
        None => {
            println!("{} Article '{}' not found", "Error:".red().bold(), article_id);
            return Ok(());
        }
    };

    println!("{}", "Summarizing article with LLM...".cyan().bold());
    println!("Title: {}\n", article.title);

    let content = article
        .content
        .as_ref()
        .or(article.summary.as_ref())
        .cloned()
        .unwrap_or_else(|| "No content available".to_string());

    let client = llm::LlmClient::new()?;

    println!("Waiting for LLM response...\n");

    match client.summarize(&article.title, &content, &article.url).await {
        Ok((summary, key_points)) => {
            println!("{}", "Summary:".green().bold());
            println!("{}\n", summary);

            if !key_points.is_empty() {
                println!("{}", "Key Points:".green().bold());
                for point in key_points {
                    println!("  - {}", point);
                }
            }

            println!("\n{}", "Original URL:".dimmed());
            println!("  {}", article.url);
        }
        Err(e) => {
            println!("{} {}", "Error:".red().bold(), e);
        }
    }

    Ok(())
}

async fn handle_analyze(storage: &Storage) -> Result<()> {
    if !llm::is_llm_available() {
        println!(
            "{} LLM assistant not detected. Run 'sovereign-reader llm-status' for setup info.",
            "Error:".red().bold()
        );
        return Ok(());
    }

    println!("{}", "Analyzing reading patterns with LLM...".cyan().bold());

    // Get read articles for analysis
    let read_ids = storage.get_read_article_ids()?;

    if read_ids.len() < 5 {
        println!(
            "{} Need at least 5 read articles for pattern analysis. Currently have: {}",
            "Info:".yellow().bold(),
            read_ids.len()
        );
        return Ok(());
    }

    // We need to get the actual article data for read articles
    // For now, use unread articles as a proxy (in real impl, would store read articles)
    let articles = storage.get_unread_articles(50)?;

    let articles_for_analysis: Vec<llm::ArticleForScoring> = articles
        .iter()
        .take(20)
        .map(|a| llm::ArticleForScoring {
            id: a.id.clone(),
            title: a.title.clone(),
            summary: a.summary.clone(),
            source: a.source_id.clone(),
            categories: a.categories.clone(),
        })
        .collect();

    let interests = storage.get_interests()?;
    let current_interests = llm::UserInterests {
        topics: interests.topics.clone(),
        keywords: interests.keywords.clone(),
        blocked_keywords: interests.blocked_keywords.clone(),
    };

    let client = llm::LlmClient::new()?;

    println!("Analyzing {} articles...\n", articles_for_analysis.len());

    match client
        .analyze_patterns(articles_for_analysis, current_interests)
        .await
    {
        Ok((topics, keywords, reasoning)) => {
            println!("{}", "Analysis Complete".green().bold());
            println!("{}", "=".repeat(60));

            println!("\n{}", "Suggested Topics:".bold());
            for topic in &topics {
                println!("  + {}", topic.green());
            }

            println!("\n{}", "Suggested Keywords:".bold());
            for keyword in &keywords {
                println!("  + {}", keyword.cyan());
            }

            println!("\n{}", "Reasoning:".bold());
            println!("{}", reasoning.dimmed());

            println!("\n{}", "To add these suggestions:".dimmed());
            for topic in &topics {
                println!("  sovereign-reader interest add-topic \"{}\"", topic);
            }
        }
        Err(e) => {
            println!("{} {}", "Error:".red().bold(), e);
        }
    }

    Ok(())
}
