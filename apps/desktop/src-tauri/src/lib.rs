use serde::{Deserialize, Serialize};
use swissmath_core::{
    Congruence, DecimalIntegerAnalysis, DecimalIntegerAnalysisError, LinearCongruence,
    LinearSolution, ModCtx, ModularFilter, ModularFilterBuild, ModularSieve, Modulus,
    MultiplicativeOrderResult, PrimalityAssessment, QuadraticError, ResidueError, ResidueSet,
    analyze_integer_decimal, crt_pair, jacobi_symbol, modular_square_roots, multiplicative_order,
    solve_linear_congruence, solve_linear_system,
};

#[derive(Debug, Serialize)]
pub struct ModularResult {
    pub modulus: String,
    pub a: String,
    pub b: String,
    pub sum: String,
    pub difference: String,
    pub product: String,
    pub power: String,
    pub inverse_a: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CrtResult {
    pub compatible: bool,
    pub residue: Option<String>,
    pub modulus: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ResidueResult {
    pub modulus: String,
    pub values: Vec<String>,
    pub len: String,
    pub query: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct LinearResult {
    pub original_modulus: String,
    pub normalized_a: String,
    pub normalized_b: String,
    pub gcd: String,
    pub reduced_a: Option<String>,
    pub reduced_b: Option<String>,
    pub reduced_modulus: String,
    pub inverse: Option<String>,
    pub solution_kind: String,
    pub residue: Option<String>,
    pub solution_modulus: Option<String>,
    pub solution_count: String,
    pub residues: Vec<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LinearRowInput {
    pub a: String,
    pub b: String,
    pub modulus: String,
}

#[derive(Debug, Serialize)]
pub struct SystemResult {
    pub rows: Vec<LinearResult>,
    pub solution_kind: String,
    pub residue: Option<String>,
    pub modulus: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SieveFilterInput {
    pub kind: String,
    pub modulus: String,
    #[serde(default)]
    pub residues: Vec<String>,
    pub a: Option<String>,
    pub b: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SieveCommandResult {
    pub start: String,
    pub end: String,
    pub total_values: String,
    pub normalized_filter_count: String,
    pub survivor_count: String,
    pub survivor_percentage: String,
    pub preview: Vec<String>,
    pub anchor_modulus: Option<String>,
    pub anchor_allowed_count: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct IntegerFactorResult {
    pub prime: String,
    pub exponent: String,
}

#[derive(Debug, Serialize)]
pub struct IntegerAnalysisResult {
    pub n: String,
    pub classification: String,
    pub exact: bool,
    pub primality: String,
    pub factors: Vec<IntegerFactorResult>,
    pub phi: Option<String>,
    pub lambda: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderCommandResult {
    pub a: String,
    pub modulus: String,
    pub exists: bool,
    pub order: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct QuadraticSymbolsResult {
    pub a: String,
    pub modulus: String,
    pub jacobi: String,
    pub legendre: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ModularRootsResult {
    pub a: String,
    pub modulus: String,
    pub exists: bool,
    pub root_count: String,
    pub roots: Vec<String>,
    pub message: String,
}

fn parse_u64(name: &str, value: &str) -> Result<u64, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{name}: inserisci un valore."));
    }
    value
        .parse::<u64>()
        .map_err(|_| format!("{name}: usa un intero non negativo compreso tra 0 e 2^64-1."))
}

fn parse_modulus(value: &str) -> Result<Modulus, String> {
    let value = parse_u64("Modulo", value)?;
    Modulus::new(value).ok_or_else(|| "Il modulo deve essere maggiore di zero.".to_owned())
}

fn number_theory_error_message(error: swissmath_core::NumberTheoryError) -> String {
    match error {
        swissmath_core::NumberTheoryError::ZeroUndefined => {
            "La fattorizzazione prima di 0 non è definita.".to_owned()
        }
        swissmath_core::NumberTheoryError::SearchFailed => {
            "La ricerca limitata dei fattori non ha trovato uno split verificato.".to_owned()
        }
        swissmath_core::NumberTheoryError::Overflow => {
            "Il risultato esatto non è rappresentabile in u64.".to_owned()
        }
        swissmath_core::NumberTheoryError::NonPrimeBase => {
            "La base della valutazione deve essere prima.".to_owned()
        }
    }
}

fn decimal_analysis_error_message(error: DecimalIntegerAnalysisError) -> String {
    match error {
        DecimalIntegerAnalysisError::Input(input) => match input {
            swissmath_core::PrimalityInputError::Empty => {
                "Numero n: inserisci un intero decimale.".to_owned()
            }
            swissmath_core::PrimalityInputError::InvalidDecimal => {
                "Numero n: usa solo cifre decimali.".to_owned()
            }
            swissmath_core::PrimalityInputError::Negative => {
                "Numero n: i valori negativi non sono supportati.".to_owned()
            }
        },
        DecimalIntegerAnalysisError::NumberTheory(error) => number_theory_error_message(error),
    }
}

fn quadratic_error_message(error: QuadraticError) -> String {
    match error {
        QuadraticError::ZeroModulus => "Il modulo deve essere maggiore di zero.".to_owned(),
        QuadraticError::JacobiRequiresOddModulus => {
            "Il simbolo di Jacobi richiede un modulo positivo dispari.".to_owned()
        }
        QuadraticError::PrimeModulusRequired => {
            "Questa operazione richiede un modulo primo dispari.".to_owned()
        }
        QuadraticError::NonCoprimeUnsupported => {
            "Il caso in cui a e il modulo non sono coprimi non è ancora incluso nel solutore generale.".to_owned()
        }
        QuadraticError::Arithmetic => {
            "Il calcolo quadratico non è rappresentabile nel dominio u64.".to_owned()
        }
        QuadraticError::Factorization(error) => number_theory_error_message(error),
    }
}

fn residue_error_message(error: ResidueError) -> &'static str {
    match error {
        ResidueError::OutOfRange => "contiene un residuo fuori dal modulo",
        ResidueError::ModulusMismatch => "usa moduli incompatibili",
        ResidueError::AllocationFailed => "richiede più memoria di quella disponibile",
    }
}

fn linear_solution_parts(
    solution: LinearSolution,
) -> (&'static str, Option<String>, Option<String>) {
    match solution {
        LinearSolution::None => ("none", None, None),
        LinearSolution::All => ("all", None, None),
        LinearSolution::Class(congruence) => (
            "class",
            Some(congruence.residue().to_string()),
            Some(congruence.modulus().get().to_string()),
        ),
    }
}

fn small_solution_residues(
    result: &swissmath_core::LinearSolveResult,
    original_modulus: Modulus,
) -> Vec<String> {
    const MAX_VISIBLE_RESIDUES: u64 = 1_000;
    if result.solution_count(original_modulus) > MAX_VISIBLE_RESIDUES {
        return Vec::new();
    }

    match result.solution {
        LinearSolution::None => Vec::new(),
        LinearSolution::All => (0..original_modulus.get())
            .map(|value| value.to_string())
            .collect(),
        LinearSolution::Class(congruence) => {
            let residue = u128::from(congruence.residue());
            let step = u128::from(congruence.modulus().get());
            (0..result.solution_count(original_modulus))
                .map(|index| (residue + step * u128::from(index)).to_string())
                .collect()
        }
    }
}

fn linear_result(equation: LinearCongruence) -> LinearResult {
    let original_modulus = equation.modulus();
    let result = solve_linear_congruence(equation);
    let (solution_kind, residue, solution_modulus) = linear_solution_parts(result.solution);
    let message = match result.solution {
        LinearSolution::None => {
            "Nessuna soluzione: il massimo comune divisore non divide il termine noto.".to_owned()
        }
        LinearSolution::All => "Ogni intero è soluzione.".to_owned(),
        LinearSolution::Class(_) => "Soluzione ridotta a una classe di congruenza.".to_owned(),
    };

    LinearResult {
        original_modulus: original_modulus.get().to_string(),
        normalized_a: result.normalized_a.to_string(),
        normalized_b: result.normalized_b.to_string(),
        gcd: result.gcd.to_string(),
        reduced_a: result.reduced_a.map(|value| value.to_string()),
        reduced_b: result.reduced_b.map(|value| value.to_string()),
        reduced_modulus: result.reduced_modulus.to_string(),
        inverse: result.inverse.map(|value| value.to_string()),
        solution_kind: solution_kind.to_owned(),
        residue,
        solution_modulus,
        solution_count: result.solution_count(original_modulus).to_string(),
        residues: small_solution_residues(&result, original_modulus),
        message,
    }
}

fn parse_linear_row(index: usize, row: LinearRowInput) -> Result<LinearCongruence, String> {
    let a = parse_u64(&format!("a nella riga {index}"), &row.a)?;
    let b = parse_u64(&format!("b nella riga {index}"), &row.b)?;
    let modulus = parse_modulus(&row.modulus)?;
    Ok(LinearCongruence::new(a, b, modulus))
}

#[tauri::command]
fn solve_linear(a: String, b: String, modulus: String) -> Result<LinearResult, String> {
    let equation = LinearCongruence::new(
        parse_u64("a", &a)?,
        parse_u64("b", &b)?,
        parse_modulus(&modulus)?,
    );
    Ok(linear_result(equation))
}

#[tauri::command]
fn solve_system(rows: Vec<LinearRowInput>) -> Result<SystemResult, String> {
    let equations = rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| parse_linear_row(index + 1, row))
        .collect::<Result<Vec<_>, _>>()?;
    let explanations = equations
        .iter()
        .copied()
        .map(linear_result)
        .collect::<Vec<_>>();
    let solution = solve_linear_system(equations).map_err(|_| {
        "Il sistema è compatibile, ma il periodo combinato non è rappresentabile in u64.".to_owned()
    })?;
    let (solution_kind, residue, modulus) = linear_solution_parts(solution);
    let message = match solution {
        LinearSolution::None => "Nessuna soluzione comune.".to_owned(),
        LinearSolution::All => "Ogni intero è soluzione del sistema.".to_owned(),
        LinearSolution::Class(_) => {
            "Le classi sono compatibili e sono state combinate con CRT.".to_owned()
        }
    };

    Ok(SystemResult {
        rows: explanations,
        solution_kind: solution_kind.to_owned(),
        residue,
        modulus,
        message,
    })
}

fn parse_sieve_filter(
    index: usize,
    input: SieveFilterInput,
) -> Result<Option<ModularFilter>, String> {
    let modulus = parse_modulus(&input.modulus)?;
    let label = format!("Filtro {index}");
    match input.kind.as_str() {
        "allowed" => ModularFilter::from_allowed(modulus, parse_residues(&label, input.residues)?)
            .map(Some)
            .map_err(|error| format!("{label}: {}.", residue_error_message(error))),
        "excluded" => {
            ModularFilter::from_excluded(modulus, parse_residues(&label, input.residues)?)
                .map(Some)
                .map_err(|error| format!("{label}: {}.", residue_error_message(error)))
        }
        "linear" => {
            let a = input
                .a
                .ok_or_else(|| format!("{label}: inserisci il coefficiente a."))?;
            let b = input
                .b
                .ok_or_else(|| format!("{label}: inserisci il termine b."))?;
            let equation = LinearCongruence::new(parse_u64("a", &a)?, parse_u64("b", &b)?, modulus);
            match ModularFilter::from_linear_congruence(equation) {
                ModularFilterBuild::None => Ok(None),
                ModularFilterBuild::All => Ok(Some(
                    ModularFilter::from_allowed(Modulus::new(1).expect("one is a modulus"), [0])
                        .expect("modulus one is a valid full filter"),
                )),
                ModularFilterBuild::Filter(filter) => Ok(Some(filter)),
            }
        }
        _ => Err(format!("{label}: tipo di filtro non riconosciuto.")),
    }
}

#[tauri::command]
fn run_sieve(
    start: String,
    end: String,
    preview: String,
    filters: Vec<SieveFilterInput>,
) -> Result<SieveCommandResult, String> {
    let start = parse_u64("Da", &start)?;
    let end = parse_u64("A", &end)?;
    if start > end {
        return Err("L'intervallo deve soddisfare Da ≤ A.".to_owned());
    }
    let preview = parse_u64("Anteprima", &preview)?;
    if preview > 1_000 {
        return Err("L'anteprima può contenere al massimo 1.000 valori.".to_owned());
    }

    let mut impossible = false;
    let mut built_filters = Vec::new();
    for (index, input) in filters.into_iter().enumerate() {
        match parse_sieve_filter(index + 1, input)? {
            Some(filter) if filter.modulus().get() == 1 && filter.is_full() => {}
            Some(filter) => built_filters.push(filter),
            None => impossible = true,
        }
    }
    if impossible {
        built_filters.push(
            ModularFilter::from_allowed(Modulus::new(1).expect("one is a modulus"), [])
                .expect("modulus one is a valid empty filter"),
        );
    }

    let sieve = ModularSieve::new(built_filters)
        .map_err(|_| "Non è stato possibile costruire i filtri modulari.".to_owned())?;
    let result = sieve
        .search(start, end, preview as usize)
        .map_err(|_| "Intervallo non valido.".to_owned())?;
    let percentage = (result.survivor_count as f64 / result.total_values as f64) * 100.0;
    let message = if result.survivor_count == 0 {
        "Nessun valore compatibile nell'intervallo.".to_owned()
    } else {
        "Ricerca completata.".to_owned()
    };

    Ok(SieveCommandResult {
        start: result.start.to_string(),
        end: result.end.to_string(),
        total_values: result.total_values.to_string(),
        normalized_filter_count: result.normalized_filter_count.to_string(),
        survivor_count: result.survivor_count.to_string(),
        survivor_percentage: format!("{percentage:.2}%").replace('.', ","),
        preview: result
            .preview
            .into_iter()
            .map(|value| value.to_string())
            .collect(),
        anchor_modulus: result.anchor_modulus.map(|value| value.get().to_string()),
        anchor_allowed_count: result
            .anchor_modulus
            .map(|_| result.anchor_allowed_count.to_string()),
        message,
    })
}

#[tauri::command]
fn analyze_integer(n: String) -> Result<IntegerAnalysisResult, String> {
    match analyze_integer_decimal(&n).map_err(decimal_analysis_error_message)? {
        DecimalIntegerAnalysis::Neither { n } => Ok(IntegerAnalysisResult {
            n,
            classification: "né_primo_né_composto".to_owned(),
            exact: false,
            primality: "né_primo_né_composto".to_owned(),
            factors: Vec::new(),
            phi: None,
            lambda: None,
            note: Some(
                "0 non è né primo né composto; la fattorizzazione prima non è definita.".to_owned(),
            ),
        }),
        DecimalIntegerAnalysis::Exact(analysis) => {
            let classification = match analysis.classification {
                swissmath_core::IntegerClassification::Unit => "unità",
                swissmath_core::IntegerClassification::Prime => "primo",
                swissmath_core::IntegerClassification::Composite => "composto",
            };
            Ok(IntegerAnalysisResult {
                n: analysis.n.to_string(),
                classification: classification.to_owned(),
                exact: true,
                primality: primality_label(analysis.primality).to_owned(),
                factors: analysis
                    .factorization
                    .factors()
                    .iter()
                    .map(|factor| IntegerFactorResult {
                        prime: factor.prime.to_string(),
                        exponent: factor.exponent.to_string(),
                    })
                    .collect(),
                phi: Some(analysis.phi.to_string()),
                lambda: Some(analysis.lambda.to_string()),
                note: None,
            })
        }
        DecimalIntegerAnalysis::U128 { n, primality } => {
            let classification = match primality {
                PrimalityAssessment::Composite => "composto",
                PrimalityAssessment::PrimeExact => "primo_esatto",
                PrimalityAssessment::ExactProofIncomplete => "prova_incompleta",
                PrimalityAssessment::Neither => "né_primo_né_composto",
                PrimalityAssessment::ProbablePrime => "probabile_primo",
            };
            Ok(IntegerAnalysisResult {
                n,
                classification: classification.to_owned(),
                exact: false,
                primality: primality_label(primality).to_owned(),
                factors: Vec::new(),
                phi: None,
                lambda: None,
                note: if primality == PrimalityAssessment::ExactProofIncomplete {
                    Some(
                        "SwissMath non è riuscito a completare una prova esatta con il percorso rapido disponibile.".to_owned(),
                    )
                } else {
                    Some(
                        "Per numeri oltre 64 bit è disponibile soltanto la valutazione di primalità; fattorizzazione, φ, λ e ordine restano fuori dominio.".to_owned(),
                    )
                },
            })
        }
        DecimalIntegerAnalysis::Large { n, primality } => {
            let (classification, primality_label) = match primality {
                PrimalityAssessment::Composite => ("composto", "composito"),
                PrimalityAssessment::PrimeExact => ("primo", "primo_esatto"),
                PrimalityAssessment::Neither => ("né_primo_né_composto", "né_primo_né_composto"),
                PrimalityAssessment::ExactProofIncomplete => {
                    ("prova_incompleta", "prova_incompleta")
                }
                PrimalityAssessment::ProbablePrime => ("probabile_primo", "probabile_primo"),
            };
            Ok(IntegerAnalysisResult {
                n,
                classification: classification.to_owned(),
                exact: false,
                primality: primality_label.to_owned(),
                factors: Vec::new(),
                phi: None,
                lambda: None,
                note: Some(
                    "Per numeri oltre 128 bit SwissMath esegue un test Baillie–PSW: un risultato \"probabile primo\" non costituisce una prova formale di primalità.".to_owned(),
                ),
            })
        }
    }
}

fn primality_label(assessment: PrimalityAssessment) -> &'static str {
    match assessment {
        PrimalityAssessment::Neither => "né_primo_né_composto",
        PrimalityAssessment::Composite => "composito",
        PrimalityAssessment::PrimeExact => "primo_esatto",
        PrimalityAssessment::ExactProofIncomplete => "prova_incompleta",
        PrimalityAssessment::ProbablePrime => "probabile_primo",
    }
}

fn parse_i128(name: &str, value: &str) -> Result<i128, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{name}: inserisci un intero."));
    }
    value
        .parse::<i128>()
        .map_err(|_| format!("{name}: usa un intero compreso nel dominio signed i128."))
}

#[tauri::command]
fn calculate_quadratic_symbols(
    a: String,
    modulus: String,
) -> Result<QuadraticSymbolsResult, String> {
    let a_value = parse_i128("a", &a)?;
    let modulus_value = parse_u64("Modulo n", &modulus)?;
    let jacobi = jacobi_symbol(a_value, modulus_value).map_err(quadratic_error_message)?;
    let legendre = if modulus_value >= 3 && swissmath_core::is_prime(modulus_value) {
        Some(jacobi)
    } else {
        None
    };
    Ok(QuadraticSymbolsResult {
        a: a_value.to_string(),
        modulus: modulus_value.to_string(),
        jacobi: jacobi.to_string(),
        legendre: legendre.map(|value| value.to_string()),
        message: if legendre.is_some() {
            "Simboli di Jacobi e Legendre calcolati.".to_owned()
        } else {
            "Simbolo di Jacobi calcolato.".to_owned()
        },
    })
}

#[tauri::command]
fn find_modular_roots(a: String, modulus: String) -> Result<ModularRootsResult, String> {
    const ROOT_PREVIEW_LIMIT: usize = 100;
    let a_value = parse_i128("a", &a)?;
    let modulus_value = parse_u64("Modulo n", &modulus)?;
    let roots = modular_square_roots(a_value, modulus_value).map_err(quadratic_error_message)?;
    let exists = !roots.is_empty();
    let root_count = roots.len().to_string();
    let preview = roots
        .iter()
        .take(ROOT_PREVIEW_LIMIT)
        .map(|root| root.to_string())
        .collect();
    Ok(ModularRootsResult {
        a: a_value.to_string(),
        modulus: modulus_value.to_string(),
        exists,
        root_count,
        roots: preview,
        message: if exists {
            "Radici modulari calcolate.".to_owned()
        } else {
            "Nessuna radice quadrata nel modulo indicato.".to_owned()
        },
    })
}

#[tauri::command]
fn calculate_multiplicative_order(
    a: String,
    modulus: String,
) -> Result<OrderCommandResult, String> {
    let a_value = parse_u64("a", &a)?;
    let modulus_value = parse_u64("Modulo n", &modulus)?;
    match multiplicative_order(a_value, modulus_value).map_err(number_theory_error_message)? {
        MultiplicativeOrderResult::Exists(order) => Ok(OrderCommandResult {
            a: a_value.to_string(),
            modulus: modulus_value.to_string(),
            exists: true,
            order: Some(order.to_string()),
            message: "Ordine moltiplicativo calcolato.".to_owned(),
        }),
        MultiplicativeOrderResult::DoesNotExist => Ok(OrderCommandResult {
            a: a_value.to_string(),
            modulus: modulus_value.to_string(),
            exists: false,
            order: None,
            message: "L'ordine non esiste: a e n non sono coprimi.".to_owned(),
        }),
    }
}

#[tauri::command]
fn calculate_modular(
    modulus: String,
    a: String,
    b: String,
    exponent: String,
) -> Result<ModularResult, String> {
    let modulus = parse_modulus(&modulus)?;
    let modulus_value = modulus.get();
    let a = parse_u64("a", &a)? % modulus_value;
    let b = parse_u64("b", &b)? % modulus_value;
    let exponent = parse_u64("Esponente", &exponent)?;
    let context = ModCtx::new(modulus);

    Ok(ModularResult {
        modulus: modulus_value.to_string(),
        a: a.to_string(),
        b: b.to_string(),
        sum: context.add(a, b).to_string(),
        difference: context.sub(a, b).to_string(),
        product: context.mul(a, b).to_string(),
        power: context.pow(a, exponent).to_string(),
        inverse_a: context.inv(a).map(|value| value.to_string()),
    })
}

#[tauri::command]
fn calculate_crt(
    residue_a: String,
    modulus_a: String,
    residue_b: String,
    modulus_b: String,
) -> Result<CrtResult, String> {
    let modulus_a = parse_modulus(&modulus_a)?;
    let modulus_b = parse_modulus(&modulus_b)?;
    let residue_a = parse_u64("Residuo A", &residue_a)?;
    let residue_b = parse_u64("Residuo B", &residue_b)?;
    let left = Congruence::new(residue_a, modulus_a);
    let right = Congruence::new(residue_b, modulus_b);

    match crt_pair(left, right) {
        Ok(Some(result)) => Ok(CrtResult {
            compatible: true,
            residue: Some(result.residue().to_string()),
            modulus: Some(result.modulus().get().to_string()),
            message: "Le due congruenze sono compatibili.".to_owned(),
        }),
        Ok(None) => Ok(CrtResult {
            compatible: false,
            residue: None,
            modulus: None,
            message: "Le due congruenze sono incompatibili.".to_owned(),
        }),
        Err(error) => Ok(CrtResult {
            compatible: true,
            residue: None,
            modulus: None,
            message: format!(
                "Le congruenze sono compatibili, ma il minimo comune multiplo non è rappresentabile: {error}."
            ),
        }),
    }
}

fn parse_residues(name: &str, values: Vec<String>) -> Result<Vec<u64>, String> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| parse_u64(&format!("{name}[{}]", index + 1), value))
        .collect()
}

fn set_values(set: &ResidueSet) -> Vec<String> {
    set.iter().map(|value| value.to_string()).collect()
}

#[tauri::command]
fn calculate_residues(
    modulus: String,
    left: Vec<String>,
    right: Vec<String>,
    operation: String,
) -> Result<ResidueResult, String> {
    let modulus = parse_modulus(&modulus)?;
    let modulus_value = modulus.get();
    let left = ResidueSet::from_iter(modulus, parse_residues("A", left)?)
        .map_err(|error| error.to_string())?;
    let right = ResidueSet::from_iter(modulus, parse_residues("B", right)?)
        .map_err(|error| error.to_string())?;

    let result = match operation.as_str() {
        "intersection" => {
            let set = left
                .intersection(&right)
                .map_err(|error| error.to_string())?;
            ResidueResult {
                modulus: modulus_value.to_string(),
                values: set_values(&set),
                len: set.len().to_string(),
                query: None,
                message: "Intersezione calcolata.".to_owned(),
            }
        }
        "union" => {
            let set = left.union(&right).map_err(|error| error.to_string())?;
            ResidueResult {
                modulus: modulus_value.to_string(),
                values: set_values(&set),
                len: set.len().to_string(),
                query: None,
                message: "Unione calcolata.".to_owned(),
            }
        }
        "difference" => {
            let set = left.difference(&right).map_err(|error| error.to_string())?;
            ResidueResult {
                modulus: modulus_value.to_string(),
                values: set_values(&set),
                len: set.len().to_string(),
                query: None,
                message: "Differenza A \\ B calcolata.".to_owned(),
            }
        }
        "complement" => {
            let set = left.complement();
            ResidueResult {
                modulus: modulus_value.to_string(),
                values: set_values(&set),
                len: set.len().to_string(),
                query: None,
                message: "Complemento di A calcolato.".to_owned(),
            }
        }
        "intersection_count" => ResidueResult {
            modulus: modulus_value.to_string(),
            values: Vec::new(),
            len: left
                .intersection_count(&right)
                .map_err(|error| error.to_string())?
                .to_string(),
            query: Some("count".to_owned()),
            message: "Cardinalità dell'intersezione.".to_owned(),
        },
        "intersects" => ResidueResult {
            modulus: modulus_value.to_string(),
            values: Vec::new(),
            len: left
                .intersects(&right)
                .map_err(|error| error.to_string())?
                .to_string(),
            query: Some("boolean".to_owned()),
            message: "L'intersezione è non vuota?".to_owned(),
        },
        "is_subset_of" => ResidueResult {
            modulus: modulus_value.to_string(),
            values: Vec::new(),
            len: left
                .is_subset_of(&right)
                .map_err(|error| error.to_string())?
                .to_string(),
            query: Some("boolean".to_owned()),
            message: "A è sottoinsieme di B?".to_owned(),
        },
        _ => return Err("Operazione ResidueSet non riconosciuta.".to_owned()),
    };

    Ok(result)
}

pub fn run() {
    runner::run();
}

mod runner {
    pub fn run() {
        tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![
                super::calculate_modular,
                super::calculate_crt,
                super::calculate_residues,
                super::solve_linear,
                super::solve_system,
                super::run_sieve,
                super::analyze_integer,
                super::calculate_multiplicative_order,
                super::calculate_quadratic_symbols,
                super::find_modular_roots
            ])
            .run(tauri::generate_context!())
            .expect("errore durante l'avvio di SwissMath");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LinearRowInput, SieveFilterInput, analyze_integer, calculate_crt, calculate_modular,
        calculate_multiplicative_order, calculate_quadratic_symbols, calculate_residues,
        find_modular_roots, run_sieve, solve_linear, solve_system,
    };

