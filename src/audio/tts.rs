use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

/// Text-to-Speech Engine Manager (Thai voices: th-TH-PremwadeeNeural, th-TH-NiwatNeural)
/// Powered by Microsoft Edge Neural AI with smart local caching for zero latency.
#[derive(Clone)]
pub struct TtsManager {
    cache_dir: PathBuf,
}

impl TtsManager {
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Self {
        let p = cache_dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&p);
        Self { cache_dir: p }
    }

    /// Generate unique cache file name based on text and voice
    pub fn get_cache_path(&self, text: &str, voice: &str) -> PathBuf {
        let hash = format!("{:x}", md5_hash(format!("{}_{}", text, voice).as_bytes()));
        self.cache_dir.join(format!("tts_{}.mp3", hash))
    }

    pub fn has_cached(&self, text: &str, voice: &str) -> Option<PathBuf> {
        let p = self.get_cache_path(text, voice);
        if p.exists() {
            Some(p)
        } else {
            None
        }
    }

    /// Synthesize speech to an MP3/WAV file, utilizing local cache if already synthesized
    pub async fn synthesize(&self, text: &str, voice: &str) -> Result<PathBuf, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err("Empty text for TTS".to_string());
        }

        let cache_file = self.get_cache_path(trimmed, voice);
        if cache_file.exists() && cache_file.metadata().map(|m| m.len()).unwrap_or(0) > 100 {
            return Ok(cache_file);
        }

        // 1. Attempt High-Quality Microsoft Edge Neural TTS (Real AI voices with distinct Male/Female)
        match synthesize_edge_tts(trimmed, voice).await {
            Ok(audio_bytes) if audio_bytes.len() > 100 => {
                if let Ok(_) = fs::write(&cache_file, &audio_bytes) {
                    return Ok(cache_file);
                }
            }
            Err(e) => {
                eprintln!("⚠️ [TTS] Edge Neural TTS failed: {}. Falling back to Cloud voice...", e);
            }
            _ => {}
        }

        // 2. Fallback to Cloud Voice TTS (Google Translate Voice API)
        match synthesize_cloud_tts(trimmed, voice).await {
            Ok(audio_bytes) if audio_bytes.len() > 100 => {
                if let Ok(_) = fs::write(&cache_file, &audio_bytes) {
                    return Ok(cache_file);
                }
            }
            Err(e) => {
                eprintln!("⚠️ [TTS] Cloud TTS fallback failed: {}. Trying System voice...", e);
            }
            _ => {}
        }

        // 3. Fallback: Windows System Speech Synthesis
        #[cfg(windows)]
        {
            let wav_hash = format!("{:x}", md5_hash(format!("sapi_{}_{}", trimmed, voice).as_bytes()));
            let wav_file = self.cache_dir.join(format!("tts_{}.wav", wav_hash));
            if wav_file.exists() && wav_file.metadata().map(|m| m.len()).unwrap_or(0) > 100 {
                return Ok(wav_file);
            }

            let escaped_text = trimmed.replace('\'', "''").replace('"', "`\"");
            let ps_cmd = format!(
                "Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.SetOutputToWaveFile('{}'); $s.Speak('{}'); $s.Dispose()",
                wav_file.display(),
                escaped_text
            );

            let res = tokio::process::Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
                .output()
                .await;

            if let Ok(out) = res {
                if out.status.success() && wav_file.exists() {
                    return Ok(wav_file);
                }
            }
        }

        Err("Failed to synthesize speech via Edge TTS and fallback synthesizer".to_string())
    }
}

const WIN_EPOCH: u64 = 11644473600;
const TRUSTED_CLIENT_TOKEN: &str = "6A5AA1D4EAFF4E9FB37E23D68491D6F4";
const CHROMIUM_MAJOR_VERSION: &str = "143";
const SEC_MS_GEC_VERSION: &str = "2-143.0.3650.75";

const SHA256_INITIAL: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
    0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

const SHA256_STATE: [u32; 8] = [
    0xFD9FBEA2, 0x91243CE7, 0xBE499094, 0xCD2E1E0A,
    0x6C006F49, 0x04BD35AB, 0xFEB0A7E4, 0x03EFFBD2,
];

