// Windows.Media.Ocr: lines of text with word boxes, in the image's pixel space.

use super::capture::Image;
use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Security::Cryptography::CryptographicBuffer;

pub struct Word {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub struct Line {
    pub text: String,
    pub words: Vec<Word>,
}

impl Line {
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        bounds_of(&self.words)
    }
}

pub fn bounds_of(words: &[Word]) -> (f64, f64, f64, f64) {
    if words.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let x0 = words.iter().map(|w| w.x).fold(f64::MAX, f64::min);
    let y0 = words.iter().map(|w| w.y).fold(f64::MAX, f64::min);
    let x1 = words.iter().map(|w| w.x + w.w).fold(f64::MIN, f64::max);
    let y1 = words.iter().map(|w| w.y + w.h).fold(f64::MIN, f64::max);
    (x0, y0, x1 - x0, y1 - y0)
}

pub fn available_languages() -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(list) = OcrEngine::AvailableRecognizerLanguages() {
        if let Ok(n) = list.Size() {
            for i in 0..n {
                if let Ok(l) = list.GetAt(i) {
                    if let Ok(t) = l.LanguageTag() {
                        v.push(t.to_string());
                    }
                }
            }
        }
    }
    v
}

fn engine(languages: &str) -> Result<OcrEngine, String> {
    let wanted: Vec<String> = languages.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    if wanted.is_empty() {
        return OcrEngine::TryCreateFromUserProfileLanguages().map_err(|_| {
            format!("Text recognition is not available: no OCR language is installed for this user's languages. Installed: {}. Add one with Settings > Time & language > Language > (language) > Optical character recognition, or 'Add-WindowsCapability -Online -Name Language.OCR~~~en-US~0.0.1.0'", available_languages().join(", "))
        });
    }
    for tag in &wanted {
        if let Ok(lang) = Language::CreateLanguage(&HSTRING::from(tag.as_str())) {
            if OcrEngine::IsLanguageSupported(&lang).unwrap_or(false) {
                if let Ok(e) = OcrEngine::TryCreateFromLanguage(&lang) {
                    return Ok(e);
                }
            }
        }
    }
    Err(format!("Unsupported OCR language '{}'. Supported: {}", wanted.join(","), available_languages().join(", ")))
}

pub fn recognize(img: &Image, languages: &str) -> Result<(Vec<Line>, String), String> {
    let eng = engine(languages)?;
    let lang = eng.RecognizerLanguage().and_then(|l| l.LanguageTag()).map(|t| t.to_string()).unwrap_or_default();
    // Windows OCR has a maximum image dimension; scale down if needed and map back.
    let max = OcrEngine::MaxImageDimension().unwrap_or(10000).max(1) as u32;
    let (src, factor) = if img.width > max || img.height > max {
        let f = (max as f64 / img.width.max(img.height) as f64).min(1.0);
        (img.resize_to_width(((img.width as f64) * f) as u32), f)
    } else {
        (img.clone(), 1.0)
    };
    let buf = CryptographicBuffer::CreateFromByteArray(&src.bgra).map_err(|e| format!("Text recognition failed: {}", e.message()))?;
    let bmp = SoftwareBitmap::CreateCopyFromBuffer(&buf, BitmapPixelFormat::Bgra8, src.width as i32, src.height as i32).map_err(|e| format!("Text recognition failed: {}", e.message()))?;
    let result = eng.RecognizeAsync(&bmp).map_err(|e| format!("Text recognition failed: {}", e.message()))?.join().map_err(|e| format!("Text recognition failed: {}", e.message()))?;
    let mut lines = Vec::new();
    let ls = result.Lines().map_err(|e| e.message())?;
    for i in 0..ls.Size().unwrap_or(0) {
        let l = match ls.GetAt(i) {
            Ok(l) => l,
            Err(_) => continue,
        };
        let mut words = Vec::new();
        if let Ok(ws) = l.Words() {
            for j in 0..ws.Size().unwrap_or(0) {
                if let Ok(w) = ws.GetAt(j) {
                    let r = w.BoundingRect().unwrap_or_default();
                    words.push(Word {
                        text: w.Text().map(|t| t.to_string()).unwrap_or_default(),
                        x: r.X as f64 / factor,
                        y: r.Y as f64 / factor,
                        w: r.Width as f64 / factor,
                        h: r.Height as f64 / factor,
                    });
                }
            }
        }
        let text = l.Text().map(|t| t.to_string()).unwrap_or_else(|_| words.iter().map(|w| w.text.clone()).collect::<Vec<_>>().join(" "));
        lines.push(Line { text, words });
    }
    Ok((lines, lang))
}

/// Words of a line that cover the (loose) match of `needle`, or None.
pub fn match_words<'a>(line: &'a Line, needle: &str) -> Option<Vec<&'a Word>> {
    let n = crate::common::normalize_loose(needle);
    if n.is_empty() {
        return None;
    }
    // try contiguous word spans, shortest first
    let k = line.words.len();
    for len in 1..=k {
        for s in 0..=(k - len) {
            let joined: String = line.words[s..s + len].iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" ");
            if crate::common::normalize_loose(&joined).contains(&n) {
                return Some(line.words[s..s + len].iter().collect());
            }
        }
    }
    None
}
