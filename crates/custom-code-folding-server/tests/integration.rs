use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn send_message(stdin: &mut impl Write, content: &str) {
    let msg = format!("Content-Length: {}\r\n\r\n{}", content.len(), content);
    stdin.write_all(msg.as_bytes()).unwrap();
    stdin.flush().unwrap();
}

fn read_message(reader: &mut BufReader<impl Read>) -> serde_json::Value {
    // Read headers
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some(len) = line.strip_prefix("Content-Length: ") {
            content_length = len.parse().unwrap();
        }
    }

    // Read body
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[test]
fn test_server_folding_range() {
    let server_binary = env!("CARGO_BIN_EXE_custom-code-folding-server");

    let mut child = Command::new(server_binary)
        .arg("--stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");

    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());

    // Initialize
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "initializationOptions": {
                "include_defaults": true
            }
        }
    });
    send_message(&mut stdin, &init_request.to_string());
    let init_result = read_message(&mut reader);
    assert!(init_result["result"]["capabilities"]["foldingRangeProvider"]
        .as_bool()
        .unwrap());

    // Send initialized notification
    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        })
        .to_string(),
    );

    // Small delay to let server process notifications before requests
    thread::sleep(Duration::from_millis(50));

    // Open a document with fold markers
    let test_doc = "line 0\n# +++ Section A\nline 2\nline 3\n# ---\nline 5\n";
    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.py",
                    "languageId": "python",
                    "version": 1,
                    "text": test_doc
                }
            }
        })
        .to_string(),
    );

    thread::sleep(Duration::from_millis(50));

    // Request folding ranges
    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/foldingRange",
            "params": {
                "textDocument": {
                    "uri": "file:///test.py"
                }
            }
        })
        .to_string(),
    );
    let folding_result = read_message(&mut reader);

    let ranges = folding_result["result"].as_array().unwrap();
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0]["startLine"], 1);
    assert_eq!(ranges[0]["endLine"], 4);
    assert_eq!(ranges[0]["kind"], "region");
    assert_eq!(ranges[0]["collapsedText"], "Section A");

    // Close stdin to terminate the server
    drop(stdin);
    child.wait().unwrap();
}
