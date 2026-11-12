#![forbid(unsafe_code)]
//! Minimal reqwest probe against the local Ollama endpoint (diagnostic).

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let urls = [
            "http://127.0.0.1:11434/api/tags",
            "http://localhost:11434/api/tags",
        ];
        for url in urls {
            match reqwest::Client::new().get(url).send().await {
                Ok(r) => println!("GET {url} -> HTTP {}", r.status()),
                Err(e) => {
                    println!("GET {url} -> ERR {e}");
                    let mut src = std::error::Error::source(&e);
                    while let Some(s) = src {
                        println!("   caused by: {s}");
                        src = s.source();
                    }
                }
            }
        }
    });
}
