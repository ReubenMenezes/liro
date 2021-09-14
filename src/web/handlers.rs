use super::error::{Error::*, Result};
use crate::{
    db::Pool,
    lichess,
    models::{Challenge, User},
};
use askama::Template;
use serde::Deserialize;
use warp::Reply;

#[derive(Template)]
#[template(path = "linked.html")]
struct AccountLinkedTemplate<'a> {
    username: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct CallbackParams {
    code: String,
    state: u64,
}

pub async fn oauth_callback_handler(params: CallbackParams, pool: Pool) -> Result<impl Reply> {
    trace!("oauth_callback_handler() called");
    let challenge = Challenge::find(&pool, params.state)
        .await
        .map_err(|_| DBAccessError)?
        .ok_or_else(|| ChallengeNotFoundError)?;

    let access_token = lichess::auth::fetch_access_token(&params.code, &challenge.code_verifier())
        .await
        .map_err(|e| LichessError(e))?;
    let lichess_user = lichess::api::fetch_account(&access_token)
        .await
        .map_err(|e| LichessError(e))?;

    let user = User::new(
        &pool,
        challenge.discord_id(),
        lichess_user.get_username().to_string(),
    )
    .await
    .map_err(|_| DBAccessError)?;

    let template = AccountLinkedTemplate {
        username: user.lichess_username(),
    };

    match template.render() {
        Ok(output) => Ok(warp::reply::html(output)),
        Err(e) => Err(warp::reject::custom(TemplateError(e))),
    }
}
