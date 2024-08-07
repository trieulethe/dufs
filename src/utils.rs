use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
#[cfg(feature = "tls")]
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::error::Error;
use std::{
    borrow::Cow,
    fs::File,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use url::Url;
use reqwest::Client;
use std::io::{self, BufRead, BufReader, Write};

// Function to rewrite the M3U8 file with local segment paths
pub fn write_m3u8(content: &str, output_file: &str) -> io::Result<()> {
    let reader = BufReader::new(content.as_bytes());
    let mut output = File::create(output_file)?;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("#") {
            writeln!(output, "{}", line)?;
        } else if line.trim().is_empty() {
            writeln!(output)?;
        } else {
            let file_name = line.split('/').last().unwrap_or(&line);
            writeln!(output, "{}", file_name)?;
        }
    }
    Ok(())
}

// Function to download the M3U8 file and parse its segments
pub async fn download_m3u8(url: &str) -> Result<(String, Vec<String>)> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let content = response.text().await?;
    println!("M3U8 Content:\n{}", content);

    // Create base URL from M3U8 URL
    let base_url = if url.ends_with(".m3u8") {
        url.rsplitn(2, '/').nth(1).unwrap_or("")
    } else {
        url.rsplitn(2, '/').nth(0).unwrap_or("")
    };

    // Collect all segment URLs
    let segments: Vec<String> = content
        .lines()
        .filter(|line| !line.starts_with("#") && !line.trim().is_empty())
        .map(|line| {
            let segment_url = if line.starts_with("http") {
                line.to_string()
            } else {
                format!("{}/{}", base_url, line)
            };
            println!("Segment URL: {}", segment_url);
            segment_url
        })
        .collect();

    Ok((content, segments))
}

// Function to download an individual segment
pub async fn download_segment(client: &Client, segment_url: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    println!("Downloading segment: {}", segment_url);
    for attempt in 0..5 {
        let result = tokio::time::timeout(Duration::from_secs(15), client.get(segment_url).send()).await;
        match result {
            Ok(Ok(res)) => {
                if res.status().is_success() {
                    let bytes = res.bytes().await?;
                    println!("DONE segment: {}", segment_url);
                    return Ok(bytes.to_vec());
                } else {
                    eprintln!("Failed to download segment {}: HTTP {}", segment_url, res.status());
                }
            }
            Ok(Err(err)) => {
                eprintln!("Error downloading {}: {}", segment_url, err);
            }
            Err(_) => {
                eprintln!("Timeout downloading segment {}", segment_url);
            }
        }

        if attempt < 4 {
            // Delay before retrying
            println!("Retry download: {}", segment_url);
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    Err(format!("Failed to download segment {} after 5 attempts", segment_url).into())
}

pub fn gen_html_no_poster() -> String {
        format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="robots" content="noindex">
            <title>Live CDN</title>
            <link href="https://cdnjs.cloudflare.com/ajax/libs/video.js/8.3.0/video-js.min.css" rel="stylesheet">
        </head>
        <body style="background:#000;color:#fff;">
            <script src="https://cdnjs.cloudflare.com/ajax/libs/video.js/8.3.0/video.min.js"></script>
            <div class="video-container">
                <video 
                    id="my-player"
                    class="video-js"
                    muted 
                    controls 
                    preload="auto"
                    data-setup='{}'
                    style="position:fixed;right:0;bottom:0;min-width:100%;max-width:100%;min-height:100%;max-height:100%;object-fit:fill;"
                >
                    <source id="video-source" type="application/x-mpegURL"></source>
                    <p class="vjs-no-js">
                        To view this video please enable JavaScript, and consider upgrading to a
                        web browser that
                        <a href="https://videojs.com/html5-video-support/" target="_blank">
                            supports HTML5 video
                        </a>
                    </p>
                </video>
            </div>
            <script>
                let url = window.location.href;
                let hls_url = url.replace("/html", "/hls");
                hls_url = url.replace(".html", ".m3u8");  
                if (url){}
                    document.getElementById('video-source').src = hls_url;
                    let video = document.getElementById('my-player')
                {} else {}
                {}
                const player = videojs('my-player');
            </script>
        </body>
        </html>
    "#,
        "{}", "{", "}", "{", "}"
    )
}

pub fn gen_html_hls() -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="robots" content="noindex">
            <title>Live CDN</title>
            <link href="https://cdnjs.cloudflare.com/ajax/libs/video.js/8.3.0/video-js.min.css" rel="stylesheet">
        </head>
        <body style="background:#000;color:#fff;">
            <script src="https://cdnjs.cloudflare.com/ajax/libs/video.js/8.3.0/video.min.js"></script>
            <div class="video-container">
                <video 
                    id="my-player"
                    class="video-js"
                    muted 
                    controls 
                    preload="auto"
                    poster=""
                    data-setup='{}'
                    style="position:fixed;right:0;bottom:0;min-width:100%;max-width:100%;min-height:100%;max-height:100%;object-fit:fill;"
                >
                    <source id="video-source" type="application/x-mpegURL"></source>
                    <p class="vjs-no-js">
                        To view this video please enable JavaScript, and consider upgrading to a
                        web browser that
                        <a href="https://videojs.com/html5-video-support/" target="_blank">
                            supports HTML5 video
                        </a>
                    </p>
                </video>
            </div>
            <script>
                let url = window.location.href;
                let hls_url = url.replace("/html", "/hls");
                hls_url = url.replace(".html", ".m3u8");  
                let poster = url.replace("/html", "/image");
                poster = url.replace("index.html", "thumb3.webp");
                if (url){}
                    document.getElementById('video-source').src = hls_url;
                    let video = document.getElementById('my-player')
                    video.setAttribute('poster', poster);
                {} else {}
                {}
                const player = videojs('my-player');
            </script>
        </body>
        </html>
    "#,
        "{}", "{", "}", "{", "}"
    )
}

