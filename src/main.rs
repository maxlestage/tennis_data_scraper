use csv::Writer;
use fantoccini::{Client, ClientBuilder, Locator};
use rayon::prelude::*;
use scraper::{Html, Selector};
use std::{env, fs, path::PathBuf};
use tokio::time::Instant;

/// Address of the running WebDriver (e.g. `geckodriver`/`chromedriver`).
const WEBDRIVER_URL: &str = "http://127.0.0.1:4444";
/// Directory where the per-player CSV files are written.
const OUTPUT_DIR: &str = "./Data";
/// Number of columns a valid statistics row is expected to have.
const EXPECTED_COLUMNS: usize = 13;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let start: Instant = Instant::now();

    let players: Vec<String> = env::args().skip(1).collect();
    if players.is_empty() {
        eprintln!("Usage: tennis_data_scraper <player> [<player> ...]");
        std::process::exit(1);
    }

    // 1. Scrape sequentially over a single WebDriver session (network bound).
    let pages: Vec<(String, String)> = scrape_players(&players).await?;

    // 2. Make sure the output directory exists before writing.
    fs::create_dir_all(OUTPUT_DIR)?;

    // 3. Parse the collected HTML and write each CSV in parallel (CPU bound).
    //    rayon lets us process every player's page on a separate worker thread.
    pages
        .par_iter()
        .map(|(player, html)| (player, write_player_csv(player, html)))
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|(player, result)| match result {
            Ok(rows) => println!("✓ {player}: {rows} rows written"),
            Err(e) => eprintln!("✗ {player}: {e}"),
        });

    println!("Time elapsed: {:?}", start.elapsed());
    Ok(())
}

/// Fetches the raw page HTML for every requested player, reusing a single
/// WebDriver session. A failure on one player is logged and skipped rather
/// than aborting the whole run.
async fn scrape_players(players: &[String]) -> Result<Vec<(String, String)>, BoxError> {
    let client: Client = ClientBuilder::native()
        .connect(WEBDRIVER_URL)
        .await
        .map_err(|e| format!("failed to connect to WebDriver at {WEBDRIVER_URL}: {e}"))?;

    let mut pages: Vec<(String, String)> = Vec::with_capacity(players.len());
    for player in players {
        match fetch_player_html(&client, player).await {
            Ok(html) => pages.push((player.clone(), html)),
            Err(e) => eprintln!("✗ {player}: failed to fetch page ({e})"),
        }
    }

    client.close().await?;
    Ok(pages)
}

/// Navigates to a player's Tennis Abstract page and returns the `<body>` HTML.
async fn fetch_player_html(client: &Client, player: &str) -> Result<String, fantoccini::error::CmdError> {
    let url: String = format!(
        "https://www.tennisabstract.com/cgi-bin/player-classic.cgi?p={player}"
    );

    client.goto(&url).await?;
    let body: fantoccini::elements::Element = client.find(Locator::Css("body")).await?;
    body.html(false).await
}

/// Parses the player's HTML, extracting every statistics table row, and writes
/// them to `Data/<player>.csv`. Returns the number of rows written.
fn write_player_csv(player: &str, html: &str) -> Result<usize, BoxError> {
    let path: PathBuf = PathBuf::from(OUTPUT_DIR).join(format!("{player}.csv"));

    let fragment: Html = Html::parse_fragment(html);
    let row_selector: Selector =
        Selector::parse("tr").expect("\"tr\" is a valid CSS selector");

    let mut writer: Writer<fs::File> = Writer::from_path(&path)?;
    let mut written: usize = 0;

    for element in fragment.select(&row_selector) {
        let row: Vec<&str> = element
            .text()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .collect();

        if row.len() == EXPECTED_COLUMNS {
            writer.write_record(&row)?;
            written += 1;
        }
    }

    writer.flush()?;
    Ok(written)
}
