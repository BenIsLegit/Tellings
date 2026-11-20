//! Talking to ElevenLabs: list the voices, the models and the pronunciation
//! dictionaries, check the credits, record speech, make music.
//!
//! Only `speak`, `converse` and `compose` cost credits. The caller is
//! responsible for having asked first.

use crate::error::{Error, Result};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

pub const BASE: &str = "https://api.elevenlabs.io";
/// Where a test run says ElevenLabs is. See `dev/stand-in-elevenlabs.mjs`.
pub const BASE_ENV_VAR: &str = "PVC_ELEVENLABS_URL";
/// The sound format the first video was recorded in.
pub const MP3: &str = "mp3_44100_128";

pub struct Client {
    http: reqwest::blocking::Client,
    base: String,
    key: String,
    /// How long to wait before each new try when ElevenLabs says it is busy.
    waits: Vec<Duration>,
}

/// How a voice speaks when a setup does not say: the settings stored with it.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct OwnSettings {
    pub stability: Option<f32>,
    #[serde(alias = "similarity_boost")]
    pub similarity: Option<f32>,
    pub style: Option<f32>,
    #[serde(alias = "use_speaker_boost")]
    pub speaker_boost: Option<bool>,
    pub speed: Option<f32>,
}

/// Read a part that is nice to have. If it is not shaped as expected, go without it.
fn if_it_fits<'de, D, T>(d: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    Ok(serde_json::from_value(Value::deserialize(d)?).ok())
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
    /// The voice's own settings, when the list carries them. It often does
    /// not: `Client::voice_settings` asks for one voice's.
    #[serde(default, deserialize_with = "if_it_fits")]
    pub settings: Option<OwnSettings>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Language {
    pub id: String,
    pub name: String,
}

/// One ElevenLabs model that turns text into speech.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Credits for each letter. 1 for most models, 0.5 for the fast ones.
    pub credits_per_letter: f32,
    /// The most letters one piece of speech may hold. 0 when ElevenLabs did not say.
    pub max_letters: u64,
    pub can_use_style: bool,
    pub can_use_speaker_boost: bool,
    /// True when it reads a tag like `[reverent]` as direction. See `takes_direction`.
    #[serde(default)]
    pub takes_direction: bool,
    /// True when it can record several voices as one conversation. See `speaks_dialogue`.
    #[serde(default)]
    pub speaks_dialogue: bool,
    pub languages: Vec<Language>,
}

/// True for the models that read a tag like `[reverent]` as direction instead
/// of saying it aloud: Eleven v3, v4 and whatever follows them. The older
/// models (Multilingual v2, Flash, Turbo) would speak the words. ElevenLabs'
/// list of models does not say which is which, so this goes by the name.
pub fn takes_direction(model: &str) -> bool {
    let number: String = model.strip_prefix("eleven_v").unwrap_or("").chars().take_while(char::is_ascii_digit).collect();
    number.parse::<u32>().is_ok_and(|n| n >= 3)
}

/// True for the models that can record several voices as one conversation in
/// a plain request: Eleven v3, v4 and whatever follows them. Their "turbo" and
/// "conversational" kin only do it over a live connection, which this tool does
/// not use, so the name has to end at the number. ElevenLabs' list of models
/// does not say which is which, so this goes by the name.
pub fn speaks_dialogue(model: &str) -> bool {
    let number = model.strip_prefix("eleven_v").unwrap_or("");
    !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()) && number.parse::<u32>().is_ok_and(|n| n >= 3)
}

/// The most letters one conversation may hold, all its parts together.
/// ElevenLabs: "keep the total character count ... at or below 2,000".
pub const DIALOGUE_MAX_LETTERS: u64 = 2000;

/// A pronunciation dictionary kept in the ElevenLabs account.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryInfo {
    pub id: String,
    pub name: String,
    /// Its newest version.
    pub version: String,
    /// How many words it knows.
    pub rules: u64,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Credits {
    pub used: i64,
    pub limit: i64,
    /// The plan: "free", "starter", "creator" …
    pub tier: Option<String>,
    /// When the count starts again, in seconds since 1970.
    pub resets: Option<i64>,
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
    /// The letters those times belong to, in order. Empty when ElevenLabs did
    /// not send them one letter at a time.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
}

/// Everything one request for speech can say. Whatever is left out is left to
/// the voice's own settings and to ElevenLabs.
#[derive(Clone, Debug, Default)]
pub struct SpeechRequest<'a> {
    pub voice_id: &'a str,
    pub model: &'a str,
    pub text: &'a str,
    pub stability: Option<f32>,
    pub similarity: Option<f32>,
    pub style: Option<f32>,
    pub speaker_boost: Option<bool>,
    /// How fast the model itself speaks. ElevenLabs takes 0.7 to 1.2.
    pub speed: Option<f32>,
    pub clone_strength: Option<f32>,
    /// The same number asks for the same recording again, as near as ElevenLabs can.
    pub seed: Option<u32>,
    /// A language code, like "en".
    pub language: Option<&'a str>,
    /// "on" or "off". Left out, ElevenLabs decides.
    pub normalize: Option<&'a str>,
    /// Pronunciation dictionaries, as (id, version). Three at most.
    pub dictionaries: &'a [(String, String)],
    /// Like "mp3_44100_128". Empty means that one.
    pub output_format: &'a str,
}

