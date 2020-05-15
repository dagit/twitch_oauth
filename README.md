# twitch_oauth
Simple Rust example of oauth with twitch

# Usage

1. Register an app at dev.twitch.tv
1. Put your client-id into a file named, `client-id`
1. Put your client-secret into a file named, `client-secret`
1. `cargo run` this program in the same directory as your client-id and client-secret
1. The program will print a URL, paste that into your browser. Note: You may need to use a private browsing window to avoid a CSRF error. You will also need to open port 8080 to receive the redirect from twitch.
1. The program will make one more HTTP POST, if that is successful it will write your new oauth token to a file named `oauth-token`.

This is just a toy sketch and not a fully fleshed out instance of how you should do this in a real application. For example, this throws away the renew code. Also it has the scope hard coded to `channel:read:redepmitons`.
