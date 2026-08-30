use std::{env, process, thread::sleep, time::Duration};

const PROGRESS_TEMPLATE: &str = "download:MYTORY_PROGRESS:%(progress.downloaded_bytes)s:%(progress.total_bytes)s:%(progress.total_bytes_estimate)s:%(progress.speed)s:%(progress.eta)s";

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let Some(url) = arguments.last() else {
        fail("unknown", "Missing URL argument.");
    };

    let has_expected_progress_template = arguments.windows(2).any(|arguments| {
        arguments[0] == "--progress-template" && arguments[1] == PROGRESS_TEMPLATE
    });

    if !arguments.iter().any(|argument| argument == "--newline")
        || !arguments.iter().any(|argument| argument == "-o")
        || !arguments.iter().any(|argument| argument == "--")
        || !has_expected_progress_template
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
        "simulator://estimated-progress" => {
            println!("MYTORY_PROGRESS:524288:NA:1048576:1048576:12");
        }
        "simulator://slow-success" => {
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
            sleep(Duration::from_secs(2));
        }
        "simulator://transient-network-failure" => {
            println!("MYTORY_PROGRESS:524288:1048576:NA:1048576:12");
            fail("transient_network", "Temporary network interruption.");
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

fn fail(kind: &str, message: &str) -> ! {
    eprintln!("MYTORY_FAILURE:{kind}:{message}");
    process::exit(1);
}
