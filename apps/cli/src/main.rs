use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde::Serialize;
use serde_json::{Value, json};
use swissmath_core::{
    DecimalIntegerAnalysis, FpLinearSystemSolution, FpMatrix, FpPolynomial, LinearCongruence,
    LinearSolution, Modulus, PrimalityAssessment, PrimeField, Valuation, analyze_integer_decimal,
    assess_primality_decimal, extended_gcd, factor, inv_mod, modular_square_roots, next_prime,
    previous_prime, rational_reconstruct, solve_linear_congruence, valuation,
};

const CORE_VERSION: &str = "0.7";

#[derive(Debug)]
struct Cli {
    command: String,
    values: Vec<String>,
    json: bool,
    jsonl: bool,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    column: Option<String>,
}

#[derive(Debug)]
struct OperationResult {
    human: String,
    result: Value,
    exactness: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct OutputRecord {
    operation: String,
    input: Value,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exactness: Option<&'static str>,
    core_version: &'static str,
    elapsed_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorBody>,
    #[serde(skip)]
    human: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("swissmath: {message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "-h" | "--help"))
    {
        println!("{}", usage());
        return Ok(());
    }
    let cli = parse_cli(arguments.into_iter())?;
    if let Some(path) = cli.input.as_deref() {
        if is_field_family(&cli.command) {
            let mut values = cli.values.clone();
            values.push(
                std::fs::read_to_string(path)
                    .map_err(|error| format!("could not read {}: {error}", path.display()))?,
            );
            return emit_record(&cli, timed_execute(&cli.command, &values), false);
        }
        return run_csv(&cli, path);
    }
    if cli.column.is_some() || cli.output.is_some() {
        return Err("--column e --output richiedono --input <file.csv>".to_owned());
    }
    let mut values = cli.values.clone();
    if is_field_family(&cli.command) && !io::stdin().is_terminal() {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| format!("could not read stdin: {error}"))?;
        if !input.trim().is_empty() {
            values.push(input);
        }
    }
    if !values.is_empty() {
        let record = timed_execute(&cli.command, &values);
        return emit_record(&cli, record, false);
    }
    if io::stdin().is_terminal() {
        return Err(format!("input mancante\n\n{}", usage()));
    }
    run_stream(&cli)
}

fn parse_cli(arguments: impl Iterator<Item = String>) -> Result<Cli, String> {
    let mut command = None;
    let mut values = Vec::new();
    let mut json = false;
    let mut jsonl = false;
    let mut input = None;
    let mut output = None;
    let mut column = None;
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--json" => json = true,
            "--jsonl" => jsonl = true,
            "--input" => input = Some(PathBuf::from(next_option(&mut arguments, "--input")?)),
            "--output" => output = Some(PathBuf::from(next_option(&mut arguments, "--output")?)),
            "--column" => column = Some(next_option(&mut arguments, "--column")?),
            value if value.starts_with('-') && value.parse::<i128>().is_err() => {
                return Err(format!("opzione sconosciuta: {value}"));
            }
            value if command.is_none() => command = Some(value.to_owned()),
            value => values.push(value.to_owned()),
        }
    }
    if json && jsonl {
        return Err("--json e --jsonl sono mutuamente esclusivi".to_owned());
    }
    Ok(Cli {
        command: command.ok_or_else(|| usage().to_owned())?,
        values,
        json,
        jsonl,
        input,
        output,
        column,
    })
}

fn next_option(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("valore mancante per {option}"))
}

fn usage() -> &'static str {
    "Uso: swissmath <comando> [argomenti] [--json|--jsonl]\n\
Comandi: prime, factor, analyze, gcd, xgcd, inverse, congruence,\n\
         next-prime, prev-prime, reconstruct, sqrtmod, valuation,\n\
         mobius, radical, squarefree, divisor-count, divisor-sum, divisors\n\
         matrix <add|sub|mul|matvec|det|rank|rref|solve|inverse|kernel> <p> ...\n\
         polynomial <add|sub|mul|divrem|gcd|xgcd|derivative|evaluate|powmod> <p> ...\n\
CSV: swissmath <comando-scalare> --input file.csv --column n [--output out.csv]"
}

