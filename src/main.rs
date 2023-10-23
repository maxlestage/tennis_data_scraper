use csv::Writer;
use fantoccini::{ClientBuilder, Locator};
use scraper::{Html, Selector};
// use serde_json::json;
use std::{
    fs::{self, File},
    io::{BufReader, Error, Read, Write},
};
use tokio::time::Instant;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let start: Instant = Instant::now();
    let args: Vec<String> = env::args().collect();

    let player1: &String = &args[1];

    let _ = get_data(player1).await;

    let _ = create_data_for_each_players(player1);

    fs::remove_file("index.html")?;
    // txt_to_json();

    let duration: std::time::Duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
    Ok(())
}

async fn get_data(player: &str) -> Result<(), fantoccini::error::CmdError> {
    let url: String = format!(
        "https://www.tennisabstract.com/cgi-bin/player-classic.cgi?p={}",
        player
    );
    let client: fantoccini::Client = ClientBuilder::native()
        .connect("http://127.0.0.1:4444")
        .await
        .expect("failed to connect to WebDriver");

    client
        .goto(url.as_str())
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

fn create_data_for_each_players(player: &str) -> Result<(), Error> {
    let file_name: String = format!("./Data/{player}.csv");

    let formatted_contents: String = format!("{}", &reading_file());
    let html: &str = formatted_contents.as_str();

    let fragment: Html = Html::parse_fragment(html);
    let selector: Selector = Selector::parse("tr").unwrap();
    let mut wtr: Writer<File> = Writer::from_path(file_name.as_str())?;

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

// fn txt_to_json() {
//     // Lire le fichier texte
//     let data: String =
//         fs::read_to_string("joueurs_tennis_1.txt").expect("Erreur lors de la lecture du fichier");

//     // Diviser le contenu du fichier texte en un vecteur de noms
//     let noms: Vec<&str> = data.split('\n').collect();

//     // Créer un objet Rust avec la structure requise
//     let obj: serde_json::Value = json!({
//         "joueurs": noms
//     });

//     // Convertir l'objet en une chaîne JSON
//     let json: String = obj.to_string();

//     // Écrire la chaîne JSON dans un fichier
//     fs::write("joueurs.json", json).expect("Erreur lors de l'écriture du fichier");
// }
