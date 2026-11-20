//! Talking to ElevenLabs: list the voices, check the credits, record speech.
//!
//! Only `speak` costs credits. The caller is responsible for having asked first.

use crate::error::{Error, Result};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

pub const BASE: &str = "https://api.elevenlabs.io";

pub struct Client {
    http: reqwest::blocking::Client,
    base: String,
    key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VoiceInfo {
    #[serde(alias = "voice_id")]
    pub id: String,
    pub name: String,
    /// "cloned", "generated", "premade", "professional" …
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, alias = "preview_url")]
    pub preview_url: Option<String>,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Credits {
    pub used: i64,
    pub limit: i64,
}

impl Credits {
    pub fn left(&self) -> i64 {
        (self.limit - self.used).max(0)
    }
}

/// When each letter of the text is spoken, in seconds from the start of the sound.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Alignment {
    pub starts: Vec<f32>,
    pub ends: Vec<f32>,
}

pub struct SpeechRequest<'a> {
    pub voice_id: &'a str,
    pub model: &'a str,
    pub text: &'a str,
    pub stability: Option<f32>,
    pub similarity: Option<f32>,
    pub style: Option<f32>,
}

pub struct Spoken {
    /// An MP3 file.
    pub audio: Vec<u8>,
    pub alignment: Option<Alignment>,
}

impl Client {
    pub fn new(key: &str) -> Result<Self> {
        Self::with_base(key, BASE)
    }

    pub fn with_base(key: &str, base: &str) -> Result<Self> {
        let http = reqwest::blocking::Client::builder().timeout(Duration::from_secs(180)).user_agent("prayer-video-creator").build()?;
        Ok(Client { http, base: base.trim_end_matches('/').to_string(), key: key.to_string() })
    }