    #[test]
    fn modular_command_returns_canonical_exact_strings() {
        let result = calculate_modular("7".into(), "10".into(), "5".into(), "4".into()).unwrap();
        assert_eq!(result.a, "3");
        assert_eq!(result.b, "5");
        assert_eq!(result.sum, "1");
        assert_eq!(result.product, "1");
        assert_eq!(result.power, "4");
        assert_eq!(result.inverse_a.as_deref(), Some("5"));
    }

    #[test]
    fn crt_command_distinguishes_incompatibility() {
        let compatible = calculate_crt("2".into(), "3".into(), "3".into(), "5".into()).unwrap();
        assert!(compatible.compatible);
        assert_eq!(compatible.residue.as_deref(), Some("8"));
        assert_eq!(compatible.modulus.as_deref(), Some("15"));

        let incompatible = calculate_crt("1".into(), "2".into(), "0".into(), "4".into()).unwrap();
        assert!(!incompatible.compatible);
        assert!(incompatible.residue.is_none());
    }

    #[test]
    fn residue_command_returns_sorted_deduplicated_values() {
        let result = calculate_residues(
            "12".into(),
            vec!["9".into(), "1".into(), "1".into(), "4".into()],
            vec!["4".into(), "8".into()],
            "intersection".into(),
        )
        .unwrap();
        assert_eq!(result.values, vec!["4"]);
        assert_eq!(result.len, "1");
    }