/// Everything one request for a conversation can say. A conversation takes
/// fewer settings than one voice does: no style, no speaker boost, no speed.
#[derive(Clone, Debug, Default)]
pub struct DialogueRequest<'a> {
    pub model: &'a str,
    /// Each part in order, as (voice id, words).
    pub parts: &'a [(String, String)],
    pub stability: Option<f32>,
    pub similarity: Option<f32>,
    pub clone_strength: Option<f32>,
    pub seed: Option<u32>,
    pub language: Option<&'a str>,
    /// "on" or "off". Left out, ElevenLabs decides.
    pub normalize: Option<&'a str>,
    /// Pronunciation dictionaries, as (id, version). Three at most.
    pub dictionaries: &'a [(String, String)],
    /// Like "mp3_44100_128". Empty means that one.
    pub output_format: &'a str,
}

/// Where one voice speaks in a conversation, as ElevenLabs reports it.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct VoiceSegment {
    /// Which part of the conversation this is, counting from 0.
    pub part: usize,
    /// When it starts and ends, in seconds.
    pub start: f32,
    pub end: f32,
    /// Where it starts and ends among the timed letters. The end is not included.
    pub first_letter: usize,
    pub letters_end: usize,
}

#[derive(Debug)]
pub struct Spoken {
    /// A sound file, in the format asked for.
    pub audio: Vec<u8>,
    pub alignment: Option<Alignment>,
    /// The same timings for the text after ElevenLabs spelled out numbers and short forms.
    pub normalized: Option<Alignment>,
    /// For a conversation: where each voice speaks. Empty for one voice.
    pub segments: Vec<VoiceSegment>,
    /// The credits ElevenLabs says it charged, when it says.
    pub cost: Option<u64>,
    /// ElevenLabs' own name for this request.
    pub request_id: Option<String>,
}

/// The ElevenLabs model that makes music. The one ElevenLabs uses when none
/// is named is being withdrawn, so this one is always named.
pub const MUSIC_MODEL: &str = "music_v2_5";

/// How long to wait for a piece of music. It is made in one go and can take minutes.
const MUSIC_WAIT: Duration = Duration::from_secs(900);

/// One request for music, made from a description in words.
#[derive(Clone, Debug, Default)]
pub struct MusicRequest<'a> {
    pub prompt: &'a str,
    /// How long a piece, in thousandths of a second. ElevenLabs takes 3 seconds to 10 minutes.
    pub length_ms: u64,
    /// No singing.
    pub instrumental: bool,
}

#[derive(Debug)]
pub struct Composed {
    /// A sound file, in the format asked for.
    pub audio: Vec<u8>,
    /// ElevenLabs' own name for the piece, when it sends one.
    pub song_id: Option<String>,
    /// The credits ElevenLabs says it charged. It is not known to say so for
    /// music, but if it ever does, this is where it lands.
    pub cost: Option<u64>,
    pub request_id: Option<String>,
}

/// 1.1 is held as 1.10000002. Send the number the person chose.
fn tidy(v: f32) -> Value {
    json!((v as f64 * 1000.0).round() / 1000.0)
}

/// Read one set of letter timings out of an answer.
fn alignment_in(body: &Value, key: &str) -> Option<Alignment> {
    let times = |name: &str| -> Option<Vec<f32>> { body[key][name].as_array().map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect()) };
    match (times("character_start_times_seconds"), times("character_end_times_seconds")) {
        (Some(starts), Some(ends)) if !starts.is_empty() && starts.len() == ends.len() => {
            let letters: Vec<&str> = body[key]["characters"].as_array().map(|a| a.iter().map(|c| c.as_str().unwrap_or("")).collect()).unwrap_or_default();
            let one_each = letters.len() == starts.len() && letters.iter().all(|c| c.chars().count() == 1);
            Some(Alignment { starts, ends, text: if one_each { letters.concat() } else { String::new() } })
        }
        _ => None,
    }
}

impl Client {
    pub fn new(key: &str) -> Result<Self> {
        Self::with_base(key, BASE)
    }

    /// The real service, unless a test run has pointed the tool at a stand-in.
    pub fn from_env(key: &str) -> Result<Self> {
        match std::env::var(BASE_ENV_VAR) {
            Ok(base) if !base.trim().is_empty() => Self::with_base(key, &base),
            _ => Self::new(key),
        }
    }

    pub fn with_base(key: &str, base: &str) -> Result<Self> {
        let http = reqwest::blocking::Client::builder().timeout(Duration::from_secs(180)).user_agent("prayer-video-creator").build()?;
        let waits = [2, 6, 15].map(Duration::from_secs).to_vec();
        Ok(Client { http, base: base.trim_end_matches('/').to_string(), key: key.to_string(), waits })
    }

    /// Change how long to wait between tries when ElevenLabs is busy. One try for each wait.
    pub fn patient(mut self, waits: Vec<Duration>) -> Self {
        self.waits = waits;
        self
    }

    fn get(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        self.http.get(format!("{}{path}", self.base)).header("xi-api-key", &self.key)
    }

