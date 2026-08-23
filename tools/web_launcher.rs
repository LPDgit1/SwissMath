use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::time::Duration;

const INDEX_HTML: &[u8] = include_bytes!("../dist/web/index.html");
const APP_JS: &[u8] = include_bytes!("../dist/web/app.js");
const STYLES_CSS: &[u8] = include_bytes!("../dist/web/styles.css");
const WASM_JS: &[u8] = include_bytes!("../dist/web/pkg/swissmath_web.js");
const WASM_BINARY: &[u8] = include_bytes!("../dist/web/pkg/swissmath_web_bg.wasm");

struct Options {
    port: u16,
    open_browser: bool,
}

fn options() -> Result<Options, String> {
    let mut port = 0;
    let mut open_browser = true;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-browser" => open_browser = false,
            "--port" => {
                let value = args.next().ok_or("Manca il valore dopo --port.")?;
                port = value
                    .parse::<u16>()
                    .map_err(|_| format!("Porta non valida: {value}"))?;
            }
            _ => return Err(format!("Argomento non riconosciuto: {arg}")),
        }
    }

    Ok(Options { port, open_browser })
}

fn asset(path: &str) -> Option<(&'static [u8], &'static str)> {
    match path {
        "/" | "/index.html" => Some((INDEX_HTML, "text/html; charset=utf-8")),
        "/app.js" => Some((APP_JS, "text/javascript; charset=utf-8")),
        "/styles.css" => Some((STYLES_CSS, "text/css; charset=utf-8")),
        "/pkg/swissmath_web.js" => Some((WASM_JS, "text/javascript; charset=utf-8")),
        "/pkg/swissmath_web_bg.wasm" => Some((WASM_BINARY, "application/wasm")),
        _ => None,
    }
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    if !head_only {
        stream.write_all(body)?;
    }
    stream.flush()
}

fn handle(mut stream: TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;

    let request_line = {
        let mut reader = BufReader::new(&mut stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line)? == 0 {
            return Ok(());
        }
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
                break;
            }
        }
        request_line
    };

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let requested = parts.next().unwrap_or_default();
    let head_only = method == "HEAD";

    if method != "GET" && !head_only {
        return respond(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain; charset=utf-8",
            b"Metodo non consentito.",
            false,
        );
    }

    let path = requested.split('?').next().unwrap_or_default();
    if path == "/favicon.ico" {
        return respond(
            &mut stream,
            "204 No Content",
            "image/x-icon",
            &[],
            head_only,
        );
    }

    match asset(path) {
        Some((body, content_type)) => respond(&mut stream, "200 OK", content_type, body, head_only),
        None => respond(
            &mut stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"Risorsa non trovata.",
            head_only,
        ),
    }
}

fn open_default_browser(url: &str) -> io::Result<()> {
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()?
        .wait()?;
    Ok(())
}

fn run() -> Result<(), String> {
    let options = options()?;
    let listener = TcpListener::bind(("127.0.0.1", options.port))
        .map_err(|error| format!("Impossibile avviare SwissMath: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("Impossibile determinare l'indirizzo locale: {error}"))?;
    let url = format!("http://127.0.0.1:{}/", address.port());

    println!("SwissMath Web e' pronto su {url}");
    println!("Lascia aperta questa finestra mentre usi l'applicazione.");
    println!("Chiudila per arrestare SwissMath.");

    if options.open_browser {
        open_default_browser(&url)
            .map_err(|error| format!("Impossibile aprire il browser: {error}. Apri {url}"))?;
    }

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                std::thread::spawn(|| {
                    if let Err(error) = handle(stream) {
                        eprintln!("Richiesta locale non completata: {error}");
                    }
                });
            }
            Err(error) => eprintln!("Connessione locale non accettata: {error}"),
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        eprintln!("Premi Invio per chiudere.");
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        std::process::exit(1);
    }
}
