use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ZammadCreateTicketRequest {
    title: String,
    customer: String,
    group: String,
    article: ZammadCreateTicketArticle,
}

#[derive(Debug, Serialize)]
pub struct ZammadCreateTicketArticle {
    body: String,
    subject: String,
    #[serde(rename = "type")]
    _type: String,
    internal: bool,
}