fn is_field_family(command: &str) -> bool {
    matches!(command, "matrix" | "polynomial")
}

fn run_stream(cli: &Cli) -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.map_err(|error| format!("lettura stdin: {error}"))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record = timed_execute(&cli.command, &[trimmed.to_owned()]);
        if cli.jsonl || cli.json {
            serde_json::to_writer(&mut stdout, &record).map_err(|error| error.to_string())?;
            writeln!(stdout).map_err(|error| error.to_string())?;
        } else if let Some(error) = record.error {
            writeln!(
                stdout,
                "{trimmed}: errore [{}] {}",
                error.code, error.message
            )
            .map_err(|error| error.to_string())?;
        } else if let Some(human) = record.human {
            writeln!(stdout, "{human}").map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn run_csv(cli: &Cli, path: &Path) -> Result<(), String> {
    if !cli.values.is_empty() {
        return Err("CSV e argomenti posizionali non possono essere combinati".to_owned());
    }
    if !is_scalar_command(&cli.command) {
        return Err("il workflow CSV accetta soltanto comandi a input singolo".to_owned());
    }
    let column = cli
        .column
        .as_deref()
        .ok_or_else(|| "--column è richiesto con --input".to_owned())?;
    let reader = CsvReader::new(BufReader::new(
        File::open(path).map_err(|error| format!("{}: {error}", path.display()))?,
    ));
    if let Some(output) = cli.output.as_deref() {
        let file =
            File::create(output).map_err(|error| format!("{}: {error}", output.display()))?;
        process_csv(cli, column, reader, BufWriter::new(file))
    } else {
        process_csv(cli, column, reader, io::stdout().lock())
    }
}

fn process_csv<R: Read, W: Write>(
    cli: &Cli,
    column: &str,
    mut reader: CsvReader<R>,
    mut writer: W,
) -> Result<(), String> {
    let original_headers = reader
        .next_record()?
        .ok_or_else(|| "CSV vuoto".to_owned())?;
    let column_index = original_headers
        .iter()
        .position(|header| header == column)
        .ok_or_else(|| format!("colonna CSV non trovata: {column}"))?;
    let mut headers = original_headers;
    headers.extend(
        [
            "swissmath_status",
            "swissmath_result",
            "swissmath_exactness",
            "swissmath_error",
        ]
        .map(str::to_owned),
    );
    write_csv_record(&mut writer, &headers)?;
    while let Some(mut row) = reader.next_record()? {
        let input = row
            .get(column_index)
            .ok_or_else(|| "riga CSV più corta dell'intestazione".to_owned())?
            .to_owned();
        let record = timed_execute(&cli.command, std::slice::from_ref(&input));
        let result = record
            .result
            .as_ref()
            .map(Value::to_string)
            .unwrap_or_default();
        let error = record
            .error
            .as_ref()
            .map(|error| format!("{}: {}", error.code, error.message))
            .unwrap_or_default();
        row.extend([
            record.status.to_owned(),
            result,
            record.exactness.unwrap_or_default().to_owned(),
            error,
        ]);
        write_csv_record(&mut writer, &row)?;
    }
    writer.flush().map_err(|error| error.to_string())
}

struct CsvReader<R> {
    reader: R,
    finished: bool,
}

impl<R: Read> CsvReader<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            finished: false,
        }
    }

    fn next_record(&mut self) -> Result<Option<Vec<String>>, String> {
        if self.finished {
            return Ok(None);
        }
        let mut fields = Vec::new();
        let mut field = Vec::new();
        let mut in_quotes = false;
        let mut after_quote = false;
        let mut saw_byte = false;
        loop {
            let mut byte = [0_u8; 1];
            let count = self
                .reader
                .read(&mut byte)
                .map_err(|error| error.to_string())?;
            if count == 0 {
                self.finished = true;
                if in_quotes {
                    return Err("campo CSV tra virgolette non terminato".to_owned());
                }
                if !saw_byte && fields.is_empty() && field.is_empty() {
                    return Ok(None);
                }
                fields.push(csv_field(field)?);
                return Ok(Some(fields));
            }
            saw_byte = true;
            match (in_quotes, after_quote, byte[0]) {
                (true, _, b'"') => {
                    in_quotes = false;
                    after_quote = true;
                }
                (true, _, value) => field.push(value),
                (false, true, b'"') => {
                    field.push(b'"');
                    in_quotes = true;
                    after_quote = false;
                }
                (false, true, b',') => {
                    fields.push(csv_field(std::mem::take(&mut field))?);
                    after_quote = false;
                }
                (false, true, b'\n') => {
                    fields.push(csv_field(field)?);
                    return Ok(Some(fields));
                }
                (false, true, b'\r') => {}
                (false, true, _) => {
                    return Err("carattere non valido dopo un campo CSV quotato".to_owned());
                }
                (false, false, b'"') if field.is_empty() => in_quotes = true,
                (false, false, b'"') => return Err("virgolette CSV non valide".to_owned()),
                (false, false, b',') => {
                    fields.push(csv_field(std::mem::take(&mut field))?);
                }
                (false, false, b'\n') => {
                    fields.push(csv_field(field)?);
                    return Ok(Some(fields));
                }
                (false, false, b'\r') => {}
                (false, false, value) => field.push(value),
            }
        }
    }
}