    #[test]
    fn linear_command_exposes_reduced_class_and_small_residues() {
        let result = solve_linear("14".into(), "8".into(), "30".into()).unwrap();
        assert_eq!(result.solution_kind, "class");
        assert_eq!(result.residue.as_deref(), Some("7"));
        assert_eq!(result.solution_modulus.as_deref(), Some("15"));
        assert_eq!(result.solution_count, "2");
        assert_eq!(result.residues, vec!["7", "22"]);
        assert_eq!(result.inverse.as_deref(), Some("13"));
    }

    #[test]
    fn linear_command_reports_no_solution_and_system_outcomes() {
        let impossible = solve_linear("6".into(), "5".into(), "15".into()).unwrap();
        assert_eq!(impossible.solution_kind, "none");
        assert_eq!(impossible.solution_count, "0");

        let compatible = solve_system(vec![
            LinearRowInput {
                a: "14".into(),
                b: "8".into(),
                modulus: "30".into(),
            },
            LinearRowInput {
                a: "3".into(),
                b: "6".into(),
                modulus: "15".into(),
            },
        ])
        .unwrap();
        assert_eq!(compatible.solution_kind, "class");
        assert_eq!(compatible.residue.as_deref(), Some("7"));
        assert_eq!(compatible.modulus.as_deref(), Some("15"));

        let incompatible = solve_system(vec![
            LinearRowInput {
                a: "1".into(),
                b: "1".into(),
                modulus: "2".into(),
            },
            LinearRowInput {
                a: "1".into(),
                b: "0".into(),
                modulus: "4".into(),
            },
        ])
        .unwrap();
        assert_eq!(incompatible.solution_kind, "none");
    }