const SHA256_SCHEDULE: [u32; 14] = [
    0xEB24BF9E, 0x1558691F, 0xA0C09EE2, 0x0CFA32BA,
    0xA622EE8C, 0xD1DD9491, 0xC0DD25A0, 0xF9E70AB6,
    0xF0C31573, 0x7058CE4E, 0x7188E606, 0x00E5AB33,
    0xD258B060, 0x72766564,
];

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5,
    0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
    0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3,
    0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
    0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC,
    0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
    0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7,
    0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
    0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13,
    0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
    0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3,
    0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
    0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5,
    0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
    0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208,
    0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2,
];

const SHA256_ADJUSTMENT: u32 = 0xC9DC06A7;
const SHA256_SIGMA0_ADJUSTMENT: u32 = 0x059C0B80;
const SHA256_SIGMA1_ADJUSTMENT: u32 = 0x000B92C0;

const CS64_COEFFICIENTS: [(u32, u32); 14] = [
    (0xE82E91B1, 0xB32700D0),
    (0x44DE24C6, 0x88DFF51D),
    (0x3BEB65BC, 0x29514038),
    (0xC89C5933, 0xB328CBDF),
    (0x8D1EC211, 0xD405B4D5),
    (0x314EF5D5, 0x26E55F3C),
    (0xE92491B1, 0x607801C6),
    (0xF0A5B9D5, 0x82B2C1FB),
    (0x06064CF3, 0xFA51A3DA),
    (0xDA79FA60, 0x207CC464),
    (0xBED20560, 0x3C3C74F3),
    (0x89F9C9F8, 0xD2D14B61),
    (0x35FA38C6, 0xBB04A2C1),
    (0xCB49E3C6, 0xC5626167),
];

const CS64_FACTOR: u32 = 0x82893933;
const CS64_ADJUSTMENT0: u32 = 0xF2F90B1A;
const CS64_ADJUSTMENT1: u32 = 0x0B1A0000;

fn rol16(v: u32) -> u32 {
    v.rotate_left(16)
}

fn sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

fn sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

fn expand_sha256_schedule(words: &mut Vec<u32>, first_word: usize) {
    for index in first_word..64 {
        let x = words[index - 2];
        let y = words[index - 15];
        let s1 = sigma1(x);
        let s0 = sigma0(y);
        let sum = s1.wrapping_add(words[index - 7]).wrapping_add(s0).wrapping_add(words[index - 16]);
        words.push(sum);
    }
}

fn sha256_rounds(state: [u32; 8], words: &[u32], first_round: usize) -> [u32; 8] {
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for index in first_round..64 {
        let k = if first_round == 14 && index == 14 {
            SHA256_ADJUSTMENT
        } else {
            SHA256_ROUND_CONSTANTS[index]
        };
        let w = words[index];
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = g ^ (e & (f ^ g));
        let t1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(k).wrapping_add(w);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) | (c & (a | b));
        let t2 = s0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    [a, b, c, d, e, f, g, h]
}

fn sha256_init(data: &[u8]) -> [u32; 8] {
    let mut words = vec![0u32; 14];
    let mut b1 = [0u8; 4];
    b1[1..4].copy_from_slice(&data[0..3]);
    words.push(u32::from_be_bytes(b1));

    let mut b2 = [0u8; 4];
    b2.copy_from_slice(&data[3..7]);
    words.push(u32::from_be_bytes(b2));

    for index in 16..30 {
        let mut val = SHA256_SCHEDULE[index - 16];
        if index - 2 >= 14 {
            let s1 = if index == 16 {
                sigma1(words[index - 2]) ^ SHA256_SIGMA1_ADJUSTMENT
            } else {
                sigma1(words[index - 2])
            };
            val = val.wrapping_add(s1);
        }
        if index - 7 >= 14 {
            val = val.wrapping_add(words[index - 7]);
        }
        if index - 15 >= 14 {
            let s0 = if index == 29 {
                sigma0(words[index - 15]) ^ SHA256_SIGMA0_ADJUSTMENT
            } else {
                sigma0(words[index - 15])
            };
            val = val.wrapping_add(s0);
        }
        words.push(val);
    }

    let val30 = sigma1(words[28])
        .wrapping_add(words[23])
        .wrapping_add(sigma0(words[15]))
        .wrapping_add(words[14])
        .wrapping_add(SHA256_ADJUSTMENT)
        .wrapping_sub(SHA256_ROUND_CONSTANTS[14]);
    words.push(val30);

    expand_sha256_schedule(&mut words, 31);
    let working = sha256_rounds(SHA256_STATE, &words, 14);

    let mut out = [0u32; 8];
    for i in 0..8 {
        out[i] = SHA256_INITIAL[i].wrapping_add(working[i]);
    }
    out
}

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let length = 57 + data.len();
    let mut padded = Vec::new();
    padded.extend_from_slice(&data[7..]);
    padded.push(0x80);
    let rem = (55usize.wrapping_sub(length)) % 64;
    padded.extend(std::iter::repeat(0u8).take(rem));
    padded.extend_from_slice(&(length as u64 * 8).to_be_bytes());

    let mut state = sha256_init(&data[0..7]);
    for chunk in padded.chunks_exact(64) {
        let mut words = Vec::with_capacity(64);
        for i in 0..16 {
            let w = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
            words.push(w);
        }
        expand_sha256_schedule(&mut words, 16);
        let working = sha256_rounds(state, &words, 0);
        for i in 0..8 {
            state[i] = state[i].wrapping_add(working[i]);
        }
    }

    let mut digest = [0u8; 32];
    for (i, &w) in state.iter().enumerate() {
        digest[i * 4..i * 4 + 4].copy_from_slice(&w.to_be_bytes());
    }
    digest
}