fn csv_field(bytes: Vec<u8>) -> Result<String, String> {
    String::from_utf8(bytes).map_err(|_| "il CSV deve essere UTF-8".to_owned())
}

fn write_csv_record(writer: &mut impl Write, fields: &[String]) -> Result<(), String> {
    for (index, field) in fields.iter().enumerate() {
        if index != 0 {
            writer.write_all(b",").map_err(|error| error.to_string())?;
        }
        if field
            .bytes()
            .any(|byte| matches!(byte, b',' | b'"' | b'\r' | b'\n'))
        {
            writer.write_all(b"\"").map_err(|error| error.to_string())?;
            writer
                .write_all(field.replace('"', "\"\"").as_bytes())
                .map_err(|error| error.to_string())?;
            writer.write_all(b"\"").map_err(|error| error.to_string())?;
        } else {
            writer
                .write_all(field.as_bytes())
                .map_err(|error| error.to_string())?;
        }
    }
    writer.write_all(b"\n").map_err(|error| error.to_string())
}

fn is_scalar_command(command: &str) -> bool {
    matches!(
        command,
        "prime"
            | "factor"
            | "analyze"
            | "next-prime"
            | "prev-prime"
            | "mobius"
            | "radical"
            | "squarefree"
            | "divisor-count"
            | "divisor-sum"
            | "divisors"
    )
}

fn timed_execute(command: &str, values: &[String]) -> OutputRecord {
    let started = Instant::now();
    let input = if values.len() == 1 {
        Value::String(values[0].clone())
    } else {
        json!(values)
    };
    match execute(command, values) {
        Ok(output) => OutputRecord {
            operation: command.to_owned(),
            input,
            status: "ok",
            result: Some(output.result),
            exactness: Some(output.exactness),
            core_version: CORE_VERSION,
            elapsed_ms: started.elapsed().as_secs_f64() * 1_000.0,
            error: None,
            human: Some(output.human),
        },
        Err(message) => OutputRecord {
            operation: command.to_owned(),
            input,
            status: "error",
            result: None,
            exactness: None,
            core_version: CORE_VERSION,
            elapsed_ms: started.elapsed().as_secs_f64() * 1_000.0,
            error: Some(ErrorBody {
                code: classify_error(&message),
                message,
            }),
            human: None,
        },
    }
}

