use crate::lichess;
use askama::Template;
use std::convert::Infallible;
use thiserror::Error;
use warp::{http::StatusCode, reply, reply::html, Rejection, Reply};

#[derive(Error, Debug)]
pub enum Error {
    #[error("error accessing database")]
    DBAccess,
    #[error("authentication challenge not found")]
    ChallengeNotFound,
    #[error("templating error: {0}")]
    Template(#[from] askama::Error),
    #[error("lichess error: {0}")]
    Lichess(#[from] lichess::Error),
}

impl warp::reject::Reject for Error {}

pub type Result<T> = std::result::Result<T, Rejection>;

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    message: &'static str,
}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    trace!("handle_rejection() called");
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::DBAccess => {
                code = StatusCode::BAD_REQUEST;
                message = "there was an error accessing the database";
            }
            _ => {
                error!("unhandled application error: {:?}", err);
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = "Internal Server Error";
            }
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed";
    } else {
        error!("unhandled error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let template = ErrorTemplate { message };

    match template.render() {
        Ok(v) => Ok(reply::with_status(html(v), code)),
        Err(_) => Ok(reply::with_status(
            html(String::from(message)),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}
