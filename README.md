# fitbit-rs

Example oauth2 program talking to the fitbit api

# Setup

1. [Register](https://dev.fitbit.com/apps/new) a Fitbit app here.
2. Set OAuth 2.0 Application Type to "Personal"
3. Set the redirect URL to http://localhost:8080/callback
4. Set the `CLIENT_ID` and `CLIENT_SECRET` environment variables with the values from the website
5. `cargo run`
6. Click the link and approve the access.

You should get intraday heartbeat data for today.