fn emit_record(cli: &Cli, record: OutputRecord, streaming: bool) -> Result<(), String> {
    if cli.json || cli.jsonl || streaming {
        let text = serde_json::to_string(&record).map_err(|error| error.to_string())?;
        println!("{text}");
        return Ok(());
    }
    if let Some(error) = record.error {
        return Err(format!("[{}] {}", error.code, error.message));
    }
    println!("{}", record.human.unwrap_or_default());
    Ok(())
}

fn classify_error(message: &str) -> &'static str {
    if message.contains("nessun") || message.contains("non esiste") {
        "no_solution"
    } else if message.contains("ricostruzione") {
        "no_reconstruction"
    } else if message.contains("support") || message.contains("dominio") {
        "out_of_domain"
    } else {
        "invalid_input"
    }
}

fn execute(command: &str, values: &[String]) -> Result<OperationResult, String> {
    match command {
        "prime" => operation_prime(one(values)?),
        "factor" => operation_factor(parse_u64(one(values)?)?),
        "analyze" => operation_analyze(one(values)?),
        "gcd" => {
            let (a, b) = two_u64(values)?;
            let value = swissmath_core::gcd(a, b);
            exact(value.to_string(), json!({ "gcd": value.to_string() }))
        }
        "xgcd" => {
            let (a, b) = two_u64(values)?;
            let value = extended_gcd(a, b);
            exact(
                format!("gcd={}; x={}; y={}", value.gcd, value.x, value.y),
                json!({ "gcd": value.gcd.to_string(), "x": value.x.to_string(), "y": value.y.to_string() }),
            )
        }
        "inverse" => {
            let (a, modulus) = two_u64(values)?;
            let modulus = modulus_value(modulus)?;
            let inverse =
                inv_mod(a, modulus).ok_or_else(|| "nessun inverso modulare".to_owned())?;
            exact(
                inverse.to_string(),
                json!({ "inverse": inverse.to_string(), "modulus": modulus.get().to_string() }),
            )
        }
        "congruence" => operation_congruence(values),
        "next-prime" => {
            let n = parse_u64(one(values)?)?;
            let prime = next_prime(n).map_err(|error| error.to_string())?;
            exact(prime.to_string(), json!({ "prime": prime.to_string() }))
        }
        "prev-prime" => {
            let n = parse_u64(one(values)?)?;
            let prime = previous_prime(n).ok_or_else(|| "nessun primo precedente".to_owned())?;
            exact(prime.to_string(), json!({ "prime": prime.to_string() }))
        }
        "reconstruct" => {
            let (residue, modulus) = two_u64(values)?;
            let result = rational_reconstruct(residue, modulus)
                .map_err(|error| format!("parametri di ricostruzione non validi: {error:?}"))?
                .ok_or_else(|| "nessuna ricostruzione razionale".to_owned())?;
            exact(
                result.to_string(),
                json!({ "numerator": result.numerator.to_string(), "denominator": result.denominator.to_string() }),
            )
        }
        "sqrtmod" => {
            require_arity(values, 2)?;
            let a = parse_i128(&values[0])?;
            let modulus = parse_u64(&values[1])?;
            let roots = modular_square_roots(a, modulus).map_err(|error| error.to_string())?;
            exact(
                roots
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join(" "),
                json!({ "roots": roots.iter().map(u64::to_string).collect::<Vec<_>>(), "modulus": modulus.to_string() }),
            )
        }
        "valuation" => {
            let (n, p) = two_u64(values)?;
            let value = valuation(n, p).map_err(|error| error.to_string())?;
            match value {
                Valuation::Finite(exponent) => exact(
                    exponent.to_string(),
                    json!({ "valuation": exponent, "infinite": false }),
                ),
                Valuation::Infinite => exact("infinite".to_owned(), json!({ "infinite": true })),
            }
        }
        "mobius" | "radical" | "squarefree" | "divisor-count" | "divisor-sum" | "divisors" => {
            operation_factor_derived(command, parse_u64(one(values)?)?)
        }
        "matrix" => operation_fp_matrix(values),
        "polynomial" => operation_fp_polynomial(values),
        _ => Err(format!("comando sconosciuto: {command}")),
    }
}

