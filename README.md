# Kinogate

A kinode app that runs a telegram bot, token-gating a private groupchat you have.

## Setup

To build, you need to have the compiled .wasm of the telegram bot worker in your `/pkg/` directory, which you can get by running `kit b` on this repo:
<https://github.com/kinode-dao/telegram-bot>.

And then ```cp /pkg/tg.wasm ../path_to_your_kinogate/pkg/```.

You can now start the app by running `kit bs`, and initialize it by giving it some json.

```m our@kinogate:kinogate:template.os json```

where json is:

```json
'{
  "tg_token": "your_tg_token",
  "chain": 11155111_or_some_other_chain,
  "contract": "your_token_contract, needs to implement balanceOf()",
  "min_amount": 1,
  "priv_chat_id": priv_chat_id i64,
  "url": "url where others can reach your signature_page, kinode needs to be direct for now"
}'
```

###

Then just text your bot /start (or something else), and share the link to it!
