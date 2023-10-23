use csv::Writer;
use fantoccini::{ClientBuilder, Locator};
use scraper::{Html, Selector};
use std::{
    fs::File,
    io::{BufReader, Error, Read, Write},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = get_data().await;

    let formatted_contents: String = format!("{}", &reading_file());
    let html: &str = formatted_contents.as_str();

    let fragment: Html = Html::parse_fragment(html);
    let selector: Selector = Selector::parse("tr").unwrap();
    let mut wtr: Writer<File> = Writer::from_path("output.csv")?;

    for element in fragment.select(&selector) {
        let mut row: Vec<String> = Vec::new();
        for text in element.text() {
            let trimmed_text: &str = text.trim();
            if !trimmed_text.is_empty() {
                row.push(trimmed_text.to_string());
            }
        }
        if row.len() == 13 {
            wtr.write_record(&row)?;
        }
    }

    wtr.flush()?;

    Ok(())
}

async fn get_data() -> Result<(), fantoccini::error::CmdError> {
    let client: fantoccini::Client = ClientBuilder::native()
        .connect("http://127.0.0.1:4444")
        .await
        .expect("failed to connect to WebDriver");

    client
        .goto("https://www.tennisabstract.com/cgi-bin/player-classic.cgi?p=RogerFederer")
        .await
        .expect("failed to go to website");

    let body: fantoccini::elements::Element = client
        .find(Locator::Css("body"))
        .await
        .expect("failed to find body");

    let html: String = body.html(false).await.expect("failed to get HTML");

    let mut file: File = File::create("index.html")?;
    file.write_all(html.as_bytes())?;

    client.close().await
}

fn reading_file() -> String {
    let file: File = File::open("index.html").unwrap();
    let mut buf_reader: BufReader<File> = BufReader::new(file);
    let mut contents: String = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    // println!("{}", contents);
    // Ok(())
    contents
}