fn operation_fp_matrix(values: &[String]) -> Result<OperationResult, String> {
    if values.len() < 3 {
        return Err("usage: swissmath matrix <operation> <prime> <matrix> [operand]".to_owned());
    }
    let operation = values[0].as_str();
    let field = prime_field(&values[1])?;
    let left = parse_fp_matrix(field, &values[2])?;
    let matrix_json = |matrix: &FpMatrix| json!(matrix.to_rows());
    match operation {
        "add" | "sub" | "mul" => {
            require_arity(values, 4)?;
            let right = parse_fp_matrix(field, &values[3])?;
            let result = match operation {
                "add" => left.add(field, &right),
                "sub" => left.sub(field, &right),
                _ => left.mul(field, &right),
            }
            .map_err(field_error)?;
            exact(
                format_matrix(&result),
                json!({ "matrix": matrix_json(&result), "modulus": field.modulus().to_string() }),
            )
        }
        "matvec" => {
            require_arity(values, 4)?;
            let vector = parse_i128_list(&values[3])?;
            let result = left.mul_vector(field, &vector).map_err(field_error)?;
            exact(
                format_vector(&result),
                json!({ "vector": result, "modulus": field.modulus().to_string() }),
            )
        }
        "det" => {
            require_arity(values, 3)?;
            let result = left.determinant(field).map_err(field_error)?;
            exact(
                result.to_string(),
                json!({ "determinant": result.to_string(), "modulus": field.modulus().to_string() }),
            )
        }
        "rank" => {
            require_arity(values, 3)?;
            let result = left.rank(field);
            exact(
                result.to_string(),
                json!({ "rank": result, "modulus": field.modulus().to_string() }),
            )
        }
        "rref" => {
            require_arity(values, 3)?;
            let result = left.rref(field);
            exact(
                format_matrix(&result.matrix),
                json!({ "matrix": matrix_json(&result.matrix), "pivots": result.pivot_columns, "modulus": field.modulus().to_string() }),
            )
        }
        "inverse" => {
            require_arity(values, 3)?;
            let result = left.inverse(field).map_err(field_error)?;
            exact(
                format_matrix(&result),
                json!({ "matrix": matrix_json(&result), "modulus": field.modulus().to_string() }),
            )
        }
        "kernel" => {
            require_arity(values, 3)?;
            let result = left.kernel(field);
            exact(
                result
                    .iter()
                    .map(|row| format_vector(row))
                    .collect::<Vec<_>>()
                    .join("\n"),
                json!({ "basis": result, "modulus": field.modulus().to_string() }),
            )
        }
        "solve" => {
            require_arity(values, 4)?;
            let rhs = parse_i128_list(&values[3])?;
            match left.solve(field, &rhs).map_err(field_error)? {
                FpLinearSystemSolution::None => exact(
                    "no solution".to_owned(),
                    json!({ "kind": "none", "modulus": field.modulus().to_string() }),
                ),
                FpLinearSystemSolution::Unique(solution) => exact(
                    format_vector(&solution),
                    json!({ "kind": "unique", "solution": solution, "modulus": field.modulus().to_string() }),
                ),
                FpLinearSystemSolution::Infinite {
                    particular,
                    kernel_basis,
                } => exact(
                    format!(
                        "particular {}\nkernel {}",
                        format_vector(&particular),
                        kernel_basis
                            .iter()
                            .map(|row| format_vector(row))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ),
                    json!({ "kind": "infinite", "particular": particular, "kernel_basis": kernel_basis, "modulus": field.modulus().to_string() }),
                ),
            }
        }
        _ => Err(format!("unknown matrix operation: {operation}")),
    }
}

