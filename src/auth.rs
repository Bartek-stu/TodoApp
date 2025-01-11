use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    http::header::HeaderMap,
    HttpMessage,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::Deserialize;

use crate::model::UserId;

#[derive(Debug, Deserialize, Clone)]
pub struct ClientPrincipal {
    pub auth_typ: String,
    pub name_typ: String,
    pub role_typ: String,
    pub claims: Vec<Claim>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Claim {
    pub typ: String,
    pub val: String,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub principal_id: UserId,
    pub principal_name: String,
    pub idp: String,
    pub claims: ClientPrincipal,
}

pub async fn auth_middleware(
    req: ServiceRequest,
    next: actix_web::middleware::Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let headers = req.headers();

    let client_principal = extract_client_principal(headers)?;

    let principal_id = UserId::from(extract_header(headers, "X-MS-CLIENT-PRINCIPAL-ID")?);
    let principal_name = extract_header(headers, "X-MS-CLIENT-PRINCIPAL-NAME")?;
    let idp = extract_header(headers, "X-MS-CLIENT-PRINCIPAL-IDP")?;

    let auth_context = AuthContext {
        principal_id,
        principal_name,
        idp,
        claims: client_principal,
    };

    req.extensions_mut().insert(auth_context);

    next.call(req).await
}

fn extract_client_principal(headers: &HeaderMap) -> Result<ClientPrincipal, actix_web::Error> {
    let encoded = headers
        .get("X-MS-CLIENT-PRINCIPAL")
        .ok_or_else(|| ErrorUnauthorized("Missing X-MS-CLIENT-PRINCIPAL header"))?
        .to_str()
        .map_err(|_| ErrorUnauthorized("Invalid X-MS-CLIENT-PRINCIPAL header"))?;

    let decoded = BASE64_STANDARD
        .decode(encoded.as_bytes())
        .map_err(|_| ErrorUnauthorized("Failed to decode client principal"))?;

    let decoded_str = String::from_utf8(decoded)
        .map_err(|_| ErrorUnauthorized("Invalid UTF-8 in client principal"))?;

    serde_json::from_str::<ClientPrincipal>(&decoded_str)
        .map_err(|_| ErrorUnauthorized("Invalid JSON in client principal"))
}

fn extract_header(headers: &HeaderMap, key: &str) -> Result<String, actix_web::Error> {
    let value = headers
        .get(key)
        .ok_or_else(|| ErrorUnauthorized(format!("Missing header: {}", key)))?
        .to_str()
        .map_err(|_| ErrorUnauthorized(format!("Invalid header: {}", key)))?
        .to_string();
    Ok(value)
}