pub fn create_html_file(file_path: &str, html_content: &str) {
    // Mở tệp tin để ghi
    let mut file = File::create(file_path).expect("Không thể tạo tệp tin");

    // Ghi nội dung HTML vào tệp tin
    match file.write_all(html_content.as_bytes()) {
        Ok(_) => println!("Tệp tin HTML đã được tạo và ghi thành công."),
        Err(e) => eprintln!("Lỗi khi ghi tệp tin HTML: {}", e),
    }
}

pub fn get_path_from_url(url_str: &str) -> Option<String> {
    // Parse the URL
    if let Ok(url) = Url::parse(url_str) {
        // Extract the path and remove the leading "/"
        let path = url.path().trim_start_matches('/');
        return Some(path.to_string());
    }

    None
}

pub fn check_file_exist(file_path: &str) -> bool {
    std::fs::metadata(file_path).is_ok()
}

pub fn unix_now() -> Result<Duration> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .with_context(|| "Invalid system time")
}

pub fn encode_uri(v: &str) -> String {
    let parts: Vec<_> = v.split('/').map(urlencoding::encode).collect();
    parts.join("/")
}

pub fn decode_uri(v: &str) -> Option<Cow<str>> {
    percent_encoding::percent_decode(v.as_bytes())
        .decode_utf8()
        .ok()
}

pub fn get_file_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
}

#[cfg(unix)]
pub async fn get_file_mtime_and_mode(path: &Path) -> Result<(DateTime<Utc>, u16)> {
    use std::os::unix::prelude::MetadataExt;
    let meta = tokio::fs::metadata(path).await?;
    let datetime: DateTime<Utc> = meta.modified()?.into();
    Ok((datetime, meta.mode() as u16))
}

#[cfg(not(unix))]
pub async fn get_file_mtime_and_mode(path: &Path) -> Result<(DateTime<Utc>, u16)> {
    let meta = tokio::fs::metadata(&path).await?;
    let datetime: DateTime<Utc> = meta.modified()?.into();
    Ok((datetime, 0o644))
}

pub fn try_get_file_name(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| anyhow!("Failed to get file name of `{}`", path.display()))
}

pub fn glob(pattern: &str, target: &str) -> bool {
    let pat = match ::glob::Pattern::new(pattern) {
        Ok(pat) => pat,
        Err(_) => return false,
    };
    pat.matches(target)
}

