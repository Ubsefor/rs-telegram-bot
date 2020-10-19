use tbot::{
    markup::{inline_code, markdown_v2},
    prelude::*,
    types::{
        input_file::{Photo},
        inline_query::{self, result::Article},
        input_message_content::Text,
        parameters::Text as ParseMode,
    },
    Bot,
};

use tokio::sync::Mutex;
use reqwest::get;
#[macro_use]
extern crate error_chain;
extern crate image;

const PHOTO: &[u8] = include_bytes!("./assets/photo.jpg");

error_chain!{
    foreign_links {
        Io(std::io::Error);
        Reqwest(reqwest::Error);
    }
}

async fn fetch_picture() -> Result <String> {
    let tmp_res = reqwest::get("https://inspirobot.me/api?generate=true").await?;
    let tmp_body = tmp_res.text().await?;
    let target = tmp_body.as_str();
    let response = reqwest::get(target).await?;
    let content =  response.text().await?;
    Ok(content.to_string())
}

#[tokio::main]
async fn main() {
    let mut bot =
        Bot::from_env("RS_TBOT_TOKEN").stateful_event_loop(Mutex::new(0_u32));

    bot.text(|context, _| async move {
        let calc_result = meval::eval_str(&context.text.value);
        let message = if let Ok(answer) = calc_result {
            markdown_v2(("= ", inline_code([answer.to_string()]))).to_string()
        } else {
            markdown_v2("Whops, I couldn't evaluate your expression :(")
                .to_string()
        };
        let reply = ParseMode::markdown_v2(&message);

        let call_result = context.send_message_in_reply(reply).call().await;
        if let Err(err) = call_result {
            dbg!(err);
        }
    });

    bot.command("inspire", |context, _| async move {
        let raw_img = fetch_picture().await.unwrap();
        println!("Got response: {:?}", raw_img);
        let photo = Photo::bytes(raw_img.as_bytes());
        let call_result = context.send_photo(photo).call().await;
        if let Err(err) = call_result {
            dbg!(err);
        }
    });

    bot.inline(move |context, id| async move {
        let calc_result = meval::eval_str(&context.query);
        let (title, message) = if let Ok(answer) = calc_result {
            let answer = answer.to_string();
            let message = markdown_v2(inline_code([
                context.query.as_str(),
                " = ",
                answer.as_str(),
            ]))
            .to_string();
            (answer, message)
        } else {
            let title = "Whops...".into();
            let message = markdown_v2("I couldn't evaluate your expression :(")
                .to_string();
            (title, message)
        };

        let id = {
            let mut id = id.lock().await;
            *id += 1;
            id.to_string()
        };
        let content = Text::new(ParseMode::markdown_v2(&message));
        let article = Article::new(&title, content).description(&message);
        let result = inline_query::Result::new(&id, article);

        let call_result = context.answer(&[result]).call().await;
        if let Err(err) = call_result {
            dbg!(err);
        }
    });

    bot.polling().start().await.unwrap();
}