fn words_le4_even(data: &[u8]) -> Vec<u32> {
    let rem = (4 - (data.len() % 4)) % 4;
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat(0u8).take(rem));
    let mut words = Vec::new();
    for chunk in padded.chunks_exact(4) {
        words.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    if words.len() % 2 != 0 {
        words.push(0);
    }
    words
}

fn cs64_digest(data: &[u8], digest: &[u8; 32]) -> (u32, u32) {
    let k_a = u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]]) | 1;
    let k_b = u32::from_le_bytes([digest[4], digest[5], digest[6], digest[7]]) | 1;

    let factors = [
        k_a.wrapping_mul(CS64_FACTOR),
        k_b.wrapping_mul(CS64_FACTOR),
    ];

    let mut terms = Vec::new();
    for (index, (coeff, _)) in CS64_COEFFICIENTS.iter().enumerate() {
        terms.push(coeff.wrapping_mul(factors[index % 2]));
    }
    let mut offsets: Vec<u32> = CS64_COEFFICIENTS.iter().map(|(_, offset)| *offset).collect();

    let mut data_with_zero = Vec::with_capacity(1 + data.len());
    data_with_zero.push(0x00);
    data_with_zero.extend_from_slice(data);

    for (index, word) in words_le4_even(&data_with_zero).iter().enumerate() {
        let term = word.wrapping_mul(if index % 2 == 0 { k_a } else { k_b });
        let offset = rol16(*word).wrapping_mul(if index % 2 == 0 { 0xEF0569FB } else { 0xC31713DB });
        terms.push(term);
        offsets.push(offset);
    }

    terms[14] = terms[14].wrapping_add(CS64_ADJUSTMENT0.wrapping_mul(factors[0]));
    offsets[14] = offsets[14].wrapping_add(CS64_ADJUSTMENT1);

    let mut swap_acc = 0u32;
    let mut swap_last = 0u32;
    let mut state = 0u32;
    let mut acc = 0u32;

    for index in (0..terms.len()).step_by(2) {
        let x = terms[index];
        let y = terms[index + 1];

        let t1_val = rol16(offsets[index].wrapping_add(x));
        let q1 = t1_val.wrapping_mul(0x76F50F34);
        swap_acc = swap_acc.wrapping_add(rol16(q1).wrapping_mul(0xC3EFEA97).wrapping_add(q1));

        let t2_val = rol16(offsets[index + 1].wrapping_add(y));
        let q2 = t2_val.wrapping_mul(0x1881CE3C);
        swap_last = rol16(q2).wrapping_mul(0x35BD1EC9).wrapping_add(q2);
        swap_acc = swap_acc.wrapping_add(swap_last);

        state = state.wrapping_mul(k_a).wrapping_add(x);
        state = rol16(state).wrapping_mul(0xCF98B111);
        state = rol16(state).wrapping_mul(0x87085B9F);
        state = rol16(state).wrapping_mul(0x12CEB96D);
        state = rol16(state).wrapping_mul(0x257E1D83);
        acc = acc.wrapping_add(state);

        let t3 = state.wrapping_mul(k_b).wrapping_add(y);
        let t4 = rol16(t3).wrapping_mul(0xA27416F5);
        let t5 = rol16(t4).wrapping_mul(0xD38396FF);
        state = rol16(rol16(t5).wrapping_mul(0x7C932B89)).wrapping_mul(0xBFA49F69);
        acc = acc.wrapping_add(state);
    }

    (swap_last ^ state, swap_acc ^ acc)
}

