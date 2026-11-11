#![forbid(unsafe_code)]
//! OCR 轉接器（V2.5；D-OCR1 v1.2）：**內嵌 PaddleOCR（ONNX）**——中文＋英文必備。
//!
//! 使用者裁決（2026-11-10）：零外部執行檔；精度趨近 100%（乾淨列印 ~97-99%，
//! 殘差由置信度閘＋人工確認吸收——fail-closed 既有架構）。
//! 引擎：`paddleocr_rs_onnx`（MIT、Rust 1.92+、純 Rust 推論、無外部執行階段程式庫）。
//! 模型三件套（RapidOCR 模型庫，Apache-2.0）：det/rec ONNX＋ppocr_keys_v1 字典。
//! 模型路徑：erp_config 鍵 `ocr.models_dir` → exe 同目錄 models/ocr/（與伺服器
//! 「絕對路徑優先」解析序同構）。引擎冪等快取（模組級 OnceLock）。

use sqlx::SqlitePool;

pub use paddleocr_rs_onnx::OcrEngine;

/// OCR 引擎可用性（derive；每次查詢重新評估檔案存在性）。
#[derive(Debug, Clone)]
pub struct OcrAvailability {
    pub available: bool,
    pub models_dir: Option<String>,
    pub missing: Vec<String>,
    pub note: String,
}

const MODEL_FILES: [&str; 3] = [
    "ch_PP-OCRv4_det_infer.onnx",
    "ch_PP-OCRv4_rec_infer.onnx",
    "ppocr_keys_v1.txt",
];

/// 置信度閘（D-OCR3）：行平均分數低於此值 → 該列轉 error（低置信不冒充合格）。
pub const CONFIDENCE_GATE: f64 = 0.60;

/// 模型目錄解析序：erp_config 鍵 `ocr.models_dir` → exe 同目錄 models/ocr/。
async fn models_dir(pool: &SqlitePool) -> Option<std::path::PathBuf> {
    let (value,): (String,) = sqlx::query_as(
        "SELECT config_value FROM erp_config WHERE config_category = 'ocr' AND config_key = 'models_dir'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(trimmed))
    }
}

fn default_models_dir() -> Option<std::path::PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(std::path::Path::to_path_buf))?;
    let dir = exe_dir.join("models").join("ocr");
    MODEL_FILES
        .iter()
        .all(|f| dir.join(f).exists())
        .then_some(dir)
}

/// 探測模型三件套是否齊備（derive；管理頁／匯入流程／net 面板共用）。
pub async fn availability(pool: &SqlitePool) -> OcrAvailability {
    let dir = match models_dir(pool).await.or_else(default_models_dir) {
        Some(d) => d,
        None => {
            return OcrAvailability {
                available: false,
                models_dir: None,
                missing: MODEL_FILES.iter().map(|s| s.to_string()).collect(),
                note: "OCR 模型未部署（models/ocr/ 缺 det/rec/keys）；處理方式：由管理員放置模型檔或設定 ocr.models_dir"
                    .to_string(),
            }
        }
    };
    let missing: Vec<String> = MODEL_FILES
        .iter()
        .filter(|f| !dir.join(f).exists())
        .map(|s| s.to_string())
        .collect();
    if missing.is_empty() {
        OcrAvailability {
            available: true,
            models_dir: Some(dir.to_string_lossy().to_string()),
            missing: Vec::new(),
            note: "PP-OCRv4 模型齊備（中文＋英文）".to_string(),
        }
    } else {
        OcrAvailability {
            available: false,
            models_dir: Some(dir.to_string_lossy().to_string()),
            missing: missing.clone(),
            note: format!(
                "模型檔缺失：{}；處理方式：補齊檔案或更正 ocr.models_dir 設定",
                missing.join("、")
            ),
        }
    }
}

/// 引擎冪等快取（模組級；OcrEngine 為 Send+Sync——ORT session 執行緒安全）。
static ENGINE: std::sync::OnceLock<paddleocr_rs_onnx::OcrEngine> = std::sync::OnceLock::new();

/// 取得或初始化引擎（首次載入模型 bytes；失敗不快取——下輪重試）。
pub fn get_or_init_engine(
    models_dir: &std::path::Path,
) -> Result<&'static paddleocr_rs_onnx::OcrEngine, String> {
    if let Some(engine) = ENGINE.get() {
        return Ok(engine);
    }
    let det = std::fs::read(models_dir.join(MODEL_FILES[0]))
        .map_err(|e| format!("det 模型讀取失敗：{e}"))?;
    let rec = std::fs::read(models_dir.join(MODEL_FILES[1]))
        .map_err(|e| format!("rec 模型讀取失敗：{e}"))?;
    let keys = std::fs::read(models_dir.join(MODEL_FILES[2]))
        .map_err(|e| format!("字典檔讀取失敗：{e}"))?;
    let engine = paddleocr_rs_onnx::OcrEngine::new(&det, &rec, &keys)
        .map_err(|e| format!("OCR 引擎初始化失敗：{e}"))?;
    let _ = ENGINE.set(engine);
    Ok(ENGINE.get().expect("engine initialized exactly once"))
}

/// 圖片 bytes → 行（文字＋辨識分數 0.0-1.0）。呼叫端先以 [`availability`] 探測。
pub fn image_to_lines(
    engine: &paddleocr_rs_onnx::OcrEngine,
    image_bytes: &[u8],
) -> Result<Vec<(String, f64)>, String> {
    let img = image::load_from_memory(image_bytes).map_err(|e| format!("影像解碼失敗：{e}"))?;
    let blocks = engine
        .recognize_all(&img, paddleocr_rs_onnx::OrderBy::Horizontal)
        .map_err(|e| format!("OCR 推論失敗：{e}"))?;
    Ok(blocks
        .into_iter()
        .map(|b| (b.text, f64::from(b.confidence)))
        .collect())
}