fn operation_fp_polynomial(values: &[String]) -> Result<OperationResult, String> {
    if values.len() < 3 {
        return Err(
            "usage: swissmath polynomial <operation> <prime> <coefficients> [operand]".to_owned(),
        );
    }
    let operation = values[0].as_str();
    let field = prime_field(&values[1])?;
    let left = parse_fp_polynomial(field, &values[2])?;
    let result_json = |polynomial: &FpPolynomial| json!(polynomial.coefficients());
    match operation {
        "add" | "sub" | "mul" | "gcd" => {
            require_arity(values, 4)?;
            let right = parse_fp_polynomial(field, &values[3])?;
            let result = match operation {
                "add" => left.add(field, &right),
                "sub" => left.sub(field, &right),
                "mul" => left.mul(field, &right),
                _ => left.gcd(field, &right).map_err(field_error)?,
            };
            exact(
                format_polynomial(&result),
                json!({ "coefficients": result_json(&result), "modulus": field.modulus().to_string() }),
            )
        }
        "divrem" => {
            require_arity(values, 4)?;
            let right = parse_fp_polynomial(field, &values[3])?;
            let (quotient, remainder) = left.div_rem(field, &right).map_err(field_error)?;
            exact(
                format!(
                    "quotient: {}; remainder: {}",
                    format_polynomial(&quotient),
                    format_polynomial(&remainder)
                ),
                json!({ "quotient": result_json(&quotient), "remainder": result_json(&remainder), "modulus": field.modulus().to_string() }),
            )
        }
        "xgcd" => {
            require_arity(values, 4)?;
            let right = parse_fp_polynomial(field, &values[3])?;
            let result = left.extended_gcd(field, &right).map_err(field_error)?;
            exact(
                format_polynomial(&result.gcd),
                json!({ "gcd": result_json(&result.gcd), "left_coefficient": result_json(&result.left_coefficient), "right_coefficient": result_json(&result.right_coefficient), "modulus": field.modulus().to_string() }),
            )
        }
        "derivative" => {
            require_arity(values, 3)?;
            let result = left.derivative(field);
            exact(
                format_polynomial(&result),
                json!({ "coefficients": result_json(&result), "modulus": field.modulus().to_string() }),
            )
        }
        "evaluate" => {
            require_arity(values, 4)?;
            let result = left.evaluate(field, parse_i128(&values[3])?);
            exact(
                result.to_string(),
                json!({ "value": result.to_string(), "modulus": field.modulus().to_string() }),
            )
        }
        "powmod" => {
            require_arity(values, 5)?;
            let exponent = parse_u64(&values[3])?;
            let modulus = parse_fp_polynomial(field, &values[4])?;
            let result = left
                .pow_mod(field, exponent, &modulus)
                .map_err(field_error)?;
            exact(
                format_polynomial(&result),
                json!({ "coefficients": result_json(&result), "modulus": field.modulus().to_string() }),
            )
        }
        _ => Err(format!("unknown polynomial operation: {operation}")),
    }
}

fn prime_field(value: &str) -> Result<PrimeField, String> {
    PrimeField::new(parse_u64(value)?).map_err(|_| "the modulus must be a u64 prime".to_owned())
}

fn parse_i128_list(value: &str) -> Result<Vec<i128>, String> {
    value
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(parse_i128)
        .collect()
}

fn parse_fp_matrix(field: PrimeField, value: &str) -> Result<FpMatrix, String> {
    let rows = value
        .trim()
        .trim_matches(|character| matches!(character, '[' | ']'))
        .split([';', '\n'])
        .filter(|row| !row.trim().is_empty())
        .map(|row| parse_i128_list(row.trim_matches(|character| matches!(character, '[' | ']'))))
        .collect::<Result<Vec<_>, _>>()?;
    FpMatrix::new(field, &rows).map_err(field_error)
}

fn parse_fp_polynomial(field: PrimeField, value: &str) -> Result<FpPolynomial, String> {
    Ok(FpPolynomial::new(field, &parse_i128_list(value)?))
}

