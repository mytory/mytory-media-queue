use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    thread,
};

use mytory_media_queue_lib::{ManagedToolSet, ToolManager};
use sha2::{Digest, Sha256};
use uuid::Uuid;

fn test_directory() -> PathBuf {
    let path = std::env::temp_dir().join(format!("mytory-tool-manager-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn installs_a_verified_downloader_and_extractor_as_one_current_set() {
    let root = test_directory();
    let staging = root.join("staging");
    fs::create_dir_all(&staging).unwrap();
    let downloader = b"yt-dlp 2026.08.19";
    let extractor = b"yt-dlp-ejs 0.8.0";
    fs::write(staging.join("yt-dlp.whl"), downloader).unwrap();
    fs::write(staging.join("yt-dlp-ejs.whl"), extractor).unwrap();

    let manager = ToolManager::open(root.join("tools")).unwrap();
    manager
        .install(ManagedToolSet::new(
            "2026.08.19+ejs.0.8.0",
            staging.join("yt-dlp.whl"),
            sha256(downloader),
            staging.join("yt-dlp-ejs.whl"),
            sha256(extractor),
        ))
        .unwrap();

    let current = manager.current().unwrap().expect("current tool set");
    assert_eq!(current.version, "2026.08.19+ejs.0.8.0");
    assert_eq!(fs::read(current.downloader).unwrap(), downloader);
    assert_eq!(fs::read(current.extractor).unwrap(), extractor);
    assert!(root.join("tools/current/yt-dlp.whl").is_file());
    assert!(root.join("tools/current/yt-dlp-ejs.whl").is_file());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn initializes_only_once_from_the_verified_bundled_set() {
    let root = test_directory();
    let bundled = root.join("bundled");
    fs::create_dir_all(&bundled).unwrap();
    let downloader = bundled.join("yt-dlp.whl");
    let extractor = bundled.join("yt-dlp-ejs.whl");
    fs::write(&downloader, b"bundled downloader").unwrap();
    fs::write(&extractor, b"bundled extractor").unwrap();
    let manager = ToolManager::open(root.join("tools")).unwrap();

    manager
        .initialize_if_missing(ManagedToolSet::new(
            "bundled",
            &downloader,
            sha256(b"bundled downloader"),
            &extractor,
            sha256(b"bundled extractor"),
        ))
        .unwrap();
    fs::write(&downloader, b"replacement downloader").unwrap();
    manager
        .initialize_if_missing(ManagedToolSet::new(
            "replacement",
            &downloader,
            sha256(b"replacement downloader"),
            &extractor,
            sha256(b"bundled extractor"),
        ))
        .unwrap();

    assert_eq!(manager.current().unwrap().unwrap().version, "bundled");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retains_the_previous_verified_set_when_replacing_the_current_set() {
    let root = test_directory();
    let manager = ToolManager::open(root.join("tools")).unwrap();

    for (version, downloader, extractor) in [
        (
            "old",
            b"old downloader".as_slice(),
            b"old extractor".as_slice(),
        ),
        (
            "new",
            b"new downloader".as_slice(),
            b"new extractor".as_slice(),
        ),
    ] {
        let source = root.join(version);
        fs::create_dir_all(&source).unwrap();
        let downloader_path = source.join("yt-dlp.whl");
        let extractor_path = source.join("yt-dlp-ejs.whl");
        fs::write(&downloader_path, downloader).unwrap();
        fs::write(&extractor_path, extractor).unwrap();
        manager
            .install(ManagedToolSet::new(
                version,
                &downloader_path,
                sha256(downloader),
                &extractor_path,
                sha256(extractor),
            ))
            .unwrap();
    }

    let current = manager.current().unwrap().expect("new current set");
    assert_eq!(current.version, "new");
    let previous = root.join("tools/previous");
    assert_eq!(
        fs::read(previous.join("yt-dlp.whl")).unwrap(),
        b"old downloader"
    );
    assert_eq!(
        fs::read(previous.join("yt-dlp-ejs.whl")).unwrap(),
        b"old extractor"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn downloads_a_verified_manifest_set_from_a_local_http_fixture() {
    let downloader = b"updated downloader";
    let extractor = b"updated extractor";
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let manifest = format!(
        r#"{{"version":"updated","downloader":{{"url":"{base_url}/yt-dlp.whl","sha256":"{}"}},"extractor":{{"url":"{base_url}/yt-dlp-ejs.whl","sha256":"{}"}}}}"#,
        sha256(downloader),
        sha256(extractor),
    );
    let server = thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 1024];
            let length = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..length]);
            let body: &[u8] = if request.starts_with("GET /manifest.json ") {
                manifest.as_bytes()
            } else if request.starts_with("GET /yt-dlp.whl ") {
                downloader
            } else {
                extractor
            };
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(body).unwrap();
        }
    });

    let root = test_directory();
    let manager = ToolManager::open(root.join("tools")).unwrap();
    manager
        .update_from_manifest_url(format!("{base_url}/manifest.json"))
        .unwrap();

    let current = manager.current().unwrap().unwrap();
    assert_eq!(current.version, "updated");
    assert_eq!(fs::read(current.downloader).unwrap(), downloader);
    assert_eq!(fs::read(current.extractor).unwrap(), extractor);
    server.join().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_an_invalid_extractor_without_replacing_the_current_set() {
    let root = test_directory();
    let manager = ToolManager::open(root.join("tools")).unwrap();

    let first = root.join("first");
    fs::create_dir_all(&first).unwrap();
    fs::write(first.join("yt-dlp.whl"), b"old downloader").unwrap();
    fs::write(first.join("yt-dlp-ejs.whl"), b"old extractor").unwrap();
    manager
        .install(ManagedToolSet::new(
            "old",
            first.join("yt-dlp.whl"),
            sha256(b"old downloader"),
            first.join("yt-dlp-ejs.whl"),
            sha256(b"old extractor"),
        ))
        .unwrap();

    let replacement = root.join("replacement");
    fs::create_dir_all(&replacement).unwrap();
    fs::write(replacement.join("yt-dlp.whl"), b"new downloader").unwrap();
    fs::write(replacement.join("yt-dlp-ejs.whl"), b"tampered extractor").unwrap();
    let error = manager
        .install(ManagedToolSet::new(
            "new",
            replacement.join("yt-dlp.whl"),
            sha256(b"new downloader"),
            replacement.join("yt-dlp-ejs.whl"),
            sha256(b"expected extractor"),
        ))
        .unwrap_err();

    assert!(error.to_string().contains("checksum"));
    let current = manager.current().unwrap().expect("old set retained");
    assert_eq!(current.version, "old");
    assert_eq!(fs::read(current.downloader).unwrap(), b"old downloader");
    assert_eq!(fs::read(current.extractor).unwrap(), b"old extractor");

    fs::remove_dir_all(root).unwrap();
}
