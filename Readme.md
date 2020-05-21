# Setup
## twitch application
add an application to get client id and the secret
## generate OAuth access token
generate [client_credentials](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-client-credentials-flow):

```bash
curl -X POST 'https://id.twitch.tv/oauth2/token?client_id=<your client ID>&client_secret=<your client secret>&grant_type=client_credentials'
```

client ID and secret are from twitch application, call returns access token

# Run
```fish
env TWITCH_CLIENT_ID=<client_id> BEARER_TOKEN_KEY=<access_token> cargo run -- -s
```