fn format_vector(values: &[u64]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn format_matrix(matrix: &FpMatrix) -> String {
    matrix
        .to_rows()
        .iter()
        .map(|row| format_vector(row))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_polynomial(polynomial: &FpPolynomial) -> String {
    if polynomial.is_zero() {
        return "0".to_owned();
    }
    polynomial
        .coefficients()
        .iter()
        .enumerate()
        .filter(|(_, coefficient)| **coefficient != 0)
        .map(|(degree, coefficient)| match degree {
            0 => coefficient.to_string(),
            1 => format!("{coefficient}x"),
            _ => format!("{coefficient}x^{degree}"),
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn field_error(error: swissmath_core::FiniteFieldError) -> String {
    format!("finite-field operation failed: {error:?}")
}

fn operation_prime(input: &str) -> Result<OperationResult, String> {
    let assessment = assess_primality_decimal(input).map_err(|error| error.to_string())?;
    let (classification, label, exactness) = match assessment {
        PrimalityAssessment::Neither => ("neither", "Neither", "neither"),
        PrimalityAssessment::Composite => ("composite", "Composite — exact", "exact"),
        PrimalityAssessment::PrimeExact => ("prime", "Prime — exact", "exact"),
        PrimalityAssessment::ExactProofIncomplete => (
            "proof_incomplete",
            "Prime proof incomplete",
            "proof_incomplete",
        ),
        PrimalityAssessment::ProbablePrime => {
            ("probable_prime", "Probable prime — BPSW", "probable")
        }
    };
    Ok(OperationResult {
        human: label.to_owned(),
        result: json!({ "classification": classification }),
        exactness,
    })
}

fn operation_factor(n: u64) -> Result<OperationResult, String> {
    let factorization = factor(n).map_err(|error| error.to_string())?;
    let parts = factorization
        .factors()
        .iter()
        .map(|factor| {
            if factor.exponent == 1 {
                factor.prime.to_string()
            } else {
                format!("{}^{}", factor.prime, factor.exponent)
            }
        })
        .collect::<Vec<_>>();
    let expression = if n == 1 {
        "1".to_owned()
    } else {
        parts.join(" * ")
    };
    exact(
        format!("{n} = {expression}"),
        json!({
            "n": n.to_string(),
            "factors": factorization.factors().iter().map(|factor| json!({
                "prime": factor.prime.to_string(), "exponent": factor.exponent
            })).collect::<Vec<_>>()
        }),
    )
}

fn operation_analyze(input: &str) -> Result<OperationResult, String> {
    match analyze_integer_decimal(input).map_err(|error| error.to_string())? {
        DecimalIntegerAnalysis::Neither { n } => Ok(OperationResult {
            human: format!("{n}: Neither"),
            result: json!({ "n": n, "classification": "neither" }),
            exactness: "neither",
        }),
        DecimalIntegerAnalysis::Exact(analysis) => {
            let factors = analysis
                .factorization
                .factors()
                .iter()
                .map(|factor| json!({ "prime": factor.prime.to_string(), "exponent": factor.exponent }))
                .collect::<Vec<_>>();
            exact(
                format!(
                    "{}: {:?}; phi={}; lambda={}; mu={}; rad={}; tau={}; sigma={}",
                    analysis.n,
                    analysis.classification,
                    analysis.phi,
                    analysis.lambda,
                    analysis.mobius,
                    analysis.radical,
                    analysis.divisor_count,
                    analysis.divisor_sum
                ),
                json!({
                    "n": analysis.n.to_string(),
                    "classification": format!("{:?}", analysis.classification).to_lowercase(),
                    "factors": factors,
                    "phi": analysis.phi.to_string(), "lambda": analysis.lambda.to_string(),
                    "mobius": analysis.mobius, "radical": analysis.radical.to_string(),
                    "squarefree": analysis.squarefree,
                    "divisor_count": analysis.divisor_count.to_string(),
                    "divisor_sum": analysis.divisor_sum.to_string()
                }),
            )
        }
        DecimalIntegerAnalysis::U128 { n, primality }
        | DecimalIntegerAnalysis::Large { n, primality } => {
            let (classification, exactness) = assessment_words(primality);
            Ok(OperationResult {
                human: format!("{n}: {classification}"),
                result: json!({ "n": n, "classification": classification }),
                exactness,
            })
        }
    }
}

fn operation_congruence(values: &[String]) -> Result<OperationResult, String> {
    require_arity(values, 3)?;
    let a = parse_u64(&values[0])?;
    let b = parse_u64(&values[1])?;
    let modulus = modulus_value(parse_u64(&values[2])?)?;
    let result = solve_linear_congruence(LinearCongruence::new(a, b, modulus));
    match result.solution {
        LinearSolution::None => Err("nessuna soluzione".to_owned()),
        LinearSolution::All => exact(
            "tutti i residui".to_owned(),
            json!({ "kind": "all", "modulus": modulus.get().to_string() }),
        ),
        LinearSolution::Class(class) => exact(
            format!("x ≡ {} (mod {})", class.residue(), class.modulus().get()),
            json!({ "kind": "class", "residue": class.residue().to_string(), "modulus": class.modulus().get().to_string(), "solution_count": result.solution_count(modulus).to_string() }),
        ),
    }
}

fn operation_factor_derived(command: &str, n: u64) -> Result<OperationResult, String> {
    let factorization = factor(n).map_err(|error| error.to_string())?;
    match command {
        "mobius" => exact(
            factorization.mobius().to_string(),
            json!({ "mobius": factorization.mobius() }),
        ),
        "radical" => exact(
            factorization.radical().to_string(),
            json!({ "radical": factorization.radical().to_string() }),
        ),
        "squarefree" => exact(
            factorization.is_squarefree().to_string(),
            json!({ "squarefree": factorization.is_squarefree() }),
        ),
        "divisor-count" => exact(
            factorization.divisor_count().to_string(),
            json!({ "divisor_count": factorization.divisor_count().to_string() }),
        ),
        "divisor-sum" => exact(
            factorization.divisor_sum().to_string(),
            json!({ "divisor_sum": factorization.divisor_sum().to_string() }),
        ),
        "divisors" => {
            let values = factorization.divisors();
            exact(
                values
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join(" "),
                json!({ "divisors": values.iter().map(u64::to_string).collect::<Vec<_>>() }),
            )
        }
        _ => unreachable!(),
    }
}

fn assessment_words(assessment: PrimalityAssessment) -> (&'static str, &'static str) {
    match assessment {
        PrimalityAssessment::Neither => ("neither", "neither"),
        PrimalityAssessment::Composite => ("composite", "exact"),
        PrimalityAssessment::PrimeExact => ("prime", "exact"),
        PrimalityAssessment::ExactProofIncomplete => ("proof_incomplete", "proof_incomplete"),
        PrimalityAssessment::ProbablePrime => ("probable_prime", "probable"),
    }
}

fn exact(human: String, result: Value) -> Result<OperationResult, String> {
    Ok(OperationResult {
        human,
        result,
        exactness: "exact",
    })
}

fn one(values: &[String]) -> Result<&str, String> {
    require_arity(values, 1)?;
    Ok(&values[0])
}

fn two_u64(values: &[String]) -> Result<(u64, u64), String> {
    require_arity(values, 2)?;
    Ok((parse_u64(&values[0])?, parse_u64(&values[1])?))
}

fn require_arity(values: &[String], expected: usize) -> Result<(), String> {
    if values.len() == expected {
        Ok(())
    } else {
        Err(format!(
            "attesi {expected} argomenti, ricevuti {}",
            values.len()
        ))
    }
}

fn parse_u64(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("intero u64 non valido: {value}"))
}

fn parse_i128(value: &str) -> Result<i128, String> {
    value
        .parse::<i128>()
        .map_err(|_| format!("intero i128 non valido: {value}"))
}

fn modulus_value(value: u64) -> Result<Modulus, String> {
    Modulus::new(value).ok_or_else(|| "il modulo deve essere positivo".to_owned())
}
