use alloy_signer::Signature;
use alloy_sol_types::{sol, SolCall, SolValue};
use frankenstein::{
    ChatId, CreateChatInviteLinkParams, SendMessageParams, TelegramApi,
    UpdateContent::Message as TgMessage,
};
use kinode::process::standard::get_blob;
use kinode_process_lib::{
    await_message, call_init,
    eth::{call, Address as EthAddress, TransactionInput, TransactionRequest, U256, U64},
    http::{bind_http_path, serve_ui, HttpServerRequest},
    println, Address, Message,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
mod tg_api;
use tg_api::{init_tg_bot, Api, TgResponse};

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

/// after installing you need to boot the kinogate with some data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Initialize {
    tg_token: String,
    chain: u64,
    contract: String,
    min_amount: u64,
    priv_chat_id: i64,
    url: String,
}

/// incoming json frokm frontend.
#[derive(Deserialize)]
struct IncomingSignature {
    signature: String,
}

// balanceOf request on erc-20s and erc-721s
sol! {
    function balanceOf(address owner) external view returns (uint);
}

fn handle_message(
    _our: &Address,
    api: &Api,
    worker: &Address,
    info: &Initialize,
) -> anyhow::Result<()> {
    let message = await_message()?;

    if message.source().process == "http_server:distro:sys" {
        handle_http_message(message.clone(), api, worker, info)?
    }

    match message {
        Message::Response { .. } => {
            return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
        }
        Message::Request {
            ref source,
            ref body,
            ..
        } => match serde_json::from_slice(body) {
            Ok(TgResponse::Update(tg_update)) => {
                let updates = tg_update.updates;
                // assert update is from our worker
                if source != worker {
                    return Err(anyhow::anyhow!(
                        "unexpected source: {:?}, expected: {:?}",
                        source,
                        worker
                    ));
                }

                if let Some(update) = updates.last() {
                    match &update.content {
                        TgMessage(msg) => {
                            // get msg contents, and branch on what to send back!
                            let text = msg.text.clone().unwrap_or_default();

                            if msg.chat.id == info.priv_chat_id as i64 {
                                // bot shouldn't be spamming the priv chat.
                                return Ok(());
                            }

                            // fill in default send_message params, switch content later!
                            let mut params = SendMessageParams {
                                chat_id: ChatId::Integer(msg.chat.id),
                                disable_notification: None,
                                entities: None,
                                link_preview_options: None,
                                message_thread_id: None,
                                parse_mode: None,
                                text: "temp".to_string(),
                                protect_content: None,
                                reply_markup: None,
                                reply_parameters: None,
                                // todo, maybe change api so can ..Default::default()?
                            };

                            match text.as_str() {
                                "/start" => {
                                    let text = format!("hello, to enter this chat, you need at least {:?} of {:?} on chainId {:?} to enter, if you have that, sign the simple message to get invite link.", info.min_amount, info.contract, info.chain);
                                    params.text = text;
                                    api.send_message(&params)?;

                                    // maybe save user into state here?
                                    let chat_id = msg.chat.id;

                                    let link = format!(
                                        "{}/kinogate:kinogate:template.os/?chat_id={}",
                                        info.url, chat_id
                                    );
                                    params.text = link;
                                    api.send_message(&params)?;
                                }
                                _ => {
                                    params.text = "type /start to start.".to_string();
                                    api.send_message(&params)?;
                                }
                            }
                        }
                        _ => {
                            println!("got unhandled tg update: {:?}", update);
                        }
                    }
                }
            }
            Ok(TgResponse::Error(e)) => {
                println!("error from tg worker: {:?}", e);
            }
            _ => {
                // extra http_client responses coming through, will investigate.
            }
        },
    }
    Ok(())
}

fn handle_http_message(
    message: Message,
    api: &Api,
    worker: &Address,
    info: &Initialize,
) -> anyhow::Result<()> {
    match message {
        Message::Request { ref body, .. } => {
            // check src?
            let http_req = serde_json::from_slice::<HttpServerRequest>(body)?;
            if let HttpServerRequest::Http(incoming) = http_req {
                if incoming.method()? == "POST" {
                    let chat_id = incoming
                        .query_params()
                        .get("chat_id")
                        .cloned()
                        .unwrap_or_else(|| "".to_string());

                    let mut params = SendMessageParams {
                        chat_id: ChatId::Integer(chat_id.parse::<i64>()?),
                        disable_notification: None,
                        entities: None,
                        link_preview_options: None,
                        message_thread_id: None,
                        parse_mode: None,
                        text: "temp".to_string(),
                        protect_content: None,
                        reply_markup: None,
                        reply_parameters: None,
                    };

                    let blob = get_blob();
                    // get signature, check for that address actually owns the amount,
                    // generate invite link, and send!
                    if let Some(sig) = blob {
                        params.text = "verifying your signature...".to_string();
                        api.send_message(&params)?;

                        let sig: String =
                            serde_json::from_slice::<IncomingSignature>(&sig.bytes)?.signature;

                        let sig = Signature::from_str(&sig)?;

                        let address = sig.recover_address_from_msg("hello".as_bytes())?;
                        let contract_addy = EthAddress::from_str(&info.contract)?;

                        let input = balanceOfCall { owner: address };

                        let tx_input = TransactionInput {
                            data: Some(input.abi_encode().into()),
                            ..Default::default()
                        };

                        let tx_req = TransactionRequest {
                            chain_id: Some(U64::from(info.chain)),
                            to: Some(contract_addy),
                            input: tx_input,
                            ..Default::default()
                        };

                        let bal = call(tx_req, None)?;
                        let bal = U256::abi_decode(&bal.to_vec(), false)?;
                        let bal = bal.to::<u64>();

                        if bal >= info.min_amount {
                            params.text = "damn what a chad, pls do join!".to_string();
                            api.send_message(&params)?;

                            let chat_params = CreateChatInviteLinkParams {
                                chat_id: ChatId::Integer(info.priv_chat_id),
                                expire_date: None,
                                member_limit: Some(1),
                                creates_join_request: None,
                                name: None,
                            };
                            let link = api.create_chat_invite_link(&chat_params)?;

                            params.text = link.result.invite_link;

                            api.send_message(&params)?;
                        } else {
                            params.text =
                                "you don't have enough tokens to enter... it's over for u"
                                    .to_string();
                            api.send_message(&params)?;
                        }
                    } else {
                        println!("http error, no signature sent...");
                        return Ok(());
                    }
                }
            }
        }
        _ => {
            return Err(anyhow::anyhow!("unexpected message: {:?}", message));
        }
    }
    Ok(())
}

call_init!(init);

fn init(our: Address) {
    println!("kinogate: initialize me to begin!");

    let message = await_message().unwrap();
    let info = serde_json::from_slice::<Initialize>(message.body()).unwrap();

    let (api, worker) = init_tg_bot(our.clone(), &info.tg_token, None).unwrap();

    // serve ui and bind http post path.
    serve_ui(&our, "ui", false, false, vec!["/"]).unwrap();

    bind_http_path("/signature", false, false).unwrap();

    loop {
        match handle_message(&our, &api, &worker, &info) {
            Ok(()) => {}
            Err(e) => {
                println!("kinogate: error: {:?}", e);
            }
        };
    }
}
