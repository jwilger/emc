#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::{Error as IoError, ErrorKind, Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::process::{Command as ProcessCommand, Stdio};
    use std::sync::{Mutex, MutexGuard};
    use std::thread;
    use std::time::Duration;

    use assert_cmd::cargo::cargo_bin;
    use predicates::Predicate;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    static MCP_HTTP_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn mcp_http_serves_initialize_on_localhost_with_origin_check() -> Result<(), Box<dyn Error>> {
        let _guard = mcp_http_test_lock()?;
        let temp_dir = TempDir::new()?;
        let port = available_loopback_port()?;
        let server = ProcessCommand::new(cargo_bin("emc"))
            .args([
                "mcp",
                "http",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--once",
            ])
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let response = send_initialize_request(port)?;
        let output = server.wait_with_output()?;

        assert!(output.status.success());
        assert!(String::from_utf8(output.stderr)?.is_empty());
        assert!(
            String::from_utf8(output.stdout)?
                .contains(&format!("MCP HTTP listening on 127.0.0.1:{port}"))
        );
        assert!(predicate::str::contains("HTTP/1.1 200 OK").eval(&response));
        assert!(predicate::str::contains("\"serverInfo\"").eval(&response));
        assert!(predicate::str::contains("\"name\":\"emc\"").eval(&response));

        Ok(())
    }

    #[test]
    fn mcp_http_accepts_explicit_local_bind_for_server_mode() -> Result<(), Box<dyn Error>> {
        let _guard = mcp_http_test_lock()?;
        let temp_dir = TempDir::new()?;
        let port = available_loopback_port()?;
        let mut server = ProcessCommand::new(cargo_bin("emc"))
            .args([
                "mcp",
                "http",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
            ])
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let response = send_initialize_request(port)?;
        server.kill()?;
        let output = server.wait_with_output()?;

        assert!(output.status.success() || output.status.code().is_none());
        assert!(String::from_utf8(output.stderr)?.is_empty());
        assert!(predicate::str::contains("HTTP/1.1 200 OK").eval(&response));
        assert!(predicate::str::contains("\"serverInfo\"").eval(&response));

        Ok(())
    }

    #[test]
    fn mcp_http_rejects_non_local_bind_without_auth_token() -> Result<(), Box<dyn Error>> {
        let _guard = mcp_http_test_lock()?;
        let temp_dir = TempDir::new()?;
        let port = available_loopback_port()?;

        let output = ProcessCommand::new(cargo_bin("emc"))
            .args([
                "mcp",
                "http",
                "--host",
                "0.0.0.0",
                "--port",
                &port.to_string(),
                "--once",
            ])
            .current_dir(temp_dir.path())
            .output()?;

        assert!(!output.status.success());
        assert!(
            String::from_utf8(output.stderr)?
                .contains("MCP HTTP non-local bind requires --auth-token")
        );

        Ok(())
    }

    #[test]
    fn mcp_http_requires_bearer_token_for_non_local_bind() -> Result<(), Box<dyn Error>> {
        let _guard = mcp_http_test_lock()?;
        let temp_dir = TempDir::new()?;
        let port = available_loopback_port()?;
        let server = ProcessCommand::new(cargo_bin("emc"))
            .args([
                "mcp",
                "http",
                "--host",
                "0.0.0.0",
                "--port",
                &port.to_string(),
                "--auth-token",
                "package-secret",
                "--once",
            ])
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let response = send_initialize_request_with_origin(port, "0.0.0.0")?;
        let output = server.wait_with_output()?;

        assert!(output.status.success());
        assert!(String::from_utf8(output.stderr)?.is_empty());
        assert!(
            String::from_utf8(output.stdout)?
                .contains(&format!("MCP HTTP listening on 0.0.0.0:{port}"))
        );
        assert!(predicate::str::contains("HTTP/1.1 401 Unauthorized").eval(&response));

        Ok(())
    }

    fn mcp_http_test_lock() -> Result<MutexGuard<'static, ()>, Box<dyn Error>> {
        MCP_HTTP_TEST_LOCK
            .lock()
            .map_err(|error| IoError::other(error.to_string()).into())
    }

    fn available_loopback_port() -> Result<u16, Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        Ok(listener.local_addr()?.port())
    }

    fn send_initialize_request(port: u16) -> Result<String, Box<dyn Error>> {
        send_initialize_request_with_origin(port, "127.0.0.1")
    }

    fn send_initialize_request_with_origin(
        port: u16,
        origin_host: &str,
    ) -> Result<String, Box<dyn Error>> {
        let body = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"emc-http-test\",\"version\":\"0.0.0\"}}}";
        let request = format!(
            "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://{origin_host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let mut stream = connect_with_retry(port)?;
        stream.write_all(request.as_bytes())?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        Ok(response)
    }

    fn connect_with_retry(port: u16) -> Result<TcpStream, Box<dyn Error>> {
        let mut last_error = None;
        for _attempt in 0..50 {
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(stream) => return Ok(stream),
                Err(error) => {
                    last_error = Some(error);
                    thread::sleep(Duration::from_millis(20));
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| IoError::new(ErrorKind::ConnectionRefused, "connection refused"))
            .into())
    }
}
