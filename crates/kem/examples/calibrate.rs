#![forbid(unsafe_code)]
//! Cosine calibration probe: refusal queries vs false-hit claim bodies.
//! Prints the raw-cosine matrix used to set REFUSAL_FLOOR.

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
    let queries = [
        "kubernetes autoscaling cluster",
        "blockchain consensus mechanism",
        "quantum qubit error correction",
        "photosynthesis chlorophyll light",
    ];
    let claims = [
        ("claim-011 ollama/qwen", "Local Ollama has qwen3.5:4b installed (4.2B params, Q4_K_M, 262K context, 3.3 GB) and serves it locally."),
        ("claim-012 routing", "The AI fallback chain uses local Ollama (qwen3.5:4b) as PRIMARY, with keyless remote endpoints only as fallback."),
        ("claim-017 scs", "The self-consistency voting evidence baseline is 90.8% to 99.2% agreement (k=15, probability floor 0.60, automation 74.2%) per the JEV cookbook."),
    ];
    let mut texts: Vec<String> = queries.iter().map(|s| s.to_string()).collect();
    texts.extend(claims.iter().map(|(_, b)| b.to_string()));
    let pool = kem::embed::default_pool();
    match kem::embed::embed_batch(&pool, &texts).await {
        Ok(res) => {
            let vecs = &res.vecs;
            println!(
                "embed ok: {} vectors (via {} / {}), dim {}",
                vecs.len(),
                res.used_label,
                res.used_model,
                vecs[0].len()
            );
            for (qi, q) in queries.iter().enumerate() {
                let qv = &vecs[qi];
                let row: Vec<String> = claims
                    .iter()
                    .enumerate()
                    .map(|(ci, (name, _))| {
                        format!("{}={:.4}", name, kem::embed::cosine(qv, &vecs[4 + ci]))
                    })
                    .collect();
                println!("{q}: {}", row.join("  "));
            }
        }
        Err(e) => println!("embed failed: {e}"),
        }
    });
}
