use colored::Colorize;

/// Print a post in a formatted box
pub fn print_post(
    author: &str,
    content: &str,
    likes: u64,
    reposts: u64,
    replies: u64,
    time_ago: &str,
) {
    let width = 50;
    let header = format!(" Post by {} ", author);

    // Top border
    print!("{}", "┌─".cyan());
    print!("{}", header.cyan().bold());
    let remaining = width - header.len() - 1;
    for _ in 0..remaining {
        print!("{}", "─".cyan());
    }
    println!("{}", "┐".cyan());

    // Content lines
    for line in wrap_text(content, width - 4) {
        println!(
            "{} {:<width$} {}",
            "│".cyan(),
            line,
            "│".cyan(),
            width = width - 4
        );
    }

    // Separator
    print!("{}", "├".cyan());
    for _ in 0..width - 2 {
        print!("{}", "─".cyan());
    }
    println!("{}", "┤".cyan());

    // Stats line
    let stats = format!(
        " ♡ {}  ⟳ {}  💬 {}  │  {} ",
        likes, reposts, replies, time_ago
    );
    println!(
        "{} {:<width$} {}",
        "│".cyan(),
        stats,
        "│".cyan(),
        width = width - 4
    );

    // Bottom border
    print!("{}", "└".cyan());
    for _ in 0..width - 2 {
        print!("{}", "─".cyan());
    }
    println!("{}", "┘".cyan());
}

/// Print a notification
pub fn print_notification(
    notification_type: &str,
    actor: &str,
    subject: Option<&str>,
    time_ago: &str,
) {
    let icon = match notification_type {
        "follow" => "👤",
        "like" => "♡",
        "repost" => "⟳",
        "reply" => "💬",
        "mention" => "@",
        _ => "•",
    };

    print!("{} ", icon);
    print!("{} ", actor.cyan());

    match notification_type {
        "follow" => print!("followed you"),
        "like" => print!("liked your post"),
        "repost" => print!("boosted your post"),
        "reply" => print!("replied to your post"),
        "mention" => print!("mentioned you"),
        _ => print!("{}", notification_type),
    }

    println!(" {}", time_ago.dimmed());

    if let Some(subj) = subject {
        println!("    {}", truncate(subj, 60).dimmed());
    }
}

/// Print a profile
pub fn print_profile(
    handle: &str,
    display_name: Option<&str>,
    bio: Option<&str>,
    followers: u64,
    following: u64,
    posts: u64,
) {
    if let Some(name) = display_name {
        println!("{}", name.bold());
    }
    println!("{}", handle.cyan());
    println!();

    if let Some(bio_text) = bio {
        for line in wrap_text(bio_text, 60) {
            println!("{}", line);
        }
        println!();
    }

    println!(
        "{}  {}  {}",
        format!("{} followers", followers).dimmed(),
        format!("{} following", following).dimmed(),
        format!("{} posts", posts).dimmed()
    );
}

/// Wrap text to fit within a given width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Truncate text with ellipsis
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text() {
        let text = "This is a test of the text wrapping function";
        let lines = wrap_text(text, 20);

        assert!(!lines.is_empty());
        for line in &lines {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn test_wrap_empty() {
        let lines = wrap_text("", 20);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
    }
}
