//! Daily digest generation

use crate::{Article, DailyDigest, DigestFormat, DigestSection, ReaderConfig};
use anyhow::Result;
use chrono::Utc;
use minijinja::{context, Environment};
use std::collections::HashMap;

/// Generate a daily digest from articles
pub fn generate_digest(
    articles: Vec<Article>,
    config: &ReaderConfig,
) -> DailyDigest {
    // Group articles by source/category
    let mut sections_map: HashMap<String, Vec<Article>> = HashMap::new();

    for article in articles {
        // Determine section name
        let section_name = if article.source_id == "wikipedia" {
            "Wikipedia Discoveries".to_string()
        } else if !article.categories.is_empty() {
            article.categories[0].clone()
        } else {
            // Use source as section
            article.source_id.clone()
        };

        sections_map
            .entry(section_name)
            .or_default()
            .push(article);
    }

    // Sort sections and limit articles per section
    let mut sections: Vec<DigestSection> = sections_map
        .into_iter()
        .map(|(name, mut articles)| {
            // Sort by interest score, then by date
            articles.sort_by(|a, b| {
                b.interest_score
                    .unwrap_or(0)
                    .cmp(&a.interest_score.unwrap_or(0))
                    .then_with(|| b.published.cmp(&a.published))
            });

            // Limit articles
            articles.truncate(config.articles_per_source);

            DigestSection { name, articles }
        })
        .collect();

    // Sort sections (Wikipedia at the end, others alphabetically)
    sections.sort_by(|a, b| {
        let a_is_wiki = a.name == "Wikipedia Discoveries";
        let b_is_wiki = b.name == "Wikipedia Discoveries";

        if a_is_wiki && !b_is_wiki {
            std::cmp::Ordering::Greater
        } else if !a_is_wiki && b_is_wiki {
            std::cmp::Ordering::Less
        } else {
            a.name.cmp(&b.name)
        }
    });

    let total_articles = sections.iter().map(|s| s.articles.len()).sum();

    DailyDigest {
        generated_at: Utc::now(),
        date: Utc::now().date_naive(),
        sections,
        total_articles,
    }
}

/// Render a digest to the specified format
pub fn render_digest(digest: &DailyDigest, format: &DigestFormat) -> Result<String> {
    match format {
        DigestFormat::Markdown => render_markdown(digest),
        DigestFormat::Html => render_html(digest),
        DigestFormat::Plain => render_plain(digest),
        DigestFormat::Json => render_json(digest),
    }
}

fn render_markdown(digest: &DailyDigest) -> Result<String> {
    let mut env = Environment::new();
    env.add_template(
        "digest",
        r#"# Daily Digest - {{ date }}

*Generated at {{ generated_at }} | {{ total_articles }} articles*

---

{% for section in sections %}
## {{ section.name }}

{% for article in section.articles %}
### [{{ article.title }}]({{ article.url }})

{% if article.summary %}{{ article.summary }}{% endif %}

{% if article.authors %}*By {{ article.authors | join(", ") }}*{% endif %}
{% if article.published %}*Published: {{ article.published }}*{% endif %}

---

{% endfor %}
{% endfor %}
"#,
    )?;

    let tmpl = env.get_template("digest")?;
    let output = tmpl.render(context! {
        date => digest.date.to_string(),
        generated_at => digest.generated_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        total_articles => digest.total_articles,
        sections => digest.sections.iter().map(|s| {
            context! {
                name => &s.name,
                articles => s.articles.iter().map(|a| {
                    context! {
                        title => &a.title,
                        url => &a.url,
                        summary => a.summary.as_deref().unwrap_or(""),
                        authors => &a.authors,
                        published => a.published.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
                    }
                }).collect::<Vec<_>>(),
            }
        }).collect::<Vec<_>>(),
    })?;

    Ok(output)
}

