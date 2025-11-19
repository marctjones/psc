//! Sovereign Reader - Personal news aggregator CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use sovereign_reader::{
    digest, sources, storage::Storage, wikipedia::WikipediaClient, DigestFormat, Interests,
    ReaderConfig, Source, SourceType,
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