pub fn generate_sec_ms_gec_v2(unix_timestamp: Option<f64>) -> String {
    let ts = unix_timestamp.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    });

    let mut ticks = ts + WIN_EPOCH as f64;
    ticks -= ticks % 300.0;
    ticks *= 1e9 / 100.0;

    let str_to_hash = format!("{:.0}{}", ticks, TRUSTED_CLIENT_TOKEN);
    let digest = sha256_digest(str_to_hash.as_bytes());
    let (cs_a, cs_b) = cs64_digest(str_to_hash.as_bytes(), &digest);

    let mut cs64_bytes = [0u8; 8];
    cs64_bytes[0..4].copy_from_slice(&cs_a.to_le_bytes());
    cs64_bytes[4..8].copy_from_slice(&cs_b.to_le_bytes());

    let cs64_hex = cs64_bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>();
    let sha256_hex = digest.iter().map(|b| format!("{:02X}", b)).collect::<String>();

    format!("{}-{}", cs64_hex, sha256_hex)
}

/// Synthesize audio using Microsoft Edge Neural TTS WebSocket (Sec-MS-GEC v2 authenticated)
async fn synthesize_edge_tts(text: &str, voice: &str) -> Result<Vec<u8>, String> {
    let connection_id = generate_uuid();
    let request_id = generate_uuid();
    let sec_ms_gec = generate_sec_ms_gec_v2(None);

    let url_str = format!(
        "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken={}&Sec-MS-GEC={}&Sec-MS-GEC-Version={}&ConnectionId={}",
        TRUSTED_CLIENT_TOKEN, sec_ms_gec, SEC_MS_GEC_VERSION, connection_id
    );

    let mut req = url_str.into_client_request().map_err(|e| e.to_string())?;
    req.headers_mut().insert("Pragma", "no-cache".parse().unwrap());
    req.headers_mut().insert("Cache-Control", "no-cache".parse().unwrap());
    req.headers_mut().insert("Sec-WebSocket-Version", "13".parse().unwrap());
    req.headers_mut().insert(
        "User-Agent",
        format!(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 Edg/{}.0.0.0",
            CHROMIUM_MAJOR_VERSION, CHROMIUM_MAJOR_VERSION
        )
        .parse()
        .unwrap(),
    );
    req.headers_mut().insert(
        "Origin",
        "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold"
            .parse()
            .unwrap(),
    );
    req.headers_mut().insert(
        "Accept-Encoding",
        "gzip, deflate, br, zstd".parse().unwrap(),
    );
    req.headers_mut().insert(
        "Accept-Language",
        "en-US,en;q=0.9".parse().unwrap(),
    );

    let (ws_stream, _) = tokio::time::timeout(Duration::from_secs(6), connect_async(req))
        .await
        .map_err(|_| "TTS WebSocket connection timeout (6s)".to_string())?
        .map_err(|e| format!("TTS WebSocket connection error: {}", e))?;

    let (mut write, mut read) = ws_stream.split();

    // 1. Send speech.config
    let config_msg = "Content-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{\"context\":{\"synthesis\":{\"audio\":{\"metadataoptions\":{\"sentenceBoundaryEnabled\":\"false\",\"wordBoundaryEnabled\":\"false\"},\"outputFormat\":\"audio-24khz-48kbitrate-mono-mp3\"}}}}";
    write.send(Message::Text(config_msg.to_string()))
        .await
        .map_err(|e| format!("Failed to send TTS speech.config: {}", e))?;

    // 2. Send SSML request
    let lang = if voice.starts_with("en-") { "en-US" } else { "th-TH" };
    let ssml = format!(
        "<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='{}'><voice name='{}'><prosody pitch='+0Hz' rate='+0%' volume='+0%'>{}</prosody></voice></speak>",
        lang,
        voice,
        xml_escape(text)
    );
    let ssml_msg = format!(
        "X-RequestId:{}\r\nContent-Type:application/ssml+xml\r\nPath:ssml\r\n\r\n{}",
        request_id, ssml
    );
    write.send(Message::Text(ssml_msg))
        .await
        .map_err(|e| format!("Failed to send TTS SSML: {}", e))?;

    // 3. Receive audio binary chunks
    let mut audio_bytes = Vec::new();

    let timeout_duration = Duration::from_secs(10);
    let collect_res = tokio::time::timeout(timeout_duration, async {
        while let Some(msg_res) = read.next().await {
            match msg_res {
                Ok(Message::Binary(bin)) => {
                    if bin.len() >= 2 {
                        let header_len = u16::from_be_bytes([bin[0], bin[1]]) as usize;
                        if bin.len() >= 2 + header_len {
                            audio_bytes.extend_from_slice(&bin[2 + header_len..]);
                        }
                    }
                }
                Ok(Message::Text(txt)) => {
                    if txt.contains("Path:turn.end") {
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => return Err(format!("TTS stream read error: {}", e)),
                _ => {}
            }
        }
        Ok(())
    })
    .await;

    match collect_res {
        Ok(Ok(())) => {
            if audio_bytes.is_empty() {
                Err("Empty audio stream received from Edge TTS".to_string())
            } else {
                Ok(audio_bytes)
            }
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("TTS synthesis timeout (10s)".to_string()),
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn generate_uuid() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

fn md5_hash(data: &[u8]) -> u128 {
    let mut hash = 0u128;
    for &b in data {
        hash = hash.wrapping_mul(31).wrapping_add(b as u128);
    }
    hash
}

/// Fallback voice synthesis using Cloud TTS (Google Translate Voice API, zero-auth, fallback only)
async fn synthesize_cloud_tts(text: &str, voice: &str) -> Result<Vec<u8>, String> {
    let lang = if voice.starts_with("en") { "en" } else { "th" };
    let encoded_text = urlencoding_simple(text);
    let url = format!(
        "https://translate.google.com/translate_tts?ie=UTF-8&q={}&tl={}&client=tw-ob",
        encoded_text, lang
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Failed to build HTTP client for TTS: {}", e))?;

    let res = client.get(&url)
        .header("Referer", "https://translate.google.com/")
        .send()
        .await
        .map_err(|e| format!("Cloud TTS HTTP request error: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Cloud TTS returned HTTP status: {}", res.status()));
    }

    let bytes = res.bytes().await.map_err(|e| format!("Failed to read TTS audio bytes: {}", e))?;
    if bytes.is_empty() {
        return Err("Empty audio response from Cloud TTS".to_string());
    }

    Ok(bytes.to_vec())
}

fn urlencoding_simple(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.as_bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*b as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_cache_path_and_hash() {
        let temp_dir = std::env::temp_dir().join("nhk_tts_test");
        let mgr = TtsManager::new(&temp_dir);
        let path1 = mgr.get_cache_path("สวัสดีครับ", "th-TH-PremwadeeNeural");
        let path2 = mgr.get_cache_path("สวัสดีครับ", "th-TH-PremwadeeNeural");
        let path3 = mgr.get_cache_path("สวัสดีครับ", "th-TH-NiwatNeural");
        assert_eq!(path1, path2);
        assert_ne!(path1, path3, "Male and Female cache paths must differ!");
        assert!(path1.to_string_lossy().ends_with(".mp3"));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("5 > 3 & 2 < 4"), "5 &gt; 3 &amp; 2 &lt; 4");
    }

    #[test]
    fn test_urlencoding_simple() {
        assert_eq!(urlencoding_simple("hello world"), "hello%20world");
        assert!(urlencoding_simple("เทส").contains("%E0%B9%80%E0%B8%97%E0%B8%AA"));
    }

    #[test]
    fn test_sec_ms_gec_v2_vector() {
        let token = generate_sec_ms_gec_v2(Some(1729000000.0));
        let expected = "0320C1824EFA1E37-5D9752F2CE07FB361D5CBA387109ACDAA0DFBC93C80E04FBAF7CA32731B95A16";
        assert_eq!(token, expected, "Sec-MS-GEC v2 test vector mismatch!");
    }

    #[tokio::test]
    async fn test_cloud_tts_live_connection() {
        let res = synthesize_cloud_tts("เทส", "th-TH-PremwadeeNeural").await;
        assert!(res.is_ok(), "Live Cloud TTS synthesis failed: {:?}", res.err());
        assert!(!res.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_edge_tts_live_synthesis() {
        // Test real Microsoft Edge Neural TTS with Niwat (Male) voice
        let res = synthesize_edge_tts("เทส", "th-TH-NiwatNeural").await;
        assert!(res.is_ok(), "Edge Neural TTS synthesis failed: {:?}", res.err());
        let bytes = res.unwrap();
        assert!(bytes.len() > 1000, "Audio stream too short: {} bytes", bytes.len());
    }
}
