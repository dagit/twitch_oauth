use twitch_oauth::get_app_access_token;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn read_from_file(fname: &str) -> Result<String> {
    use std::io::prelude::*;
    use std::io::BufReader;
    use std::fs::File;

    let f = File::open(fname)?;
    let mut reader = BufReader::new(f);
    let mut data = String::new();
    reader.read_line(&mut data)?;
    Ok(data)
}

#[tokio::main]
async fn main() -> Result<()> {
    use async_tungstenite::tokio::connect_async_with_tls_connector;
    use tungstenite::{Message};
    use native_tls::TlsConnector;
    use futures::stream::StreamExt;
    use futures::sink::SinkExt;

    let client_secret = read_from_file("client-secret")?;
    let client_id = read_from_file("client-id")?;
    let token = get_app_access_token(&client_id, &client_secret,
                                    vec!["chat:read".to_owned()]).await?;
    println!("{}", token.expires_in);

    let ws_uri = url::Url::parse("wss://pubsub-edge.twitch.tv")?;
    let cx = TlsConnector::builder().build()?;
    let cx = tokio_tls::TlsConnector::from(cx);
    let (mut socket, response) =
        connect_async_with_tls_connector(ws_uri, Some(cx)).await?;
    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    let listen = format!("{{
        \"type\": \"LISTEN\",
        \"nonce\": \"44h1k13746815ab1r2\",
        \"data\": {{
            \"topics\": [\"channel-points-channel-v1.61258231\"],
            \"auth_token\": \"{}\"
        }}
    }}", token.access_token);

    socket.send(Message::Text(listen.into())).await?;
    while let Some(message) = socket.next().await {
        println!("{:#?}", message);
    }
    Ok(())

}
