use clap::Parser;
use dotenvy::dotenv;
use expanduser::expanduser;
use itertools::Itertools;
use rs_openai::{
    audio::{AudioModel, CreateTranscriptionRequestBuilder, Language, ResponseFormat},
    shared::types::FileMeta,
    OpenAI,
};
use std::{env::var, fs::File, io::Write, process::exit};
use youtube_dl::YoutubeDl;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The URL of the YouTube video to transcribe
    #[arg(name = "URL")]
    url: String,
    /// The OpenAI API key to use for the Whisper V2 model
    #[arg(short = 'k', long = "api-key")]
    api_key: Option<String>,
    /// The path to the output file
    #[arg(short = 'o', long = "output")]
    output_path: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let args = Args::parse();
    let url = args.url;
    let api_key = args
        .api_key
        .unwrap_or(var("OPENAI_API_KEY").expect("Missing API key"));

    let output_file = args
        .output_path
        .map(|path| expanduser(&path).ok())
        .flatten()
        .map(|path| {
            File::create(path)
                .ok()
                .expect("Failed to create output file")
        });

    print!("Fetching video metadata... ");
    std::io::stdout().flush().unwrap();
    let output = YoutubeDl::new(url).run_async().await.unwrap();
    println!("done.");

    let video = output.into_single_video().unwrap();
    let (audio_file_size, audio_url) = video
        .formats
        .expect("Missing video formats")
        .into_iter()
        .filter(|f| f.ext.as_ref().map_or(false, |ext| ext == "m4a"))
        .map(|f| (f.filesize.map(|v| v as f64).or(f.filesize_approx), f.url))
        .filter(|(size, url)| size.is_some() && url.is_some())
        .map(|(size, url)| (size.unwrap(), url.unwrap()))
        .sorted_by(|a, b| f64::total_cmp(&a.0, &b.0))
        .next()
        .expect("No suitable audio tracks found");

    if audio_file_size >= 25.0 * 1000.0 * 1000.0 {
        eprintln!(
            "Audio file is too large to transcribe, max 25 MB, got {:.2} MB",
            audio_file_size / 1000.0 / 1000.0
        );
        exit(1);
    }

    let title = video.title.expect("Missing video title");
    let mut input = String::new();
    print!("Transcribe '{}'? [y/N] ", &title);
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() != "y" {
        return;
    }

    print!("Downloading audio track... ");
    std::io::stdout().flush().unwrap();
    let audio = reqwest::get(audio_url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    println!("done.");

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

    print!("Transcribing... ");
    std::io::stdout().flush().unwrap();
    let res = openai
        .audio()
        .create_transcription_with_text_response(&req)
        .await
        .unwrap();
    println!("done.");

    if let Some(mut file) = output_file {
        file.write_all(res.as_bytes())
            .expect("Failed to write to output file");
    }
    println!("{}", res);
}
