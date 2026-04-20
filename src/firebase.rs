use crate::trip::Trip;
use crate::utils;
use chrono::NaiveDate;
use leptos::logging::log;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

const CLIENT_ID: &str = env!("GOOGLE_CLIENT_ID");
const FIREBASE_API_KEY: &str = env!("FIREBASE_API_KEY");

fn save_token_and_user_id(token: &str, user_id: &str, refresh_token: &str) {
    let local_storage = utils::get_local_storage();
    local_storage.set_item("token", &token).unwrap();
    local_storage.set_item("user_id", &user_id).unwrap();
    local_storage
        .set_item("refresh_token", &refresh_token)
        .unwrap();
}

fn get_redirect_uri() -> String {
    let origin = utils::get_location().origin().unwrap(); // unwrap is ok because we are in the browser
    let pathname = get_redirect_pathname();
    origin + pathname
}

fn get_redirect_pathname() -> &'static str {
    option_env!("REDIRECT_URI").unwrap_or("/__/auth/handler")
}

pub fn start_google_redirect() {
    // CSRF
    let state = utils::random_b64url(16); // TODO: use state to know what to do next!
    let nonce = utils::random_b64url(16);

    // Create the redirect URI
    let redirect_uri = get_redirect_uri();

    // Build Google Redirect URL
    let url = web_sys::Url::new("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
    let q = url.search_params();
    q.append("client_id", CLIENT_ID);
    q.append("redirect_uri", &redirect_uri);
    q.append("response_type", "id_token");
    q.append("scope", "openid");
    q.append("state", &state);
    q.append("nonce", &nonce);
    let auth_url = url.href();

    log!("Redirecting to Google Auth: {}", auth_url);

    // Redirect to Google Auth
    let location = utils::get_location();
    location.set_href(&auth_url).unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct FirebaseLoginResponse {
    #[serde(rename = "idToken")]
    id_token: Option<String>,
    #[serde(rename = "localId")]
    user_id: Option<String>,
    #[serde(rename = "refreshToken")]
    refresh_token: Option<String>,
}

pub async fn handle_google_redirect() {
    let location = utils::get_location();
    let pathname = location.pathname().unwrap();
    if pathname != get_redirect_pathname() {
        return;
    }

    // Parse the fragment.
    let fragment = location.hash().unwrap_or_default();
    let id_token = fragment
        .split('&')
        .find(|pair| pair.starts_with("id_token="))
        .map(|pair| pair.split('=').nth(1).unwrap_or_default());
    if id_token.is_none() {
        log!("Missing id_token in fragment");
        return;
    }
    let id_token = id_token.unwrap();

    // Exchange code for token
    log!("ID Token: {}", id_token);

    // Now call the Firebase API to sign in with the ID token.
    let body = json!({
        "requestUri": location.origin().unwrap() + get_redirect_pathname(),
        "postBody": format!("id_token={}&providerId=google.com", id_token),
        "returnSecureToken": true,
    });

    let firebase_url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithIdp?key={}",
        FIREBASE_API_KEY
    );
    let response = reqwasm::http::Request::post(&firebase_url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .unwrap();

    let response_body = response.json::<FirebaseLoginResponse>().await;
    log!("Response: {:?}", response_body);

    let (id_token, user_id, refresh_token) = match response_body {
        // I really don't know if this is the best way to do this.
        Ok(FirebaseLoginResponse {
            id_token: Some(id_token),
            user_id: Some(user_id),
            refresh_token: Some(refresh_token),
        }) => (id_token, user_id, refresh_token),
        Err(e) => {
            log!("Error: {:?}", e);
            // TODO: Handle error
            return;
        }
        _ => {
            log!("Error: {:?}", response_body);
            // TODO: Handle error
            return;
        }
    };
    save_token_and_user_id(&id_token, &user_id, &refresh_token);

    // Now let's put something in the storage.

    // outside-days
    let firestore_url = format!(
        "https://firestore.googleapis.com/v1/projects/outside-days/databases/(default)/documents/trips/{}",
        user_id
    );
    let response = reqwasm::http::Request::get(&firestore_url)
        .header("Authorization", &format!("Bearer {}", id_token))
        .send()
        .await
        .unwrap();

    let response_body = response.json::<FirestoreDocument>().await.unwrap();
    log!("Firestore Response: {:?}", response_body);
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum FirestoreValue {
    StringValue(String),
    ArrayValue {
        values: Vec<FirestoreValue>,
    },
    MapValue {
        fields: HashMap<String, FirestoreValue>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct FirestoreDocument {
    name: String,
    fields: HashMap<String, FirestoreValue>,
    #[serde(rename = "createTime")]
    create_time: String,
    #[serde(rename = "updateTime")]
    update_time: String,
}

impl FirestoreDocument {
    pub fn get_trips(&self) -> Result<Vec<Trip>, String> {
        let mut trips = Vec::new();

        if let Some(FirestoreValue::ArrayValue { values }) = self.fields.get("trips") {
            for value in values {
                if let FirestoreValue::MapValue { fields } = value {
                    let Some(FirestoreValue::StringValue(dep)) = fields.get("dep") else {
                        return Err("Missing dep".into());
                    };
                    let Some(FirestoreValue::StringValue(ret)) = fields.get("ret") else {
                        return Err("Missing ret".into());
                    };
                    let Some(FirestoreValue::StringValue(des)) = fields.get("des") else {
                        return Err("Missing des".into());
                    };
                    trips.push(Trip::new(
                        NaiveDate::parse_from_str(dep, "%Y-%m-%d")
                            .map_err(|e| format!("Invalid date: {}: {}", dep, e))?,
                        NaiveDate::parse_from_str(ret, "%Y-%m-%d")
                            .map_err(|e| format!("Invalid date: {}: {}", ret, e))?,
                        des.clone(),
                    ));
                }
            }
        }
        Ok(trips)
    }
}
