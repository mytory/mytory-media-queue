use std::{env, io::Write, process, thread::sleep, time::Duration};

const PROGRESS_TEMPLATE: &str = "download:MYTORY_PROGRESS:%(progress.downloaded_bytes)s:%(progress.total_bytes)s:%(progress.total_bytes_estimate)s:%(progress.speed)s:%(progress.eta)s";
const OUTPUT_TEMPLATE: &str = "%(title)s [%(id)s].%(ext)s";

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let Some(url) = arguments.last() else {
        fail("unknown", "Missing URL argument.");
    };

    let has_expected_progress_template = arguments.windows(2).any(|arguments| {
        arguments[0] == "--progress-template" && arguments[1] == PROGRESS_TEMPLATE
    });
    let has_expected_output_template = arguments
        .windows(2)
        .any(|arguments| arguments[0] == "-o" && arguments[1].ends_with(OUTPUT_TEMPLATE));

    if !arguments.iter().any(|argument| argument == "--newline")
        || !arguments.iter().any(|argument| argument == "-o")
        || !arguments.iter().any(|argument| argument == "--")
        || !arguments
            .iter()
            .any(|argument| argument == "--write-thumbnail")
        || !has_expected_progress_template
        || !has_expected_output_template
    {
        fail(
            "unknown",
            "Simulator did not receive the downloader argument contract.",
        );
    }

    match url.as_str() {
        "simulator://success" => {
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
        }
        "simulator://bundled-python" => {
            require_args(
                &arguments,
                &["-m", "yt_dlp", "--js-runtimes", "deno:/resources/deno"],
            );
            let python_path = env::var("PYTHONPATH").unwrap_or_default();
            if !python_path.contains("/tools/current/yt-dlp.whl")
                || !python_path.contains("/tools/current/yt-dlp-ejs.whl")
            {
                fail(
                    "unknown",
                    "Simulator did not receive the bundled Python package paths.",
                );
            }
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
        }
        "simulator://estimated-progress" => {
            println!("MYTORY_PROGRESS:524288:NA:1048576:1048576:12");
        }
        "simulator://non-utf8-output" => {
            std::io::stdout()
                .write_all(b"[download] \xFF\n")
                .expect("simulator stdout must be writable");
            std::io::stderr()
                .write_all(b"WARNING: \xFF\n")
                .expect("simulator stderr must be writable");
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
        }
        "simulator://decimal-progress" => {
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576.5:12.9");
        }
        "simulator://mp4-compatible" => {
            require_args(
                &arguments,
                &[
                    "-f",
                    "bv*[vcodec^=avc1]+ba[acodec^=mp4a]/bv*[vcodec^=avc1]+ba/bv*+ba[acodec^=mp4a]/bv*+ba/b",
                    "--merge-output-format",
                ],
            );
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
        }
        "simulator://subs" => {
            require_args(
                &arguments,
                &[
                    "--write-subs",
                    "--sub-langs",
                    "ko,en",
                    "--convert-subs",
                    "vtt",
                ],
            );
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
        }
        "simulator://cookies" => {
            if !arguments
                .windows(2)
                .any(|arguments| arguments[0] == "--cookies" && !arguments[1].is_empty())
            {
                fail(
                    "unknown",
                    "Simulator did not receive the cookie source argument.",
                );
            }
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
        }
        "simulator://ffmpeg-location" => {
            if !arguments
                .windows(2)
                .any(|arguments| arguments[0] == "--ffmpeg-location" && !arguments[1].is_empty())
            {
                fail(
                    "unknown",
                    "Simulator did not receive the ffmpeg location argument.",
                );
            }
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
        }
        "simulator://slow-success" => {
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
            sleep(Duration::from_secs(2));
        }
        "simulator://transient-network-failure" => {
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
            fail("transient_network", "Temporary network interruption.");
        }
        "simulator://raw-network-error" => {
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
            eprintln!("ERROR: [generic] sample: Unable to download webpage: <urlopen error [Errno -2] Name or service not known>");
            process::exit(1);
        }
        "simulator://raw-permission-error" => {
            eprintln!("ERROR: [download] Destination: Permission denied: '/downloads/out.mp4'");
            process::exit(1);
        }
        "simulator://raw-unknown-error" => {
            eprintln!("ERROR: [youtube] abc123: Video unavailable");
            process::exit(1);
        }
        "simulator://interrupted" => {
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
            fail("interrupted", "Download interrupted.");
        }
        "simulator://permission-failure" => {
            eprintln!("MYTORY_TEST_COOKIE=not-a-real-cookie");
            fail("permission", "Destination is not writable.");
        }
        _ => fail("unknown", "Unknown simulator scenario."),
    }
}

fn require_args(arguments: &[String], expected: &[&str]) {
    let present = arguments
        .windows(expected.len())
        .any(|window| window == expected);
    if !present {
        fail(
            "unknown",
            "Simulator did not receive the downloader argument contract.",
        );
    }
}

fn fail(kind: &str, message: &str) -> ! {
    eprintln!("MYTORY_FAILURE:{kind}:{message}");
    process::exit(1);
}