    #[test]
    fn sieve_command_returns_exact_decimal_stats() {
        let result = run_sieve(
            "0".into(),
            "100".into(),
            "5".into(),
            vec![SieveFilterInput {
                kind: "linear".into(),
                modulus: "30".into(),
                residues: Vec::new(),
                a: Some("14".into()),
                b: Some("8".into()),
            }],
        )
        .unwrap();
        assert_eq!(result.total_values, "101");
        assert_eq!(result.survivor_count, "7");
        assert_eq!(result.preview, vec!["7", "22", "37", "52", "67"]);
        assert_eq!(result.anchor_modulus.as_deref(), Some("15"));
    }

    #[test]
    fn integer_analysis_command_returns_one_exact_bundle() {
        let result = analyze_integer("360".into()).unwrap();
        assert_eq!(result.classification, "composto");
        assert_eq!(result.phi.as_deref(), Some("96"));
        assert_eq!(result.lambda.as_deref(), Some("12"));
        assert_eq!(
            result
                .factors
                .iter()
                .map(|factor| (factor.prime.as_str(), factor.exponent.as_str()))
                .collect::<Vec<_>>(),
            vec![("2", "3"), ("3", "2"), ("5", "1")]
        );

        let prime = analyze_integer("2".into()).unwrap();
        assert_eq!(prime.classification, "primo");
        assert_eq!(prime.phi.as_deref(), Some("1"));
        assert_eq!(prime.lambda.as_deref(), Some("1"));
    }

