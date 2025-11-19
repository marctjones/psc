use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process;

mod commands;
mod ui;

use commands::{identity, config, bsky, fedi, server, test, auth};

#[derive(Parser)]
#[command(name = "sovereign")]
#[command(about = "Self-hosted ActivityPub and Bluesky server", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage your identity and domain
    Identity {
        #[command(subcommand)]
        command: IdentityCommands,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Manage the Cloudflare Worker server
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },
    /// Bluesky / AT Protocol commands
    Bsky {
        #[command(subcommand)]
        command: BskyCommands,
    },
    /// ActivityPub / Fediverse commands
    Fedi {
        #[command(subcommand)]
        command: FediCommands,
    },
    /// Test and validate deployments
    Test {
        #[command(subcommand)]
        command: TestCommands,
    },
    /// Authenticate with your Sovereign server
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
}

#[derive(Subcommand)]
enum IdentityCommands {
    /// Set up your identity with a domain
    Setup {
        /// Your domain name
        domain: String,
    },
    /// Clean up DNS records and remove identity
    Cleanup,
    /// Export your identity for backup
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Import identity from backup
    Import {
        /// Backup file path
        file: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// List all configuration
    List,
}

#[derive(Subcommand)]
enum ServerCommands {
    /// Deploy the worker to Cloudflare
    Deploy,
    /// Check server status
    Status,
    /// View recent logs
    Logs {
        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },
}

#[derive(Subcommand)]
enum BskyCommands {
    /// Log in to Bluesky
    Login {
        /// Your handle (e.g., user.bsky.social)
        handle: String,
        /// Use app password
        #[arg(long)]
        app_password: bool,
    },
    /// Log out
    Logout,
    /// Show current user
    Whoami,
    /// Create a post
    Post {
        /// Post text
        text: String,
        /// Image to attach
        #[arg(long)]
        image: Option<String>,
        /// Alt text for image
        #[arg(long)]
        alt: Option<String>,
    },
    /// View timeline
    Timeline {
        /// Number of posts to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Follow a user
    Follow {
        /// Handle to follow
        handle: String,
    },
    /// Unfollow a user
    Unfollow {
        /// Handle to unfollow
        handle: String,
    },
    /// Like a post
    Like {
        /// Post URI
        uri: String,
    },
    /// View profile
    Profile {
        /// Handle (defaults to self)
        handle: Option<String>,
    },
}

#[derive(Subcommand)]
enum FediCommands {
    /// Log in to your server
    Login,
    /// Log out
    Logout,
    /// Show current user
    Whoami,
    /// Create a post
    Post {
        /// Post text
        text: String,
        /// Content warning
        #[arg(long)]
        cw: Option<String>,
        /// Image to attach
        #[arg(long)]
        image: Option<String>,
    },
    /// View timeline
    Timeline {
        /// Number of posts to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Follow a user
    Follow {
        /// User to follow (user@instance)
        user: String,
    },
    /// Unfollow a user
    Unfollow {
        /// User to unfollow (user@instance)
        user: String,
    },
    /// Favorite a post
    Favorite {
        /// Post URL
        url: String,
    },
    /// Boost a post
    Boost {
        /// Post URL
        url: String,
    },
    /// View profile
    Profile {
        /// User (defaults to self)
        user: Option<String>,
    },
}

#[derive(Subcommand)]
enum TestCommands {
    /// Run full deployment validation tests
    Validate {
        /// Server URL (e.g., https://yourdomain.com or http://localhost:8787)
        url: String,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Quick connectivity test
    Quick {
        /// Server URL
        url: String,
    },
    /// Test connectivity to a remote Mastodon/ActivityPub instance
    RemoteFedi {
        /// Instance domain (e.g., mastodon.social)
        instance: String,
    },
    /// Test connectivity to Bluesky network
    RemoteBsky,
    /// Test connectivity to a specific remote user (read-only, no data sent)
    RemoteUser {
        /// User to test (e.g., gargron@mastodon.social)
        user: String,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Log in to your Sovereign server using device authorization
    Login {
        /// Server URL (e.g., https://yourdomain.com)
        server: String,
        /// Requested scopes (default: read write)
        #[arg(short, long, default_value = "read write")]
        scope: String,
    },
    /// Log out and revoke tokens
    Logout,
    /// Show current authentication status
    Status,
    /// Refresh access token
    Refresh,
    /// Register a new OAuth client (for third-party apps)
    RegisterClient {
        /// Client name
        name: String,
        /// Redirect URIs (comma-separated)
        #[arg(long)]
        redirect_uris: Option<String>,
        /// Scopes (default: read)
        #[arg(long, default_value = "read")]
        scope: String,
    },
    /// List registered OAuth clients
    ListClients {
        /// Server URL
        server: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Identity { command } => match command {
            IdentityCommands::Setup { domain } => identity::setup(&domain).await,
            IdentityCommands::Cleanup => identity::cleanup().await,
            IdentityCommands::Export { output } => identity::export(output.as_deref()).await,
            IdentityCommands::Import { file } => identity::import(&file).await,
        },
        Commands::Config { command } => match command {
            ConfigCommands::Get { key } => config::get(&key).await,
            ConfigCommands::Set { key, value } => config::set(&key, &value).await,
            ConfigCommands::List => config::list().await,
        },
        Commands::Server { command } => match command {
            ServerCommands::Deploy => server::deploy().await,
            ServerCommands::Status => server::status().await,
            ServerCommands::Logs { lines } => server::logs(lines).await,
        },
        Commands::Bsky { command } => match command {
            BskyCommands::Login { handle, app_password } => {
                bsky::login(&handle, app_password).await
            }
            BskyCommands::Logout => bsky::logout().await,
            BskyCommands::Whoami => bsky::whoami().await,
            BskyCommands::Post { text, image, alt } => {
                bsky::post(&text, image.as_deref(), alt.as_deref()).await
            }
            BskyCommands::Timeline { limit } => bsky::timeline(limit).await,
            BskyCommands::Follow { handle } => bsky::follow(&handle).await,
            BskyCommands::Unfollow { handle } => bsky::unfollow(&handle).await,
            BskyCommands::Like { uri } => bsky::like(&uri).await,
            BskyCommands::Profile { handle } => bsky::profile(handle.as_deref()).await,
        },
        Commands::Fedi { command } => match command {
            FediCommands::Login => fedi::login().await,
            FediCommands::Logout => fedi::logout().await,
            FediCommands::Whoami => fedi::whoami().await,
            FediCommands::Post { text, cw, image } => {
                fedi::post(&text, cw.as_deref(), image.as_deref()).await
            }
            FediCommands::Timeline { limit } => fedi::timeline(limit).await,
            FediCommands::Follow { user } => fedi::follow(&user).await,
            FediCommands::Unfollow { user } => fedi::unfollow(&user).await,
            FediCommands::Favorite { url } => fedi::favorite(&url).await,
            FediCommands::Boost { url } => fedi::boost(&url).await,
            FediCommands::Profile { user } => fedi::profile(user.as_deref()).await,
        },
        Commands::Test { command } => match command {
            TestCommands::Validate { url, verbose } => test::validate(&url, verbose).await,
            TestCommands::Quick { url } => test::quick(&url).await,
            TestCommands::RemoteFedi { instance } => test::remote_fedi(&instance).await,
            TestCommands::RemoteBsky => test::remote_bsky().await,
            TestCommands::RemoteUser { user } => {
                // Parse user@instance format
                let parts: Vec<&str> = user.split('@').collect();
                if parts.len() != 2 {
                    Err(format!(
                        "Invalid user format '{}'. Use username@instance (e.g., gargron@mastodon.social)",
                        user
                    ).into())
                } else {
                    test::remote_user(parts[0], parts[1]).await
                }
            }
        },
        Commands::Auth { command } => match command {
            AuthCommands::Login { server, scope } => auth::login(&server, &scope).await,
            AuthCommands::Logout => auth::logout().await,
            AuthCommands::Status => auth::status().await,
            AuthCommands::Refresh => auth::refresh().await,
            AuthCommands::RegisterClient { name, redirect_uris, scope } => {
                let uris: Vec<String> = redirect_uris
                    .map(|s| s.split(',').map(|u| u.trim().to_string()).collect())
                    .unwrap_or_default();
                auth::register_client(&name, &uris, &scope).await
            }
            AuthCommands::ListClients { server } => auth::list_clients(&server).await,
        },
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}
