#![forbid(unsafe_code)]
//! AuxBridge 桌面套件（`auxbridge-web`）之可測試核心邏輯。
//!
//! 目前僅含更新鏈 feed（latest.json）之簽章驗證與清單組裝，供
//! `src/bin/make_latest_json.rs`（產生器）與 `tests/updater_artifact.rs`
//! （驗收閘）共用，確保「產生」與「驗收」走同一份實作（單一事實來源）。

pub mod feed;