    #[test]
    fn multiplicative_order_command_exposes_exists_and_missing() {
        let result = calculate_multiplicative_order("3".into(), "7".into()).unwrap();
        assert!(result.exists);
        assert_eq!(result.order.as_deref(), Some("6"));

        let missing = calculate_multiplicative_order("6".into(), "9".into()).unwrap();
        assert!(!missing.exists);
        assert!(missing.order.is_none());
    }

    #[test]
    fn integer_command_routes_large_primality_without_fake_factors() {
        let result = analyze_integer("170141183460469231731687303715884105727".into()).unwrap();
        assert!(!result.exact);
        assert_eq!(result.classification, "prova_incompleta");
        assert_eq!(result.primality, "prova_incompleta");
        assert!(result.factors.is_empty());
        assert!(result.phi.is_none());
        assert!(result.lambda.is_none());
        assert!(result.note.as_deref().unwrap().contains("prova esatta"));
    }

    #[test]
    fn integer_command_exposes_neither_semantics_for_zero_and_one() {
        let zero = analyze_integer("0".into()).unwrap();
        assert_eq!(zero.classification, "né_primo_né_composto");
        assert_eq!(zero.primality, "né_primo_né_composto");
        assert!(zero.phi.is_none());

        let one = analyze_integer("1".into()).unwrap();
        assert_eq!(one.classification, "unità");
        assert_eq!(one.primality, "né_primo_né_composto");
        assert_eq!(one.phi.as_deref(), Some("1"));
    }

