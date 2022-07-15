use colored::Colorize;
use regex::Regex;
use reqwest::header;
use std::time::Duration;
use std::{collections::HashMap, env, fs, fs::File, io::Write};

fn remove_duplicate_str(str_slice: Vec<String>) -> Vec<String> {
    let mut all_keys: HashMap<String, bool> = HashMap::new();
    let mut list: Vec<String> = Vec::new();
    for item in str_slice.into_iter() {
        if !all_keys.contains_key(&item) {
            all_keys.insert(item.clone(), true);
            list.push(item);
        }
    }
    list
}

fn retrieve_links(cyberlink: &str) -> Result<(Vec<String>, &str), reqwest::Error> {
    let text = get_request(cyberlink).unwrap_or_else(|err| panic!("Error getting request: {err}"));

    let re = Regex::new(r#"href="(https://fs([\w\-_]+(?:(?:\.[\w\-_]+)+))([\w\-\.,@?^=%&amp;:/~\+#]*[\w\-@?^=%&amp;/~\+#])?).([\w\-\.,@?^=%&amp;:/~\+#]*[\w\-@?^=%&amp;/~\+#])"#).unwrap();
    let raw_data: String = re
        .find_iter(&text)
        .map(|link| link.as_str())
        .collect::<Vec<&str>>()
        .join(" ");

    let re_links = Regex::new(r#"(https://fs([\w\-_]+(?:(?:\.[\w\-_]+)+))([\w\-\.,@?^=%&amp;:/~\+#]*[\w\-@?^=%&amp;/~\+#])?).([\w\-\.,@?^=%&amp;:/~\+#]*[\w\-@?^=%&amp;/~\+#])[:>.jpg|.jpeg|.png|.gif|.webp|.mp4|.webm|.mov|.mkv]"#).unwrap();
    let links: Vec<String> = re_links
        .find_iter(&raw_data)
        .map(|link| String::from(link.as_str()))
        .collect();

    Ok((links, cyberlink))
}

fn link_dispatcher((mut links, cyberlink): (Vec<String>, &str)) {
    links = remove_duplicate_str(links);
    for (x, mut link) in links.clone().into_iter().enumerate() {
        if link.contains("href") {
            let re = Regex::new(r#"(https://fs([\w\-_]+(?:(?:\.[\w\-_]+)+))([\w\-\.,@?^=%&amp;:/~\+#]*[\w\-@?^=%&amp;/~\+#])?)"#).unwrap();
            link = re.find(link.as_str()).unwrap().as_str().parse().unwrap();

            if x <= links.len() {
                println!(
                    "{}",
                    format!("Starting request for {}", &link[27..])
                        .bold()
                        .green()
                        .underline()
                );
                download(cyberlink, link).unwrap_or_else(|err| {
                    println!(
                        "{}",
                        format!("Error downloading request: {err}").bold().red()
                    )
                })
            }
        }
    }
}

fn download(cyberlink: &str, link: String) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::builder()
        .default_headers(header::HeaderMap::new())
        .timeout(Duration::from_secs(600))
        .build()?;
    let response = client.get(link.as_str()).send()?;
    let body = response.bytes();

    let mut file = File::create(format!("{}/{}", &cyberlink[23..], &link[27..]))
        .unwrap_or_else(|err| panic!("Error creating file: {err}"));
    file.write_all(
        body.unwrap_or_else(|err| panic!("Error unwrapping data: {err}"))
            .as_ref(),
    )
    .unwrap_or_else(|err| println!("{}", format!("Error writing to file: {err}").bold().red()));

    println!(
        "{}",
        format!("Downloaded {}", &link[27..])
            .bold()
            .green()
            .underline()
    );
    Ok(())
}

fn get_request(url: &str) -> Result<String, reqwest::Error> {
    let result = reqwest::blocking::get(url);
    let response = match result {
        Ok(res) => res,
        Err(err) => {
            println!("{}", format!("Error getting request: {err}").bold().red());
            return Err(err);
        }
    };

    let body = response.text();
    return match body {
        Ok(string) => Ok(string),
        Err(err) => {
            println!("{}", format!("Error parsing request: {err}").bold().red());
            Err(err)
        }
    };
}

fn folder_verifier(cyberlink: &str) {
    fs::create_dir_all(&cyberlink[23..]).unwrap_or_else(|err| panic!("Error creating dir: {err}"));
}

fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();

    let cyberlink = &args[1];
    folder_verifier(cyberlink);
    let retrieved_links = match retrieve_links(cyberlink) {
        Ok(links_tuple) => links_tuple,
        Err(err) => {
            println!("{}", format!("Error retrieving links: {err}").bold().red());
            return Err(err);
        }
    };
    link_dispatcher(retrieved_links);
    Ok(())
}
