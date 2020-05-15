use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn read_from_file(fname: &str) -> Result<String> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    let f = File::open(fname)?;
    let mut reader = BufReader::new(f);
    let mut data = String::new();
    reader.read_line(&mut data)?;
    Ok(data)
}

fn write_to_file(fname: &str, contents: &str) -> Result<()> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut f = File::create(fname)?;
    f.write_all(contents.as_bytes())?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    scope: Vec<String>,
    token_type: String,
}

fn nonce(length: usize) -> String {
    use rand::Rng;
    let mut text = String::new();
    let possible = "abcdefghijklmnopqrstuvwxyz0123456789";
    for _ in 0..length {
        let idx = rand::thread_rng().gen_range(0, possible.len() - 1) as usize;
        text += &possible[idx..idx + 1];
    }
    text
}

fn main() -> Result<()> {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use url::Url;

    let client_secret = read_from_file("client-secret")?;
    let client_id = read_from_file("client-id")?;

    let redirect_url = "http://localhost:8080";
    let scope = "channel:read:redemptions";
    let state = nonce(30);

    let auth_url = Url::parse(&format!(
        "https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        client_id,
        redirect_url,
        scope,
        state,
    ))?;

    println!("Open this URL in your browser:\n{}\n", auth_url.to_string());

    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Listening on localhost:8080");
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
            println!("reading from stream");
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line)?;
                //println!("read: {}", request_line);

                let redirect_url = request_line
                    .split_whitespace()
                    .nth(1)
                    .expect("Unable to split whitespace");
                let url = Url::parse(&("http://localhost".to_owned() + redirect_url))?;

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .expect("No code URL parameter");

                let (_, value) = code_pair;
                code = value.into_owned();
                //println!("code = {}", code);

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .expect("No state URL parameter");

                let (_, value) = state_pair;
                let state_uri = value.into_owned();
                if state == state_uri {
                    println!("State code matches");
                } else {
                    eprintln!("WARNING: State codes DO NOT MATCH! Something went wrong");
                    return Err("Bad State response".into());
                }
                //println!("state = {}", state);
            }

            let message = "Yay! We got a token from twitch";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes())?;

            let reqwest_client = reqwest::blocking::Client::new();
            let params: &[(&str, &str)] = &[
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("code", &code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", redirect_url),
            ];
            let res: TokenResponse = reqwest_client
                .post("https://id.twitch.tv/oauth2/token")
                .form(&params)
                .send()?
                .json()?;
            println!("Twitch responded and parsed");
            let fname = "oauth-token";
            write_to_file(fname, &res.access_token)?;
            println!("oauth token saved to file named: '{}'", fname);
            break;
        }
    }
    Ok(())
}
