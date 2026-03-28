use axum::{extract::State, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

struct AppState { start_time: Instant, stats: Mutex<Stats> }
struct Stats { total_recognitions: u64, total_batch_jobs: u64, total_pages_processed: u64, total_chars_extracted: u64 }

#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_ops: u64 }

#[derive(Deserialize)]
struct RecognizeRequest { image_b64: String, language: Option<String>, detect_orientation: Option<bool>, extract_tables: Option<bool>, template_id: Option<String> }
#[derive(Serialize)]
struct BoundingBox { x: f32, y: f32, w: f32, h: f32 }
#[derive(Serialize)]
struct TextLine { text: String, confidence: f32, bbox: BoundingBox }
#[derive(Serialize)]
struct RecognizeResponse { job_id: String, language: String, text: String, lines: Vec<TextLine>, page_count: u32, char_count: u32, orientation_deg: i32, processing_ms: u128 }

#[derive(Deserialize)]
struct BatchRequest { images_b64: Vec<String>, language: Option<String>, template_id: Option<String> }
#[derive(Serialize)]
struct BatchResponse { batch_id: String, job_count: u32, status: String, estimated_ms: u64 }

#[derive(Serialize)]
struct TemplateInfo { id: String, name: String, description: String, fields: Vec<String>, languages: Vec<String> }

#[derive(Serialize)]
struct LanguageInfo { code: String, name: String, script: String, vertical: bool }

#[derive(Serialize)]
struct StatsResponse { total_recognitions: u64, total_batch_jobs: u64, total_pages_processed: u64, total_chars_extracted: u64 }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "ocr_engine=info".into())).init();
    let state = Arc::new(AppState { start_time: Instant::now(), stats: Mutex::new(Stats { total_recognitions: 0, total_batch_jobs: 0, total_pages_processed: 0, total_chars_extracted: 0 }) });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/ocr/recognize", post(recognize))
        .route("/api/v1/ocr/batch", post(batch))
        .route("/api/v1/ocr/templates", get(templates))
        .route("/api/v1/ocr/languages", get(languages))
        .route("/api/v1/ocr/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("OCR_ADDR").unwrap_or_else(|_| "0.0.0.0:8113".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("OCR Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health { status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(), uptime_secs: s.start_time.elapsed().as_secs(), total_ops: st.total_recognitions + st.total_batch_jobs })
}

async fn recognize(State(s): State<Arc<AppState>>, Json(req): Json<RecognizeRequest>) -> Json<RecognizeResponse> {
    let t = Instant::now();
    let lang = req.language.unwrap_or_else(|| "ja".into());
    let text = format!("OCR result from {} bytes image", req.image_b64.len());
    let char_count = text.chars().count() as u32;
    let lines = vec![
        TextLine { text: text.clone(), confidence: 0.98, bbox: BoundingBox { x: 10.0, y: 10.0, w: 500.0, h: 24.0 } },
    ];
    { let mut st = s.stats.lock().unwrap(); st.total_recognitions += 1; st.total_pages_processed += 1; st.total_chars_extracted += char_count as u64; }
    Json(RecognizeResponse { job_id: uuid::Uuid::new_v4().to_string(), language: lang, text, lines, page_count: 1, char_count, orientation_deg: 0, processing_ms: t.elapsed().as_millis() })
}

async fn batch(State(s): State<Arc<AppState>>, Json(req): Json<BatchRequest>) -> Json<BatchResponse> {
    let count = req.images_b64.len() as u32;
    { let mut st = s.stats.lock().unwrap(); st.total_batch_jobs += 1; }
    Json(BatchResponse { batch_id: uuid::Uuid::new_v4().to_string(), job_count: count, status: "queued".into(), estimated_ms: (count as u64) * 150 })
}

async fn templates() -> Json<Vec<TemplateInfo>> {
    Json(vec![
        TemplateInfo { id: "invoice-jp".into(), name: "Japanese Invoice".into(), description: "Extract fields from standard JP invoices".into(), fields: vec!["invoice_no".into(), "date".into(), "total".into(), "vendor".into()], languages: vec!["ja".into()] },
        TemplateInfo { id: "id-card".into(), name: "ID Card".into(), description: "Extract fields from ID cards".into(), fields: vec!["name".into(), "dob".into(), "id_number".into(), "expiry".into()], languages: vec!["ja".into(), "en".into()] },
        TemplateInfo { id: "receipt".into(), name: "Receipt".into(), description: "Extract line items from receipts".into(), fields: vec!["store".into(), "date".into(), "items".into(), "total".into(), "tax".into()], languages: vec!["ja".into(), "en".into()] },
    ])
}

async fn languages() -> Json<Vec<LanguageInfo>> {
    Json(vec![
        LanguageInfo { code: "ja".into(), name: "Japanese".into(), script: "Kanji/Kana".into(), vertical: true },
        LanguageInfo { code: "en".into(), name: "English".into(), script: "Latin".into(), vertical: false },
        LanguageInfo { code: "zh-hans".into(), name: "Chinese Simplified".into(), script: "CJK".into(), vertical: true },
        LanguageInfo { code: "ko".into(), name: "Korean".into(), script: "Hangul".into(), vertical: false },
        LanguageInfo { code: "ar".into(), name: "Arabic".into(), script: "Arabic".into(), vertical: false },
    ])
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse { total_recognitions: st.total_recognitions, total_batch_jobs: st.total_batch_jobs, total_pages_processed: st.total_pages_processed, total_chars_extracted: st.total_chars_extracted })
}
