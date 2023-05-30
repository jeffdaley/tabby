use axum::{extract::State, Json};
use ctranslate2_bindings::{
    TextInferenceEngine, TextInferenceEngineCreateOptionsBuilder, TextInferenceOptionsBuilder,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use utoipa::ToSchema;

mod languages;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct CompletionRequest {
    /// https://code.visualstudio.com/docs/languages/identifiers
    #[schema(example = "python")]
    language: Option<String>,

    #[schema(example = "def fib(n):")]
    prompt: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct Choice {
    index: u32,
    text: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct CompletionResponse {
    id: String,
    created: u64,
    choices: Vec<Choice>,
}

#[utoipa::path(
    post,
    path = "/v1/completions",
    request_body = CompletionRequest ,
)]
pub async fn completion(
    State(state): State<Arc<CompletionState>>,
    Json(request): Json<CompletionRequest>,
) -> Json<CompletionResponse> {
    let options = TextInferenceOptionsBuilder::default()
        .max_decoding_length(64)
        .sampling_temperature(0.2)
        .build()
        .unwrap();
    let text = state.engine.inference(&request.prompt, options);
    let language = request.language.unwrap_or("unknown".into());
    let filtered_text = languages::remove_stop_words(&language, &text);

    Json(CompletionResponse {
        id: format!("cmpl-{}", uuid::Uuid::new_v4()),
        created: timestamp(),
        choices: [Choice {
            index: 0,
            text: filtered_text.to_string(),
        }]
        .to_vec(),
    })
}

pub struct CompletionState {
    engine: TextInferenceEngine,
}

impl CompletionState {
    pub fn new(args: &crate::serve::ServeArgs) -> Self {
        let home = std::env::var("HOME").unwrap();
        let tabby_root = format!("{}/.tabby", home);
        let model_dir = Path::new(&tabby_root).join("models").join(&args.model);
        let metadata = read_metadata(&model_dir);

        let device = format!("{}", args.device);
        let options = TextInferenceEngineCreateOptionsBuilder::default()
            .model_path(model_dir.join("ctranslate2").display().to_string())
            .tokenizer_path(model_dir.join("tokenizer.json").display().to_string())
            .device(device)
            .model_type(metadata.transformers_info.auto_model)
            .device_indices(args.device_indices.clone())
            .num_replicas_per_device(args.num_replicas_per_device)
            .build()
            .unwrap();
        let engine = TextInferenceEngine::create(options);
        Self { engine }
    }
}

fn timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metadata {
    transformers_info: TransformersInfo,
}

#[derive(Deserialize)]
struct TransformersInfo {
    auto_model: String,
}

fn read_metadata(model_dir: &std::path::PathBuf) -> Metadata {
    let file = std::fs::File::open(model_dir.join("metadata.json")).unwrap();
    let reader = std::io::BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}