    #[test]
    fn integer_command_exposes_u128_exact_and_composite_outcomes() {
        let prime = analyze_integer("39614081257132185645928677377".into()).unwrap();
        assert!(!prime.exact);
        assert_eq!(prime.classification, "primo_esatto");
        assert_eq!(prime.primality, "primo_esatto");

        let composite = analyze_integer("18446744073709551618".into()).unwrap();
        assert!(!composite.exact);
        assert_eq!(composite.classification, "composto");
        assert_eq!(composite.primality, "composito");
    }

    #[test]
    fn integer_command_keeps_probable_label_above_u128() {
        let value = "6864797660130609714981900799081393217269435300143305409394463459185543183397656052122559640661454554977296311391480858037121987999716643812574028291115057151";
        let result = analyze_integer(value.into()).unwrap();
        assert!(!result.exact);
        assert_eq!(result.classification, "probabile_primo");
        assert_eq!(result.primality, "probabile_primo");
        assert!(result.note.as_deref().unwrap().contains("Baillie"));
    }

    #[test]
    fn quadratic_commands_cover_symbols_roots_and_domain_error() {
        let symbols = calculate_quadratic_symbols("5".into(), "11".into()).unwrap();
        assert_eq!(symbols.jacobi, "1");
        assert_eq!(symbols.legendre.as_deref(), Some("1"));

        let roots = find_modular_roots("10".into(), "13".into()).unwrap();
        assert!(roots.exists);
        assert_eq!(roots.root_count, "2");
        assert_eq!(roots.roots, vec!["6", "7"]);

        let unsupported = find_modular_roots("6".into(), "15".into());
        assert!(unsupported.unwrap_err().contains("non sono coprimi"));
    }
}
