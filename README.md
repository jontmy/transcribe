# Transcribe

### Pre-requisites
You must have Rust installed on your machine and an OpenAI API key.

### Usage
```bash
git clone https://github.com/jontmy/transcribe.git
cd transcribe
cargo run --release -- <url> -o <output> -k <api_key>
```

### Limitations
- Only supports YouTube videos with English audio tracks.
- The maximum audio file size is 25 MB (fails fast if larger than 25 MB).