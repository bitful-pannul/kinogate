use frankenstein::{ChatId, SendMessageParams, TelegramApi, UpdateContent::Message as TgMessage};
use kinode_process_lib::{
    await_message, call_init,
    http::{bind_http_path, serve_ui},
    println, Address, Message,
};
use serde::{Deserialize, Serialize};
mod tg_api;
use tg_api::{init_tg_bot, Api, TgResponse};

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Initialize {
    tg_token: String,
    chain: u64,
    contract: String,
    min_amount: u64,
    priv_chat_id: u64,
}

fn handle_message(
    _our: &Address,
    api: &Api,
    worker: &Address,
    info: &Initialize,
) -> anyhow::Result<()> {
    let message = await_message()?;

    match message {
        Message::Response { .. } => {
            return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
        }
        Message::Request {
            ref source,
            ref body,
            ..
        } => match serde_json::from_slice(body)? {
            TgResponse::Update(tg_update) => {
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
                                    let text = format!("hello, to enter this chat, you need at least {:?} of {:?} on chainId {:?} to enter, if you have that, sign the message to get invite link.", info.min_amount, info.contract, info.chain);
                                    params.text = text;
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
            TgResponse::Error(e) => {
                println!("error from tg worker: {:?}", e);
            }
        },
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
    serve_ui(
        &our,
        &format!("{}/pkg/ui/", our.package_id()),
        false,
        false,
        vec![],
    )
    .unwrap();

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