    fn check(&self, resp: reqwest::blocking::Response) -> Result<reqwest::blocking::Response> {
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }
        let body: Value = resp.json().unwrap_or(Value::Null);
        let detail = &body["detail"];
        let said = detail["message"].as_str().or_else(|| detail.as_str()).unwrap_or("").to_string();
        let code = detail["status"].as_str().unwrap_or("");
        Err(Error::voice(match status.as_u16() {
            401 => "ElevenLabs did not accept the API key. Check it in Settings.".to_string(),
            _ if code == "quota_exceeded" => format!("ElevenLabs says there are not enough credits left. {said}"),
            402 => format!("ElevenLabs says this needs a paid plan or more credits. {said}"),
            404 => format!("ElevenLabs could not find that voice. {said}"),
            429 => "ElevenLabs is busy or you have sent too many requests. Wait a minute and try again.".to_string(),
            n => format!("ElevenLabs answered with error {n}. {said}"),
        }))
    }

    /// Every voice in the account.
    pub fn voices(&self) -> Result<Vec<VoiceInfo>> {
        let mut out: Vec<VoiceInfo> = Vec::new();
        let mut token: Option<String> = None;
        for _ in 0..50 {
            let mut req = self.http.get(format!("{}/v2/voices", self.base)).header("xi-api-key", &self.key).query(&[("page_size", "100")]);
            if let Some(t) = &token {
                req = req.query(&[("next_page_token", t.as_str())]);
            }
            let body: Value = self.check(req.send()?)?.json()?;
            if let Some(list) = body["voices"].as_array() {
                for v in list {
                    if let Ok(info) = serde_json::from_value::<VoiceInfo>(v.clone()) {
                        out.push(info);
                    }
                }
            }
            token = body["next_page_token"].as_str().filter(|t| !t.is_empty()).map(String::from);
            if body["has_more"] != true || token.is_none() {
                break;
            }
        }
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(out)
    }

    pub fn credits(&self) -> Result<Credits> {
        let resp = self.http.get(format!("{}/v1/user/subscription", self.base)).header("xi-api-key", &self.key).send()?;
        let body: Value = self.check(resp)?.json()?;
        Ok(Credits { used: body["character_count"].as_i64().unwrap_or(0), limit: body["character_limit"].as_i64().unwrap_or(0) })
    }

    /// Record one piece of speech. This spends credits: about one for each letter.
    pub fn speak(&self, req: &SpeechRequest) -> Result<Spoken> {
        let mut body = json!({ "text": req.text, "model_id": req.model });
        let mut settings = serde_json::Map::new();
        if let Some(v) = req.stability {
            settings.insert("stability".into(), json!(v));
        }
        if let Some(v) = req.similarity {
            settings.insert("similarity_boost".into(), json!(v));
        }
        if let Some(v) = req.style {
            settings.insert("style".into(), json!(v));
        }
        if !settings.is_empty() {
            body["voice_settings"] = Value::Object(settings);
        }
        let resp = self
            .http
            .post(format!("{}/v1/text-to-speech/{}/with-timestamps", self.base, req.voice_id))
            .header("xi-api-key", &self.key)
            .query(&[("output_format", "mp3_44100_128")])
            .json(&body)
            .send()?;
        let body: Value = self.check(resp)?.json()?;
        let audio = body["audio_base64"].as_str().ok_or_else(|| Error::voice("ElevenLabs answered without any sound."))?;
        let audio = base64::engine::general_purpose::STANDARD.decode(audio).map_err(|e| Error::voice(format!("The sound from ElevenLabs was damaged: {e}")))?;
        let times = |key: &str| -> Option<Vec<f32>> { body["alignment"][key].as_array().map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect()) };
        let alignment = match (times("character_start_times_seconds"), times("character_end_times_seconds")) {
            (Some(starts), Some(ends)) if !starts.is_empty() && starts.len() == ends.len() => Some(Alignment { starts, ends }),
            _ => None,
        };
        Ok(Spoken { audio, alignment })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_voices_across_pages() {
        let mut server = mockito::Server::new();
        let first = server
            .mock("GET", "/v2/voices")
            .match_query(mockito::Matcher::UrlEncoded("page_size".into(), "100".into()))
            .match_header("xi-api-key", "test-key")
            .with_body(r#"{"voices":[{"voice_id":"v2","name":"Zed"}],"has_more":true,"next_page_token":"abc"}"#)
            .create();
        let second = server
            .mock("GET", "/v2/voices")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page_size".into(), "100".into()),
                mockito::Matcher::UrlEncoded("next_page_token".into(), "abc".into()),
            ]))
            .with_body(r#"{"voices":[{"voice_id":"v1","name":"Imperial - Male","category":"generated"}],"has_more":false}"#)
            .create();
        let client = Client::with_base("test-key", &server.url()).unwrap();
        let voices = client.voices().unwrap();
        second.assert();
        first.assert();
        assert_eq!(voices.iter().map(|v| v.name.as_str()).collect::<Vec<_>>(), ["Imperial - Male", "Zed"]);
        assert_eq!(voices[0].id, "v1");
    }

    #[test]
    fn records_speech_with_timings() {
        let mut server = mockito::Server::new();
        let m = server
            .mock("POST", "/v1/text-to-speech/v1/with-timestamps")
            .match_query(mockito::Matcher::UrlEncoded("output_format".into(), "mp3_44100_128".into()))
            .match_header("xi-api-key", "test-key")
            .match_body(mockito::Matcher::Json(json!({ "text": "Amen.", "model_id": "eleven_multilingual_v2", "voice_settings": { "stability": 0.5 } })))
            .with_body(r#"{"audio_base64":"AAEC","alignment":{"characters":["A","m","e","n","."],"character_start_times_seconds":[0,0.1,0.2,0.3,0.4],"character_end_times_seconds":[0.1,0.2,0.3,0.4,0.5]}}"#)
            .create();
        let client = Client::with_base("test-key", &server.url()).unwrap();
        let spoken = client
            .speak(&SpeechRequest { voice_id: "v1", model: "eleven_multilingual_v2", text: "Amen.", stability: Some(0.5), similarity: None, style: None })
            .unwrap();
        m.assert();
        assert_eq!(spoken.audio, [0, 1, 2]);
        assert_eq!(spoken.alignment.unwrap().starts.len(), 5);
    }

    #[test]
    fn a_bad_key_is_said_plainly_and_never_echoed() {
        let mut server = mockito::Server::new();
        server.mock("GET", "/v1/user/subscription").with_status(401).with_body(r#"{"detail":{"status":"invalid_api_key","message":"Invalid API key"}}"#).create();
        let client = Client::with_base("secret-key-123", &server.url()).unwrap();
        let err = client.credits().unwrap_err().to_string();
        assert!(err.contains("did not accept the API key"), "{err}");
        assert!(!err.contains("secret-key-123"));
    }

    #[test]
    fn credits_left() {
        let mut server = mockito::Server::new();
        server.mock("GET", "/v1/user/subscription").with_body(r#"{"character_count":1000,"character_limit":10000}"#).create();
        let client = Client::with_base("k", &server.url()).unwrap();
        assert_eq!(client.credits().unwrap().left(), 9000);
    }
}