    fn check(&self, resp: reqwest::blocking::Response) -> Result<reqwest::blocking::Response> {
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }
        let body: Value = resp.json().unwrap_or(Value::Null);
        Err(Self::problem(status.as_u16(), &body))
    }

    /// What went wrong, in plain words, from an answer that was not a yes.
    fn problem(status: u16, body: &Value) -> Error {
        let detail = &body["detail"];
        // One problem comes as { status, message }. A badly shaped request comes as a list of them.
        let said = match detail {
            Value::Array(list) => list.iter().filter_map(|d| d["msg"].as_str()).collect::<Vec<_>>().join(" "),
            _ => detail["message"].as_str().or_else(|| detail.as_str()).unwrap_or("").to_string(),
        };
        let code = detail["status"].as_str().or_else(|| detail["code"].as_str()).unwrap_or("");
        Error::voice(match status {
            _ if code == "missing_permissions" => {
                format!("This ElevenLabs API key is not allowed to do that. {said} Make a key that is, at elevenlabs.io under API keys, and save it in Settings.")
            }
            401 => "ElevenLabs did not accept the API key. Check it in Settings.".to_string(),
            _ if code == "quota_exceeded" => format!("ElevenLabs says there are not enough credits left. {said}"),
            402 => format!("ElevenLabs says this needs a paid plan or more credits. {said}"),
            404 => format!("ElevenLabs could not find that voice or model. {said}"),
            429 => "ElevenLabs is busy or you have sent too many requests. Wait a minute and try again.".to_string(),
            n => format!("ElevenLabs answered with error {n}. {said}"),
        })
    }

    /// Every voice in the account.
    pub fn voices(&self) -> Result<Vec<VoiceInfo>> {
        let mut out: Vec<VoiceInfo> = Vec::new();
        let mut token: Option<String> = None;
        for _ in 0..50 {
            let mut req = self.get("/v2/voices").query(&[("page_size", "100")]);
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

    /// How one voice is set to speak at ElevenLabs: what is used wherever a
    /// setup leaves a setting unset.
    pub fn voice_settings(&self, voice_id: &str) -> Result<OwnSettings> {
        let body: Value = self.check(self.get(&format!("/v1/voices/{voice_id}/settings")).send()?)?.json()?;
        Ok(serde_json::from_value(body).unwrap_or_default())
    }

    /// Every model that turns text into speech, in the order ElevenLabs lists them.
    pub fn models(&self) -> Result<Vec<ModelInfo>> {
        let body: Value = self.check(self.get("/v1/models").send()?)?.json()?;
        let mut out = Vec::new();
        for m in body.as_array().map(Vec::as_slice).unwrap_or(&[]) {
            let Some(id) = m["model_id"].as_str().filter(|i| !i.is_empty()) else { continue };
            if m["can_do_text_to_speech"] != true {
                continue;
            }
            let rates = &m["model_rates"];
            let rate = rates["character_cost_multiplier"].as_f64().unwrap_or(1.0) * rates["cost_discount_multiplier"].as_f64().unwrap_or(1.0);
            let languages = m["languages"].as_array().map(Vec::as_slice).unwrap_or(&[]);
            out.push(ModelInfo {
                id: id.to_string(),
                name: m["name"].as_str().filter(|n| !n.is_empty()).unwrap_or(id).to_string(),
                description: m["description"].as_str().unwrap_or("").trim().to_string(),
                credits_per_letter: if rate > 0.0 { rate as f32 } else { 1.0 },
                max_letters: m["maximum_text_length_per_request"].as_u64().unwrap_or(0),
                can_use_style: m["can_use_style"] == true,
                can_use_speaker_boost: m["can_use_speaker_boost"] == true,
                takes_direction: takes_direction(id),
                speaks_dialogue: speaks_dialogue(id),
                languages: languages
                    .iter()
                    .filter_map(|l| Some(Language { id: l["language_id"].as_str()?.to_string(), name: l["name"].as_str().unwrap_or("").to_string() }))
                    .collect(),
            });
        }
        Ok(out)
    }

    /// The pronunciation dictionaries in the account that are still in use.
    pub fn dictionaries(&self) -> Result<Vec<DictionaryInfo>> {
        let mut out = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..50 {
            let mut req = self.get("/v1/pronunciation-dictionaries").query(&[("page_size", "100"), ("include_archived", "false")]);
            if let Some(c) = &cursor {
                req = req.query(&[("cursor", c.as_str())]);
            }
            let body: Value = self.check(req.send()?)?.json()?;
            for d in body["pronunciation_dictionaries"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
                let (Some(id), Some(name), Some(version)) = (d["id"].as_str(), d["name"].as_str(), d["latest_version_id"].as_str()) else { continue };
                if d["archived_time_unix"].as_i64().is_some() {
                    continue;
                }
                out.push(DictionaryInfo { id: id.to_string(), name: name.to_string(), version: version.to_string(), rules: d["latest_version_rules_num"].as_u64().unwrap_or(0) });
            }
            cursor = body["next_cursor"].as_str().filter(|c| !c.is_empty()).map(String::from);
            if body["has_more"] != true || cursor.is_none() {
                break;
            }
        }
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(out)
    }

    pub fn credits(&self) -> Result<Credits> {
        let body: Value = self.check(self.get("/v1/user/subscription").send()?)?.json()?;
        Ok(Credits {
            used: body["character_count"].as_i64().unwrap_or(0),
            limit: body["character_limit"].as_i64().unwrap_or(0),
            tier: body["tier"].as_str().filter(|t| !t.is_empty()).map(String::from),
            resets: body["next_character_count_reset_unix"].as_i64().filter(|t| *t > 0),
        })
    }

    /// Record one piece of speech. This spends credits: about one for each
    /// letter, more or less by the model.
    pub fn speak(&self, req: &SpeechRequest) -> Result<Spoken> {
        let mut body = json!({ "text": req.text, "model_id": req.model });
        let mut settings = serde_json::Map::new();
        if let Some(v) = req.stability {
            settings.insert("stability".into(), tidy(v));
        }
        if let Some(v) = req.similarity {
            settings.insert("similarity_boost".into(), tidy(v));
        }
        if let Some(v) = req.style {
            settings.insert("style".into(), tidy(v));
        }
        if let Some(v) = req.speaker_boost {
            settings.insert("use_speaker_boost".into(), json!(v));
        }
        if let Some(v) = req.speed {
            settings.insert("speed".into(), tidy(v));
        }
        if let Some(v) = req.clone_strength {
            settings.insert("clone_strength".into(), tidy(v));
        }
        if !settings.is_empty() {
            body["voice_settings"] = Value::Object(settings);
        }
        if let Some(v) = req.seed {
            body["seed"] = json!(v);
        }
        if let Some(v) = req.language {
            body["language_code"] = json!(v);
        }
        if let Some(v) = req.normalize {
            body["apply_text_normalization"] = json!(v);
        }
        if !req.dictionaries.is_empty() {
            body["pronunciation_dictionary_locators"] = req.dictionaries.iter().map(|(id, version)| json!({ "pronunciation_dictionary_id": id, "version_id": version })).collect();
        }
        let url = format!("{}/v1/text-to-speech/{}/with-timestamps", self.base, req.voice_id);
        self.ask_for_sound(&url, req.output_format, &body)
    }

    /// Record several voices as one conversation: each part in its own voice,
    /// each following on from the one before. This spends credits, as `speak` does.
    pub fn converse(&self, req: &DialogueRequest) -> Result<Spoken> {
        let inputs: Vec<Value> = req.parts.iter().map(|(voice_id, words)| json!({ "text": words, "voice_id": voice_id })).collect();
        let mut body = json!({ "inputs": inputs, "model_id": req.model });
        let mut settings = serde_json::Map::new();
        if let Some(v) = req.stability {
            settings.insert("stability".into(), tidy(v));
        }
        if let Some(v) = req.similarity {
            settings.insert("similarity".into(), tidy(v));
        }
        if let Some(v) = req.clone_strength {
            settings.insert("clone_strength".into(), tidy(v));
        }
        if !settings.is_empty() {
            body["settings"] = Value::Object(settings);
        }
        if let Some(v) = req.seed {
            body["seed"] = json!(v);
        }
        if let Some(v) = req.language {
            body["language_code"] = json!(v);
        }
        if let Some(v) = req.normalize {
            body["apply_text_normalization"] = json!(v);
        }
        if !req.dictionaries.is_empty() {
            body["pronunciation_dictionary_locators"] = req.dictionaries.iter().map(|(id, version)| json!({ "pronunciation_dictionary_id": id, "version_id": version })).collect();
        }
        let url = format!("{}/v1/text-to-dialogue/with-timestamps", self.base);
        self.ask_for_sound(&url, req.output_format, &body)
    }

    /// Make one piece of music from a description. This spends credits, and
    /// ElevenLabs gives no way to ask how many first: see `music.rs`.
    ///
    /// Like `speak`, it tries again only after a "busy" answer. It is never
    /// sent twice otherwise: a piece may have been made and paid for.
    pub fn compose(&self, req: &MusicRequest) -> Result<Composed> {
        let body = json!({ "prompt": req.prompt, "music_length_ms": req.length_ms, "model_id": MUSIC_MODEL, "force_instrumental": req.instrumental });
        let url = format!("{}/v1/music", self.base);
        let mut waits = self.waits.iter();
        let resp = loop {
            // Every plan can have this format. Left to itself, ElevenLabs picks one that some plans cannot.
            let sent = self.http.post(&url).header("xi-api-key", &self.key).query(&[("output_format", MP3)]).timeout(MUSIC_WAIT).json(&body).send();
            let resp = match sent {
                Ok(resp) => resp,
                Err(e) if e.is_timeout() => {
                    return Err(Error::voice(format!(
                        "ElevenLabs took more than {} minutes over the music, so the tool stopped waiting. It may still have made the piece and charged for it. \
                         Look at your credits before trying again.",
                        MUSIC_WAIT.as_secs() / 60
                    )))
                }
                Err(e) => return Err(e.into()),
            };
            match waits.next() {
                Some(wait) if resp.status().as_u16() == 429 => std::thread::sleep(*wait),
                _ => break resp,
            }
        };
        let status = resp.status();
        if !status.is_success() {
            let body: Value = resp.json().unwrap_or(Value::Null);
            let detail = &body["detail"];
            // ElevenLabs will not make music from a prompt that names a song, an artist or a label. It often offers other words.
            if detail["status"] == "bad_prompt" || detail["code"] == "bad_prompt" {
                let offer = detail["data"]["prompt_suggestion"].as_str().map(str::trim).filter(|s| !s.is_empty());
                return Err(Error::voice(match offer {
                    Some(words) => format!(
                        "ElevenLabs would not make music from those words. It refuses the name of a hymn, a song or an artist. Describe the sound instead. It suggests: \"{words}\""
                    ),
                    None => "ElevenLabs would not make music from those words. It refuses the name of a hymn, a song or an artist, and anything harmful. Describe the sound instead.".to_string(),
                }));
            }
            return Err(Self::problem(status.as_u16(), &body));
        }
        let header = |name: &str| resp.headers().get(name).and_then(|v| v.to_str().ok()).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let cost = header("character-cost").and_then(|v| v.parse::<f64>().ok()).filter(|c| *c >= 0.0).map(|c| c.ceil() as u64);
        let (song_id, request_id) = (header("song-id"), header("request-id"));
        let audio = resp.bytes()?.to_vec();
        if audio.is_empty() {
            return Err(Error::voice("ElevenLabs answered without any music. Look at your credits before trying again: it may have charged for the piece."));
        }
        Ok(Composed { audio, song_id, cost, request_id })
    }

    /// Send a request that records, and read the sound, the timings and the cost out of the answer.
    fn ask_for_sound(&self, url: &str, output_format: &str, body: &Value) -> Result<Spoken> {
        let format = if output_format.is_empty() { MP3 } else { output_format };

        // "Busy" means ElevenLabs turned the request away: nothing was recorded
        // and nothing was charged, so asking again is safe. Any other failure is
        // not tried again, because a recording may have been made and paid for.
        let mut waits = self.waits.iter();
        let resp = loop {
            let resp = self.http.post(url).header("xi-api-key", &self.key).query(&[("output_format", format)]).json(body).send()?;
            match waits.next() {
                Some(wait) if resp.status().as_u16() == 429 => std::thread::sleep(*wait),
                _ => break resp,
            }
        };
        let resp = self.check(resp)?;
        let header = |name: &str| resp.headers().get(name).and_then(|v| v.to_str().ok()).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let cost = header("character-cost").and_then(|v| v.parse::<f64>().ok()).filter(|c| *c >= 0.0).map(|c| c.ceil() as u64);
        let request_id = header("request-id");

        let body: Value = resp.json()?;
        let audio = body["audio_base64"].as_str().ok_or_else(|| Error::voice("ElevenLabs answered without any sound."))?;
        let audio = base64::engine::general_purpose::STANDARD.decode(audio).map_err(|e| Error::voice(format!("The sound from ElevenLabs was damaged: {e}")))?;
        // A segment that is not shaped as expected is left out. The timing copes without it.
        let segment = |s: &Value| -> Option<VoiceSegment> {
            Some(VoiceSegment {
                part: s["dialogue_input_index"].as_u64()? as usize,
                start: s["start_time_seconds"].as_f64()? as f32,
                end: s["end_time_seconds"].as_f64()? as f32,
                first_letter: s["character_start_index"].as_u64()? as usize,
                letters_end: s["character_end_index"].as_u64()? as usize,
            })
        };
        let segments = body["voice_segments"].as_array().map(|list| list.iter().filter_map(segment).collect()).unwrap_or_default();
        Ok(Spoken { audio, alignment: alignment_in(&body, "alignment"), normalized: alignment_in(&body, "normalized_alignment"), segments, cost, request_id })
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
            .with_body(r#"{"voices":[{"voice_id":"v2","name":"Zed","settings":{"stability":0.4,"similarity_boost":0.8,"style":0,"use_speaker_boost":true,"speed":1.0}}],"has_more":true,"next_page_token":"abc"}"#)
            .create();
        let second = server
            .mock("GET", "/v2/voices")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page_size".into(), "100".into()),
                mockito::Matcher::UrlEncoded("next_page_token".into(), "abc".into()),
            ]))
            .with_body(r#"{"voices":[{"voice_id":"v1","name":"Imperial - Male","category":"generated","settings":"not what we expect"}],"has_more":false}"#)
            .create();
        let client = Client::with_base("test-key", &server.url()).unwrap();
        let voices = client.voices().unwrap();
        second.assert();
        first.assert();
        assert_eq!(voices.iter().map(|v| v.name.as_str()).collect::<Vec<_>>(), ["Imperial - Male", "Zed"]);
        assert_eq!(voices[0].id, "v1");
        // The voice's own settings come along. Odd ones are dropped, not the voice.
        assert_eq!(voices[0].settings, None);
        let own = voices[1].settings.clone().unwrap();
        assert_eq!((own.stability, own.similarity, own.speaker_boost), (Some(0.4), Some(0.8), Some(true)));
        // The list is kept on disk in the tool's own words, and reads back the same.
        let kept: Vec<VoiceInfo> = serde_json::from_str(&serde_json::to_string(&voices).unwrap()).unwrap();
        assert_eq!(kept, voices);
    }

    #[test]
    fn a_voice_s_own_settings() {
        let mut server = mockito::Server::new();
        server.mock("GET", "/v1/voices/v1/settings").match_header("xi-api-key", "k").with_body(r#"{"stability":0.45,"similarity_boost":0.8,"style":0.0,"use_speaker_boost":true,"speed":1.0}"#).create();
        let own = Client::with_base("k", &server.url()).unwrap().voice_settings("v1").unwrap();
        assert_eq!(own, OwnSettings { stability: Some(0.45), similarity: Some(0.8), style: Some(0.0), speaker_boost: Some(true), speed: Some(1.0) });
    }

    #[test]
    fn lists_the_models_that_speak_and_what_they_cost() {
        let mut server = mockito::Server::new();
        server
            .mock("GET", "/v1/models")
            .match_header("xi-api-key", "test-key")
            .with_body(
                r#"[
                {"model_id":"eleven_v4","name":"Eleven v4","description":" The newest. ","can_do_text_to_speech":true,"can_use_style":false,"can_use_speaker_boost":false,
                 "maximum_text_length_per_request":100000,"languages":[{"language_id":"en","name":"English"},{"language_id":"es","name":"Spanish"}],
                 "model_rates":{"character_cost_multiplier":1.0}},
                {"model_id":"eleven_flash_v2_5","name":"Eleven Flash v2.5","description":"Fast.","can_do_text_to_speech":true,"can_use_style":false,"can_use_speaker_boost":false,
                 "maximum_text_length_per_request":40000,"languages":[],"model_rates":{"character_cost_multiplier":0.5,"cost_discount_multiplier":1.0}},
                {"model_id":"eleven_multilingual_sts_v2","name":"Voice changer","can_do_text_to_speech":false,"model_rates":{"character_cost_multiplier":1.0}}
            ]"#,
            )
            .create();
        let models = Client::with_base("test-key", &server.url()).unwrap().models().unwrap();
        // The voice changer does not turn text into speech, so it is left out.
        assert_eq!(models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(), ["eleven_v4", "eleven_flash_v2_5"]);
        assert_eq!((models[0].name.as_str(), models[0].description.as_str(), models[0].max_letters), ("Eleven v4", "The newest.", 100_000));
        assert_eq!(models[0].languages.len(), 2);
        assert_eq!((models[0].credits_per_letter, models[1].credits_per_letter), (1.0, 0.5));
        assert!(models[0].takes_direction && !models[1].takes_direction);
        assert!(models[0].speaks_dialogue && !models[1].speaks_dialogue);
    }

    #[test]
    fn records_speech_with_timings() {
        let mut server = mockito::Server::new();
        let m = server
            .mock("POST", "/v1/text-to-speech/v1/with-timestamps")
            .match_query(mockito::Matcher::UrlEncoded("output_format".into(), "mp3_44100_128".into()))
            .match_header("xi-api-key", "test-key")
            .match_body(mockito::Matcher::Json(json!({ "text": "Amen.", "model_id": "eleven_multilingual_v2", "voice_settings": { "stability": 0.5 } })))
            .with_header("character-cost", "5")
            .with_header("request-id", "req-1")
            .with_body(r#"{"audio_base64":"AAEC","alignment":{"characters":["A","m","e","n","."],"character_start_times_seconds":[0,0.1,0.2,0.3,0.4],"character_end_times_seconds":[0.1,0.2,0.3,0.4,0.5]}}"#)
            .create();
        let client = Client::with_base("test-key", &server.url()).unwrap();
        let spoken = client.speak(&SpeechRequest { voice_id: "v1", model: "eleven_multilingual_v2", text: "Amen.", stability: Some(0.5), ..Default::default() }).unwrap();
        m.assert();
        assert_eq!(spoken.audio, [0, 1, 2]);
        let alignment = spoken.alignment.unwrap();
        assert_eq!((alignment.starts.len(), alignment.text.as_str()), (5, "Amen."));
        // ElevenLabs says what it charged, and names the request.
        assert_eq!((spoken.cost, spoken.request_id.as_deref()), (Some(5), Some("req-1")));
    }

    #[test]
    fn everything_a_setup_can_ask_for_is_sent() {
        let mut server = mockito::Server::new();
        let m = server
            .mock("POST", "/v1/text-to-speech/v1/with-timestamps")
            .match_query(mockito::Matcher::UrlEncoded("output_format".into(), "mp3_44100_192".into()))
            .match_body(mockito::Matcher::Json(json!({
                "text": "Amen.",
                "model_id": "eleven_flash_v2_5",
                "voice_settings": { "stability": 0.35, "similarity_boost": 0.8, "style": 0.25, "use_speaker_boost": false, "speed": 1.1, "clone_strength": 0.5 },
                "seed": 7,
                "language_code": "en",
                "apply_text_normalization": "off",
                "pronunciation_dictionary_locators": [{ "pronunciation_dictionary_id": "d1", "version_id": "v9" }],
            })))
            .with_body(r#"{"audio_base64":"AAEC"}"#)
            .create();
        let client = Client::with_base("k", &server.url()).unwrap();
        let dictionaries = [("d1".to_string(), "v9".to_string())];
        let spoken = client
            .speak(&SpeechRequest {
                voice_id: "v1",
                model: "eleven_flash_v2_5",
                text: "Amen.",
                // Numbers a computer cannot hold exactly still arrive as the person wrote them.
                stability: Some(0.35),
                similarity: Some(0.8),
                style: Some(0.25),
                speaker_boost: Some(false),
                speed: Some(1.1),
                clone_strength: Some(0.5),
                seed: Some(7),
                language: Some("en"),
                normalize: Some("off"),
                dictionaries: &dictionaries,
                output_format: "mp3_44100_192",
            })
            .unwrap();
        m.assert();
        // No timings and no cost were sent back, and that is not an error.
        assert!(spoken.alignment.is_none() && spoken.cost.is_none());
    }

    #[test]
    fn two_voices_are_recorded_as_one_conversation() {
        let mut server = mockito::Server::new();
        let m = server
            .mock("POST", "/v1/text-to-dialogue/with-timestamps")
            .match_query(mockito::Matcher::UrlEncoded("output_format".into(), "mp3_44100_192".into()))
            .match_header("xi-api-key", "test-key")
            // A conversation takes "similarity", not "similarity_boost", and has no style, boost or speed.
            .match_body(mockito::Matcher::Json(json!({
                "inputs": [{ "text": "Hail Mary,", "voice_id": "lead" }, { "text": "Holy Mary,", "voice_id": "answer" }],
                "model_id": "eleven_v4",
                "settings": { "stability": 0.35, "similarity": 0.8, "clone_strength": 0.5 },
                "seed": 7,
                "language_code": "en",
                "apply_text_normalization": "off",
                "pronunciation_dictionary_locators": [{ "pronunciation_dictionary_id": "d1", "version_id": "v9" }],
            })))
            .with_header("character-cost", "20")
            .with_header("request-id", "req-2")
            .with_body(
                r#"{"audio_base64":"AAEC",
                "alignment":{"characters":["H","i","Y","o"],"character_start_times_seconds":[0,0.1,0.5,0.6],"character_end_times_seconds":[0.1,0.2,0.6,0.7]},
                "normalized_alignment":{"characters":["H","i"," ","Y","o"],"character_start_times_seconds":[0,0.1,0.2,0.5,0.6],"character_end_times_seconds":[0.1,0.2,0.5,0.6,0.7]},
                "voice_segments":[
                    {"voice_id":"lead","start_time_seconds":0,"end_time_seconds":0.2,"character_start_index":0,"character_end_index":2,"dialogue_input_index":0},
                    {"voice_id":"answer","start_time_seconds":0.5,"end_time_seconds":0.7,"character_start_index":2,"character_end_index":4,"dialogue_input_index":1},
                    {"voice_id":"answer","oddly":"shaped"}
                ]}"#,
            )
            .create();
        let client = Client::with_base("test-key", &server.url()).unwrap();
        let parts = [("lead".to_string(), "Hail Mary,".to_string()), ("answer".to_string(), "Holy Mary,".to_string())];
        let dictionaries = [("d1".to_string(), "v9".to_string())];
        let spoken = client
            .converse(&DialogueRequest {
                model: "eleven_v4",
                parts: &parts,
                stability: Some(0.35),
                similarity: Some(0.8),
                clone_strength: Some(0.5),
                seed: Some(7),
                language: Some("en"),
                normalize: Some("off"),
                dictionaries: &dictionaries,
                output_format: "mp3_44100_192",
            })
            .unwrap();
        m.assert();
        assert_eq!((spoken.audio.as_slice(), spoken.cost, spoken.request_id.as_deref()), ([0u8, 1, 2].as_slice(), Some(20), Some("req-2")));
        assert_eq!(spoken.alignment.unwrap().text, "HiYo");
        assert_eq!(spoken.normalized.unwrap().text, "Hi Yo");
        // Where each voice speaks. The one that was oddly shaped is left out.
        assert_eq!(
            spoken.segments,
            [VoiceSegment { part: 0, start: 0.0, end: 0.2, first_letter: 0, letters_end: 2 }, VoiceSegment { part: 1, start: 0.5, end: 0.7, first_letter: 2, letters_end: 4 }]
        );

        // With nothing chosen, nothing is sent but the words, the voices and the model.
        let plain = server
            .mock("POST", "/v1/text-to-dialogue/with-timestamps")
            .match_query(mockito::Matcher::UrlEncoded("output_format".into(), "mp3_44100_128".into()))
            .match_body(mockito::Matcher::Json(json!({ "inputs": [{ "text": "A", "voice_id": "x" }, { "text": "B", "voice_id": "y" }], "model_id": "eleven_v3" })))
            .with_body(r#"{"audio_base64":"AAEC"}"#)
            .create();
        let parts = [("x".to_string(), "A".to_string()), ("y".to_string(), "B".to_string())];
        let spoken = client.converse(&DialogueRequest { model: "eleven_v3", parts: &parts, ..Default::default() }).unwrap();
        plain.assert();
        assert!(spoken.alignment.is_none() && spoken.segments.is_empty() && spoken.cost.is_none());
    }

    #[test]
    fn which_models_can_record_a_conversation() {
        assert!(speaks_dialogue("eleven_v3") && speaks_dialogue("eleven_v4") && speaks_dialogue("eleven_v12"));
        // Their quick kin only do it over a live connection, and the older models not at all.
        for model in ["eleven_v4_turbo", "eleven_v3_conversational", "eleven_multilingual_v2", "eleven_flash_v2_5", "eleven_turbo_v2", "eleven_v2", "eleven_v", ""] {
            assert!(!speaks_dialogue(model), "{model}");
        }
    }

    #[test]
    fn a_busy_answer_is_tried_again_and_nothing_else_is() {
        let mut server = mockito::Server::new();
        let busy = server.mock("POST", "/v1/text-to-speech/busy/with-timestamps").match_query(mockito::Matcher::Any).with_status(429).expect(3).create();
        let client = Client::with_base("k", &server.url()).unwrap().patient(vec![Duration::ZERO, Duration::ZERO]);
        let err = client.speak(&SpeechRequest { voice_id: "busy", model: "m", text: "Amen.", ..Default::default() }).unwrap_err().to_string();
        busy.assert();
        assert!(err.contains("busy"), "{err}");

        // A refusal that is not "busy" is final: a second try could be charged twice.
        let refused = server
            .mock("POST", "/v1/text-to-speech/refused/with-timestamps")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body(r#"{"detail":{"status":"unsupported_setting","message":"This model does not take a speed."}}"#)
            .expect(1)
            .create();
        let err = client.speak(&SpeechRequest { voice_id: "refused", model: "m", text: "Amen.", ..Default::default() }).unwrap_err().to_string();
        refused.assert();
        assert!(err.contains("does not take a speed"), "{err}");
    }

    #[test]
    fn music_is_asked_for_by_its_description() {
        let mut server = mockito::Server::new();
        let m = server
            .mock("POST", "/v1/music")
            // Named, so every plan can have it. Left out, ElevenLabs picks a finer one that some plans cannot.
            .match_query(mockito::Matcher::UrlEncoded("output_format".into(), "mp3_44100_128".into()))
            .match_header("xi-api-key", "test-key")
            // The model is always named: the one ElevenLabs falls back to is being withdrawn.
            .match_body(mockito::Matcher::Json(json!({ "prompt": "slow, quiet organ", "music_length_ms": 120000, "model_id": "music_v2_5", "force_instrumental": true })))
            .with_header("song-id", "song-1")
            .with_header("request-id", "req-2")
            .with_body([0u8, 1, 2, 3])
            .create();
        let client = Client::with_base("test-key", &server.url()).unwrap();
        let composed = client.compose(&MusicRequest { prompt: "slow, quiet organ", length_ms: 120_000, instrumental: true }).unwrap();
        m.assert();
        assert_eq!(composed.audio, [0, 1, 2, 3]);
        assert_eq!((composed.song_id.as_deref(), composed.request_id.as_deref()), (Some("song-1"), Some("req-2")));
        // ElevenLabs is not known to say what music cost. Nothing is made up in its place.
        assert_eq!(composed.cost, None);
    }

    #[test]
    fn music_is_never_asked_for_twice_unless_elevenlabs_was_busy() {
        let mut server = mockito::Server::new();
        let client = Client::with_base("k", &server.url()).unwrap().patient(vec![Duration::ZERO, Duration::ZERO]);
        let ask = MusicRequest { prompt: "slow organ", length_ms: 30_000, instrumental: true };

        // Busy: turned away, nothing made, nothing charged. Safe to ask again.
        let busy = server.mock("POST", "/v1/music").match_query(mockito::Matcher::Any).with_status(429).expect(3).create();
        let err = client.compose(&ask).unwrap_err().to_string();
        busy.assert();
        assert!(err.contains("busy"), "{err}");
        busy.remove();

        // Anything else is final. A piece may have been made and charged for.
        let failed = server.mock("POST", "/v1/music").match_query(mockito::Matcher::Any).with_status(500).with_body(r#"{"detail":{"status":"server_error","message":"Something broke."}}"#).expect(1).create();
        let err = client.compose(&ask).unwrap_err().to_string();
        failed.assert();
        assert!(err.contains("error 500") && err.contains("Something broke."), "{err}");
        failed.remove();

        // An answer with no sound in it is an error, and says the piece may have been charged for.
        let empty = server.mock("POST", "/v1/music").match_query(mockito::Matcher::Any).with_body("").expect(1).create();
        let err = client.compose(&ask).unwrap_err().to_string();
        empty.assert();
        assert!(err.contains("without any music") && err.contains("may have charged"), "{err}");
        empty.remove();

        // A refusal carries ElevenLabs' own suggestion when there is one.
        let refused = server
            .mock("POST", "/v1/music")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body(r#"{"detail":{"status":"bad_prompt","message":"No.","data":{"prompt_suggestion":"A gentle hymn-like melody on organ"}}}"#)
            .expect(1)
            .create();
        let err = client.compose(&ask).unwrap_err().to_string();
        refused.assert();
        assert!(err.contains("would not make music") && err.contains("A gentle hymn-like melody on organ"), "{err}");
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
    fn a_key_that_may_not_do_something_says_so() {
        let mut server = mockito::Server::new();
        server.mock("GET", "/v1/models").with_status(401).with_body(r#"{"detail":{"status":"missing_permissions","message":"The API key you used is missing the permission models_read."}}"#).create();
        let err = Client::with_base("secret-key-123", &server.url()).unwrap().models().unwrap_err().to_string();
        assert!(err.contains("not allowed") && err.contains("models_read"), "{err}");
        assert!(!err.contains("secret-key-123"));
    }

    #[test]
    fn credits_left() {
        let mut server = mockito::Server::new();
        server.mock("GET", "/v1/user/subscription").with_body(r#"{"character_count":1000,"character_limit":10000,"tier":"creator","next_character_count_reset_unix":1767225600}"#).create();
        let client = Client::with_base("k", &server.url()).unwrap();
        let credits = client.credits().unwrap();
        assert_eq!(credits.left(), 9000);
        assert_eq!((credits.tier.as_deref(), credits.resets), (Some("creator"), Some(1_767_225_600)));
    }

    #[test]
    fn lists_pronunciation_dictionaries() {
        let mut server = mockito::Server::new();
        server
            .mock("GET", "/v1/pronunciation-dictionaries")
            .match_query(mockito::Matcher::Any)
            .with_body(
                r#"{"pronunciation_dictionaries":[
                {"id":"d2","name":"Prayers","latest_version_id":"v3","latest_version_rules_num":4},
                {"id":"d1","name":"Old","latest_version_id":"v1","latest_version_rules_num":1,"archived_time_unix":1700000000}
            ],"has_more":false}"#,
            )
            .create();
        let list = Client::with_base("k", &server.url()).unwrap().dictionaries().unwrap();
        assert_eq!(list, [DictionaryInfo { id: "d2".into(), name: "Prayers".into(), version: "v3".into(), rules: 4 }]);
    }
}