fn render_html(digest: &DailyDigest) -> Result<String> {
    let mut env = Environment::new();
    env.add_template(
        "digest",
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Daily Digest - {{ date }}</title>
    <style>
        body { font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }
        h1 { border-bottom: 2px solid #333; padding-bottom: 10px; }
        h2 { color: #444; margin-top: 30px; }
        .article { margin-bottom: 20px; padding-bottom: 20px; border-bottom: 1px solid #eee; }
        .article h3 { margin-bottom: 5px; }
        .article h3 a { color: #1a0dab; text-decoration: none; }
        .article h3 a:hover { text-decoration: underline; }
        .summary { color: #545454; line-height: 1.6; }
        .meta { font-size: 0.9em; color: #777; margin-top: 5px; }
        .stats { color: #666; font-style: italic; }
    </style>
</head>
<body>
    <h1>Daily Digest - {{ date }}</h1>
    <p class="stats">Generated at {{ generated_at }} | {{ total_articles }} articles</p>

    {% for section in sections %}
    <h2>{{ section.name }}</h2>
    {% for article in section.articles %}
    <div class="article">
        <h3><a href="{{ article.url }}">{{ article.title }}</a></h3>
        {% if article.summary %}<p class="summary">{{ article.summary }}</p>{% endif %}
        <p class="meta">
            {% if article.authors %}By {{ article.authors | join(", ") }} | {% endif %}
            {% if article.published %}{{ article.published }}{% endif %}
        </p>
    </div>
    {% endfor %}
    {% endfor %}
</body>
</html>
"#,
    )?;

    let tmpl = env.get_template("digest")?;
    let output = tmpl.render(context! {
        date => digest.date.to_string(),
        generated_at => digest.generated_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        total_articles => digest.total_articles,
        sections => digest.sections.iter().map(|s| {
            context! {
                name => &s.name,
                articles => s.articles.iter().map(|a| {
                    context! {
                        title => &a.title,
                        url => &a.url,
                        summary => a.summary.as_deref().unwrap_or(""),
                        authors => &a.authors,
                        published => a.published.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
                    }
                }).collect::<Vec<_>>(),
            }
        }).collect::<Vec<_>>(),
    })?;

    Ok(output)
}

fn render_plain(digest: &DailyDigest) -> Result<String> {
    let mut output = String::new();

    output.push_str(&format!("DAILY DIGEST - {}\n", digest.date));
    output.push_str(&format!(
        "Generated at {} | {} articles\n",
        digest.generated_at.format("%Y-%m-%d %H:%M UTC"),
        digest.total_articles
    ));
    output.push_str(&"=".repeat(60));
    output.push('\n');

    for section in &digest.sections {
        output.push_str(&format!("\n{}\n", section.name.to_uppercase()));
        output.push_str(&"-".repeat(40));
        output.push('\n');

        for article in &section.articles {
            output.push_str(&format!("\n* {}\n", article.title));
            output.push_str(&format!("  {}\n", article.url));

            if let Some(summary) = &article.summary {
                // Truncate summary for plain text
                let truncated: String = summary.chars().take(200).collect();
                output.push_str(&format!("  {}", truncated));
                if summary.len() > 200 {
                    output.push_str("...");
                }
                output.push('\n');
            }

            if !article.authors.is_empty() {
                output.push_str(&format!("  By: {}\n", article.authors.join(", ")));
            }
        }
    }

    Ok(output)
}

fn render_json(digest: &DailyDigest) -> Result<String> {
    Ok(serde_json::to_string_pretty(digest)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_generate_empty_digest() {
        let config = ReaderConfig::default();
        let digest = generate_digest(vec![], &config);
        assert_eq!(digest.total_articles, 0);
        assert!(digest.sections.is_empty());
    }

    #[test]
    fn test_render_plain() {
        let digest = DailyDigest {
            generated_at: Utc::now(),
            date: Utc::now().date_naive(),
            sections: vec![DigestSection {
                name: "Test".to_string(),
                articles: vec![Article {
                    id: "1".to_string(),
                    source_id: "test".to_string(),
                    title: "Test Article".to_string(),
                    url: "https://example.com".to_string(),
                    summary: Some("Summary".to_string()),
                    content: None,
                    authors: vec!["Author".to_string()],
                    published: Some(Utc::now()),
                    fetched_at: Utc::now(),
                    categories: vec![],
                    read: false,
                    interest_score: None,
                }],
            }],
            total_articles: 1,
        };

        let output = render_plain(&digest).unwrap();
        assert!(output.contains("DAILY DIGEST"));
        assert!(output.contains("Test Article"));
    }
}