// Load public certificate from file.
#[cfg(feature = "tls")]
pub fn load_certs<T: AsRef<Path>>(filename: T) -> Result<Vec<CertificateDer<'static>>> {
    // Open certificate file.
    let cert_file = std::fs::File::open(filename.as_ref())
        .with_context(|| format!("Failed to access `{}`", filename.as_ref().display()))?;
    let mut reader = std::io::BufReader::new(cert_file);

    // Load and return certificate.
    let mut certs = vec![];
    for cert in rustls_pemfile::certs(&mut reader) {
        let cert = cert.with_context(|| "Failed to load certificate")?;
        certs.push(cert)
    }
    if certs.is_empty() {
        anyhow::bail!("No supported certificate in file");
    }
    Ok(certs)
}

// Load private key from file.
#[cfg(feature = "tls")]
pub fn load_private_key<T: AsRef<Path>>(filename: T) -> Result<PrivateKeyDer<'static>> {
    let key_file = std::fs::File::open(filename.as_ref())
        .with_context(|| format!("Failed to access `{}`", filename.as_ref().display()))?;
    let mut reader = std::io::BufReader::new(key_file);

    // Load and return a single private key.
    for key in rustls_pemfile::read_all(&mut reader) {
        let key = key.with_context(|| "There was a problem with reading private key")?;
        match key {
            rustls_pemfile::Item::Pkcs1Key(key) => return Ok(PrivateKeyDer::Pkcs1(key)),
            rustls_pemfile::Item::Pkcs8Key(key) => return Ok(PrivateKeyDer::Pkcs8(key)),
            rustls_pemfile::Item::Sec1Key(key) => return Ok(PrivateKeyDer::Sec1(key)),
            _ => {}
        }
    }
    anyhow::bail!("No supported private key in file");
}

pub fn parse_range(range: &str, size: u64) -> Option<(u64, u64)> {
    let (unit, range) = range.split_once('=')?;
    if unit != "bytes" || range.contains(',') {
        return None;
    }
    let (start, end) = range.split_once('-')?;
    if start.is_empty() {
        let offset = end.parse::<u64>().ok()?;
        if offset <= size {
            Some((size - offset, size - 1))
        } else {
            None
        }
    } else {
        let start = start.parse::<u64>().ok()?;
        if start < size {
            if end.is_empty() {
                Some((start, size - 1))
            } else {
                let end = end.parse::<u64>().ok()?;
                if end < size {
                    Some((start, end))
                } else {
                    None
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_key() {
        assert!(glob("", ""));
        assert!(glob(".*", ".git"));
        assert!(glob("abc", "abc"));
        assert!(glob("a*c", "abc"));
        assert!(glob("a?c", "abc"));
        assert!(glob("a*c", "abbc"));
        assert!(glob("*c", "abc"));
        assert!(glob("a*", "abc"));
        assert!(glob("?c", "bc"));
        assert!(glob("a?", "ab"));
        assert!(!glob("abc", "adc"));
        assert!(!glob("abc", "abcd"));
        assert!(!glob("a?c", "abbc"));
        assert!(!glob("*.log", "log"));
        assert!(glob("*.abc-cba", "xyz.abc-cba"));
        assert!(glob("*.abc-cba", "123.xyz.abc-cba"));
        assert!(glob("*.log", ".log"));
        assert!(glob("*.log", "a.log"));
        assert!(glob("*/", "abc/"));
        assert!(!glob("*/", "abc"));
    }

    #[test]
    fn test_parse_range() {
        assert_eq!(parse_range("bytes=0-499", 500), Some((0, 499)));
        assert_eq!(parse_range("bytes=0-", 500), Some((0, 499)));
        assert_eq!(parse_range("bytes=299-", 500), Some((299, 499)));
        assert_eq!(parse_range("bytes=-500", 500), Some((0, 499)));
        assert_eq!(parse_range("bytes=-300", 500), Some((200, 499)));
        assert_eq!(parse_range("bytes=500-", 500), None);
        assert_eq!(parse_range("bytes=-501", 500), None);
        assert_eq!(parse_range("bytes=0-500", 500), None);
    }
}
