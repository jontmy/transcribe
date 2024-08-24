use dotenvy::dotenv;
use rs_openai::{
    audio::{AudioModel, CreateTranscriptionRequestBuilder, Language, ResponseFormat},
    shared::types::FileMeta,
    OpenAI,
};
use std::env::var;
use youtube_dl::YoutubeDl;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = var("OPENAI_API_KEY").unwrap();

    let output = YoutubeDl::new("https://www.youtube.com/watch?v=FJVFXsNzYZQ")
        .run_async()
        .await
        .unwrap();

    let video = output.into_single_video().unwrap();
    let audio = video
        .formats
        .unwrap()
        .into_iter()
        .find(|f| f.ext.as_ref().map_or(false, |ext| ext == "m4a"))
        .unwrap()
        .url
        .unwrap();

    let audio = reqwest::get(audio).await.unwrap().bytes().await.unwrap();
    let openai = OpenAI::new(&OpenAI {
        api_key,
        org_id: None,
    });
    let req = CreateTranscriptionRequestBuilder::default()
        .model(AudioModel::Whisper1)
        .language(Language::English)
        .response_format(ResponseFormat::Text)
        .temperature(0.0)
        .file(FileMeta {
            buffer: audio.to_vec(),
            filename: "audio.m4a".to_string(),
        })
        .build()
        .unwrap();

    let res = openai
        .audio()
        .create_transcription_with_text_response(&req)
        .await
        .unwrap();

    dbg!(res);
}
