import init, {
  wasm_analyze_integer,
  wasm_calculate_modular,
  wasm_calculate_residues,
  wasm_run_sieve,
  wasm_run_tool,
  wasm_solve_linear,
} from './pkg/swissmath_web.js';

const f = (name, label, value, help = '', type = 'text') => ({ name, label, value, help, type });
const t = (id, name, description, example, fields, command = 'tool') => ({ id, name, description, example, fields, command });

const catalog = {
  arithmetic: {
    title: 'Aritmetica',
    tools: [
      t('gcd', 'Massimo comune divisore', 'Calcola il MCD di due interi non negativi.', 'Esempio: gcd(40902, 24140) = 34', [f('a', 'Intero a', '40902'), f('b', 'Intero b', '24140')]),
      t('lcm', 'Minimo comune multiplo', 'Calcola il mcm con ordinamento protetto dall’overflow.', 'Esempio: lcm(21, 6) = 42', [f('a', 'Intero a', '21'), f('b', 'Intero b', '6')]),
      t('xgcd', 'Euclide esteso', 'Trova g, x e y tali che ax + by = g.', 'Esempio: 240x + 46y = 2', [f('a', 'Intero non negativo a', '240'), f('b', 'Intero non negativo b', '46')]),
      t('powmod', 'Potenza modulare', 'Calcola aⁿ mod m senza costruire la potenza completa.', 'Esempio: 7¹²⁸ mod 13', [f('a', 'Base a', '7'), f('exponent', 'Esponente n', '128'), f('modulus', 'Modulo m', '13')]),
      t('invmod', 'Inverso modulare', 'Trova a⁻¹ mod m quando gcd(a,m)=1.', 'Esempio: 7⁻¹ mod 26 = 15', [f('a', 'Valore a', '7'), f('modulus', 'Modulo m', '26')]),
      t('crt', 'Teorema cinese del resto', 'Combina un sistema di congruenze anche con moduli non coprimi.', 'Una riga per congruenza: residuo, modulo', [f('congruences', 'Congruenze', '2, 3\n3, 5\n2, 7', 'Righe: residuo, modulo', 'textarea')]),
      t('iroot', 'Radice intera', 'Calcola la radice n-esima inferiore e indica se è esatta.', 'Esempio: radice cubica intera di 80 = 4', [f('n', 'Intero n', '80'), f('degree', 'Indice della radice', '3')]),
      t('perfect-power', 'Potenza perfetta', 'Riconosce la rappresentazione canonica aᵏ con a>1 e k>1.', 'Esempio: 64 = 2⁶', [f('n', 'Intero n', '64')]),
      t('base-convert', 'Conversione di base', 'Converte interi tra basi da 2 a 36.', 'Esempio: ff in base 16 → 11111111 in base 2', [f('value', 'Valore', 'ff'), f('from_base', 'Base iniziale', '16'), f('to_base', 'Base finale', '2')]),
      t('modular', 'Calcolatore modulare', 'Esegue insieme somma, differenza, prodotto, potenza e inverso.', 'Operazioni in ℤ/mℤ', [f('modulus', 'Modulo', '7'), f('a', 'Valore a', '3'), f('b', 'Valore b', '5'), f('exponent', 'Esponente', '4')], 'modular'),
      t('residue-set', 'Insiemi di residui', 'Combina due insiemi materializzati nello stesso modulo.', 'Liste separate da virgole', [f('modulus', 'Modulo', '12'), f('left', 'Insieme A', '0,2,4,6'), f('right', 'Insieme B', '2,3,6,9'), f('operation', 'Operazione', 'intersection', 'intersection, union o difference')], 'residues'),
    ],
  },
  'number-theory': {
    title: 'Teoria dei numeri',
    tools: [
      t('isprime', 'Test di primalità u64', 'Fornisce un risultato deterministico nell’intero dominio u64.', 'Prime, Composite oppure Neither', [f('n', 'Intero n', '1000000007')]),
      t('nextprime', 'Primo successivo', 'Trova il minimo primo strettamente maggiore di n.', 'Esempio: dopo 1000 viene 1009', [f('n', 'Intero n', '1000')]),
      t('previousprime', 'Primo precedente', 'Trova il massimo primo strettamente minore di n.', 'Per n≤2 non esiste un risultato', [f('n', 'Intero n', '1000')]),
      t('factor', 'Fattorizzazione intera', 'Fattorizza un u64 con trial division e Pollard–Brent.', 'Esempio: 360 = 2³·3²·5', [f('n', 'Intero n', '360')]),
      t('divisors', 'Divisori', 'Calcola numero, somma ed eventualmente l’elenco dei divisori.', 'Gli elenchi molto grandi non vengono materializzati', [f('n', 'Intero n', '360')]),
      t('totient', 'Totiente di Eulero', 'Calcola φ(n) riusando una sola fattorizzazione.', 'Esempio: φ(360) = 96', [f('n', 'Intero n', '360')]),
      t('mobius', 'Funzione di Möbius', 'Calcola μ(n) dalla fattorizzazione prima.', 'Valori possibili: −1, 0, 1', [f('n', 'Intero n', '30')]),
      t('radical', 'Radicale', 'Calcola il prodotto dei divisori primi distinti.', 'Esempio: rad(360) = 30', [f('n', 'Intero n', '360')]),
      t('squarefree', 'Test squarefree', 'Verifica se nessun quadrato primo divide n.', 'Risultato esatto booleano', [f('n', 'Intero n', '30')]),
      t('divisor-count', 'Numero dei divisori', 'Calcola τ(n) dalla fattorizzazione.', 'Esempio: τ(360) = 24', [f('n', 'Intero n', '360')]),
      t('divisor-sum', 'Somma dei divisori', 'Calcola σ(n) esattamente in u128.', 'Esempio: σ(360) = 1170', [f('n', 'Intero n', '360')]),
      t('valuation', 'Valutazione p-adica', 'Calcola vₚ(n) richiedendo p primo.', 'Per n=0 restituisce ∞', [f('n', 'Intero n', '81'), f('p', 'Primo p', '3')]),
      t('jacobi', 'Simbolo di Jacobi', 'Calcola il simbolo senza fattorizzare il modulo.', 'Il modulo deve essere positivo e dispari', [f('a', 'Valore a', '5'), f('modulus', 'Modulo n', '11')]),
      t('sqrtmod', 'Radici modulari', 'Trova le radici quadrate nei domini esatti supportati dal Core.', 'Esempio: x² ≡ 10 mod 13 → 6, 7', [f('a', 'Valore a', '10'), f('modulus', 'Modulo n', '13')]),
      t('multiplicative-order', 'Ordine moltiplicativo', 'Trova il minimo k>0 con aᵏ≡1 mod n.', 'Richiede gcd(a,n)=1', [f('a', 'Valore a', '2'), f('modulus', 'Modulo n', '9')]),
      t('integer-analysis', 'Analisi completa di un intero', 'Classifica primalità e, nel dominio u64, espone fattori, φ e λ.', 'Le etichette distinguono prova esatta e probabile primo', [f('n', 'Intero decimale', '360')], 'integer'),
      t('linear-congruence', 'Congruenza lineare', 'Risolvi ax≡b mod m con spiegazione della classe soluzione.', 'Esempio: 6x ≡ 8 mod 14', [f('a', 'Coefficiente a', '6'), f('b', 'Termine b', '8'), f('modulus', 'Modulo m', '14')], 'linear'),
      t('modular-sieve', 'Filtro modulare', 'Filtra un intervallo escludendo residui periodici.', 'Esempio: escludi 0 mod 2 tra 1 e 100', [f('start', 'Inizio', '1'), f('end', 'Fine', '100'), f('modulus', 'Modulo filtro', '2'), f('residues', 'Residui esclusi', '0'), f('preview', 'Anteprima massima', '25')], 'sieve'),
    ],
  },
  fractions: {
    title: 'Frazioni e ricostruzione',
    tools: [
      t('contfrac', 'Frazione continua', 'Converte esattamente una frazione o un decimale nei suoi quozienti.', 'Esempio: 355/113 → [3; 7, 16]', [f('value', 'Valore', '355/113')]),
      t('rationalize', 'Razionalizza', 'Trova la frazione più vicina entro un denominatore massimo.', 'π con denominatore ≤10000 → 355/113', [f('value', 'Valore decimale', '3.141592653589793'), f('max_denominator', 'Denominatore massimo', '10000')]),
      t('rational-reconstruct', 'Ricostruzione razionale', 'Ricostruisce a/b da un residuo, verificando limiti e identità modulare.', 'Restituisce errore se i vincoli non determinano una soluzione valida', [f('residue', 'Residuo r', '7'), f('modulus', 'Modulo m', '101'), f('bound', 'Limite |a|, |b|', '10')]),
    ],
  },
  polynomials: {
    title: 'Polinomi',
    tools: [
      t('poly-eval', 'Valutazione polinomiale', 'Valuta coefficienti esatti con il metodo di Horner.', 'Coefficienti in ordine crescente: c₀,c₁,…', [f('coefficients', 'Coefficienti', '1, 2, 3'), f('x', 'Valore x', '4')]),
      t('poly-gcd', 'MCD polinomiale', 'Calcola il MCD monico di due polinomi su ℚ.', 'Coefficienti in ordine crescente', [f('left', 'Polinomio A', '-2, 1, 1'), f('right', 'Polinomio B', '-3, 2, 1')]),
      t('interpolate', 'Interpolazione esatta', 'Ricostruisce il polinomio che passa per punti con ascisse distinte.', 'Una riga per punto: x, y', [f('points', 'Punti', '0, 1\n1, 6\n2, 17', '', 'textarea')]),
      t('finite-differences', 'Differenze finite', 'Costruisce la tabella e rileva progressioni polinomiali esatte.', 'Esempio: 1,4,9,16,25 ha grado 2', [f('sequence', 'Sequenza', '1, 4, 9, 16, 25', '', 'textarea')]),
    ],
  },
  'linear-algebra': {
    title: 'Algebra lineare esatta',
    tools: [
      t('det', 'Determinante', 'Calcola il determinante intero con eliminazione Bareiss.', 'Righe separate da newline o punto e virgola', [f('matrix', 'Matrice', '2, 4\n6, 8', '', 'textarea')]),
      t('rank', 'Rango esatto', 'Calcola il rango senza conversioni floating point.', 'Accetta matrici rettangolari', [f('matrix', 'Matrice', '1, 2, 3\n2, 4, 6', '', 'textarea')]),
      t('solve', 'Sistema lineare', 'Distingue soluzione unica, nessuna soluzione e infinite soluzioni.', 'Matrice A e vettore b separati', [f('matrix', 'Matrice A', '2, 1\n1, -1', '', 'textarea'), f('rhs', 'Vettore b', '5, 1')]),
      t('rref', 'Forma a scala ridotta', 'Calcola la RREF esatta su ℚ.', 'Le frazioni restano normalizzate', [f('matrix', 'Matrice', '1, 2, 1\n2, 4, 3', '', 'textarea')]),
      t('nullspace', 'Nucleo', 'Restituisce una base razionale esatta dello spazio nullo.', 'Verifica A·v=0 per ogni vettore', [f('matrix', 'Matrice', '1, 2, 3\n2, 4, 6', '', 'textarea')]),
      t('hnf', 'Forma normale di Hermite', 'Calcola una HNF per matrici intere con operazioni unimodulari di riga.', 'Dominio: matrici intere', [f('matrix', 'Matrice', '2, 4\n6, 8', '', 'textarea')]),
      t('snf', 'Invarianti di Smith', 'Calcola i fattori diagonali non nulli della forma normale di Smith.', 'Ogni invariante divide il successivo', [f('matrix', 'Matrice', '2, 4\n6, 8', '', 'textarea')]),
    ],
  },
  discovery: {
    title: 'Discovery',
    tools: [
      t('guess', 'Sequence Guess', 'Prova poche ipotesi economiche ed esatte su una sequenza finita.', 'Costante → aritmetica → geometrica → polinomiale → ricorrenza', [f('sequence', 'Sequenza', '1, 4, 9, 16, 25, 36', '', 'textarea')]),
      t('integer-relation', 'Integer Relation', 'Cerca con PSLQ piccoli coefficienti interi per una relazione numerica candidata.', 'Il risultato è una candidata, non una prova da input approssimati', [f('values', 'Valori', '1, 1.4142135623730951, 2.8284271247461903', '', 'textarea'), f('tolerance', 'Tolleranza', '1e-12'), f('coefficient_limit', 'Limite coefficienti', '100')]),
      t('recurrence', 'Recurrence Finder', 'Inferisce via Berlekamp–Massey una ricorrenza e la valida su tutti i termini interi.', 'Fibonacci → a(n)=a(n−1)+a(n−2)', [f('sequence', 'Sequenza', '0, 1, 1, 2, 3, 5, 8, 13', '', 'textarea')]),
    ],
  },
};

const batchToolIds = new Set([
  'isprime', 'nextprime', 'previousprime', 'factor', 'divisors', 'totient',
  'mobius', 'radical', 'squarefree', 'divisor-count', 'divisor-sum', 'integer-analysis',
]);

const cliCommands = {
  gcd: ['gcd', 'a', 'b'], xgcd: ['xgcd', 'a', 'b'], invmod: ['inverse', 'a', 'modulus'],
  isprime: ['prime', 'n'], nextprime: ['next-prime', 'n'], previousprime: ['prev-prime', 'n'],
  factor: ['factor', 'n'], divisors: ['divisors', 'n'], mobius: ['mobius', 'n'],
  radical: ['radical', 'n'], squarefree: ['squarefree', 'n'],
  'divisor-count': ['divisor-count', 'n'], 'divisor-sum': ['divisor-sum', 'n'],
  sqrtmod: ['sqrtmod', 'a', 'modulus'], 'integer-analysis': ['analyze', 'n'],
  'linear-congruence': ['congruence', 'a', 'b', 'modulus'],
};

const wasmLoading = document.querySelector('#wasm-loading');
const wasmInitStarted = performance.now();
const controls = [...document.querySelectorAll('button, input, textarea, select')];
controls.forEach((control) => { control.disabled = true; });
let wasmReady = false;
let currentCategory = 'arithmetic';
let currentTool = catalog.arithmetic.tools[0];
let currentResult = null;

try {
  await init();
  document.documentElement.dataset.wasmInitMs = (performance.now() - wasmInitStarted).toFixed(3);
  wasmReady = true;
  controls.forEach((control) => { control.disabled = false; });
  wasmLoading?.remove();
} catch (error) {
  wasmLoading.textContent = 'Il motore matematico locale non è disponibile. Ricarica la pagina.';
  wasmLoading.classList.add('error');
}

const categoryButtons = [...document.querySelectorAll('.category-button')];
const toolSelect = document.querySelector('#tool-select');
const toolFields = document.querySelector('#tool-fields');
const form = document.querySelector('#toolbox-form');
const resultArea = document.querySelector('#toolbox-result');
const primary = document.querySelector('#toolbox-primary');
const details = document.querySelector('#toolbox-details');
const toast = document.querySelector('#toast');
const resultActions = document.querySelector('#result-actions');
const resultContext = document.querySelector('#result-actions-context');
const batchResults = document.querySelector('#batch-results');
const batchResultsBody = document.querySelector('#batch-results-body');

function showToast(message, error = false) {
  toast.textContent = message;
  toast.classList.toggle('error', error);
  toast.classList.add('show');
  clearTimeout(showToast.timer);
  showToast.timer = setTimeout(() => toast.classList.remove('show'), 3600);
}

function formatElapsed(milliseconds) {
  return milliseconds < 1000 ? `${milliseconds.toFixed(1)} ms` : `${(milliseconds / 1000).toFixed(2)} s`;
}

function selectCategory(category) {
  currentCategory = category;
  categoryButtons.forEach((button) => button.classList.toggle('active', button.dataset.category === category));
  document.querySelector('#page-title').textContent = catalog[category].title;
  toolSelect.replaceChildren(...catalog[category].tools.map((item) => {
    const option = document.createElement('option');
    option.value = item.id;
    option.textContent = item.name;
    return option;
  }));
  currentTool = catalog[category].tools[0];
  renderTool();
}

function renderTool() {
  document.querySelector('#tool-name').textContent = currentTool.name;
  document.querySelector('#tool-description').textContent = currentTool.description;
  document.querySelector('#tool-example').textContent = currentTool.example;
  toolFields.replaceChildren(...currentTool.fields.map((field) => {
    const label = document.createElement('label');
    const multiline = field.type === 'textarea' || (field.name === 'n' && batchToolIds.has(currentTool.id));
    label.className = `field${multiline ? ' wide-field' : ''}`;
    const caption = document.createElement('span');
    caption.textContent = field.label;
    const input = document.createElement(multiline ? 'textarea' : 'input');
    input.name = field.name;
    input.value = field.value;
    input.required = true;
    if (multiline) input.rows = field.type === 'textarea' ? 4 : 3;
    const help = document.createElement('small');
    help.textContent = field.help || (multiline ? 'Un valore oppure più valori, uno per riga.' : '');
    label.append(caption, input, help);
    return label;
  }));
  resultArea.classList.add('hidden');
  resultActions.classList.add('hidden');
  batchResults.classList.add('hidden');
  batchResultsBody.replaceChildren();
  primary.classList.remove('hidden');
  details.classList.remove('hidden');
  currentResult = null;
}

function decode(call, payload) {
  if (!wasmReady) throw new Error('Il motore WASM non è ancora pronto.');
  const envelope = JSON.parse(call(JSON.stringify(payload)));
  if (!envelope.ok) throw new Error(envelope.error || 'Errore matematico non specificato.');
  return envelope.value;
}

function execute(tool, input) {
  if (tool.command === 'tool') return decode(wasm_run_tool, { tool: tool.id, input });
  if (tool.command === 'modular') return decode(wasm_calculate_modular, input);
  if (tool.command === 'residues') return decode(wasm_calculate_residues, { ...input, left: input.left.split(',').map((value) => value.trim()), right: input.right.split(',').map((value) => value.trim()) });
  if (tool.command === 'integer') return decode(wasm_analyze_integer, input);
  if (tool.command === 'linear') return decode(wasm_solve_linear, input);
  if (tool.command === 'sieve') return decode(wasm_run_sieve, { start: input.start, end: input.end, preview: input.preview, filters: [{ kind: 'excluded', modulus: input.modulus, residues: input.residues.split(',').map((value) => value.trim()), a: null, b: null }] });
  throw new Error('Comando non riconosciuto.');
}

function displayValue(value) {
  if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') return String(value);
  return JSON.stringify(value, null, 2);
}

function mainResult(result) {
  return result.result ?? result.message ?? result.primality ?? result.solution_kind ?? result.survivor_count ?? result.values ?? result;
}

function exactnessOf(result) {
  if (result.exactness) return result.exactness;
  if (result.probable === true) return 'probable';
  if (result.proof_incomplete === true) return 'proof_incomplete';
  return result.exact === false ? 'qualified' : 'exact';
}

function batchInputs(input) {
  if (!batchToolIds.has(currentTool.id) || typeof input.n !== 'string' || !input.n.includes('\n')) return null;
  const values = input.n.split(/\r?\n/).map((value) => value.trim()).filter(Boolean);
  return values.length > 1 ? values : null;
}

function renderBatch(rows) {
  batchResultsBody.replaceChildren(...rows.map((row) => {
    const tr = document.createElement('tr');
    const values = [row.input, row.result, row.status, row.exactness];
    values.forEach((value, index) => {
      const cell = document.createElement('td');
      cell.textContent = displayValue(value);
      if (index === 2) cell.className = row.status === 'ok' ? 'status-ok' : 'status-error';
      tr.append(cell);
    });
    return tr;
  }));
  primary.classList.add('hidden');
  details.classList.add('hidden');
  batchResults.classList.remove('hidden');
}

function renderScalar(result) {
  const main = mainResult(result);
  primary.textContent = displayValue(main);
  const supporting = Object.fromEntries(Object.entries(result).filter(([key]) => key !== 'result'));
  details.textContent = Object.keys(supporting).length ? JSON.stringify(supporting, null, 2) : '';
  primary.classList.remove('hidden');
  details.classList.remove('hidden');
  batchResults.classList.add('hidden');
}

form.addEventListener('submit', (event) => {
  event.preventDefault();
  const input = Object.fromEntries(new FormData(form));
  const started = performance.now();
  try {
    const values = batchInputs(input);
    let records;
    if (values) {
      records = values.map((value) => {
        try {
          const result = execute(currentTool, { ...input, n: value });
          return { input: value, result: mainResult(result), details: result, status: 'ok', exactness: exactnessOf(result) };
        } catch (error) {
          return { input: value, result: error.message || String(error), details: null, status: 'error', exactness: '—' };
        }
      });
      renderBatch(records);
    } else {
      const result = execute(currentTool, input);
      renderScalar(result);
      records = [{ input, result: mainResult(result), details: result, status: 'ok', exactness: exactnessOf(result) }];
    }
    const elapsed = performance.now() - started;
    document.querySelector('#toolbox-context').textContent = `${currentTool.name} · ${formatElapsed(elapsed)}`;
    resultArea.classList.remove('hidden');
    resultContext.textContent = `${currentTool.name} · ${formatElapsed(elapsed)}`;
    resultActions.classList.remove('hidden');
    currentResult = { title: currentTool.name, tool: currentTool.id, elapsed, input, records };
    updateActionAvailability();
    showToast(`Operazione completata · tempo: ${formatElapsed(elapsed)}`);
  } catch (error) {
    const elapsed = performance.now() - started;
    resultArea.classList.add('hidden');
    resultActions.classList.add('hidden');
    batchResults.classList.add('hidden');
    showToast(`${error.message || error} · tempo: ${formatElapsed(elapsed)}`, true);
  }
});

toolSelect.addEventListener('change', () => {
  currentTool = catalog[currentCategory].tools.find((item) => item.id === toolSelect.value);
  renderTool();
});
categoryButtons.forEach((button) => button.addEventListener('click', () => selectCategory(button.dataset.category)));

function exportPayload() {
  return {
    application: 'SwissMath Web', web_version: '0.2', core_version: '0.6',
    operation: currentResult.tool, elapsed_ms: currentResult.elapsed, records: currentResult.records,
  };
}

function plainResult() {
  return currentResult.records.map((record) => `${displayValue(record.input)}\t${displayValue(record.result)}\t${record.status}\t${record.exactness}`).join('\n');
}

function csvEscape(value) {
  const text = typeof value === 'string' ? value : JSON.stringify(value);
  return /[",\r\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}

function resultCsv() {
  return ['input,result,status,exactness', ...currentResult.records.map((record) => [record.input, record.result, record.status, record.exactness].map(csvEscape).join(','))].join('\n');
}

function download(name, content, type) {
  const blob = new Blob([content], { type });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = name;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}

function shellValue(value) {
  return /^[A-Za-z0-9_.+\-]+$/.test(value) ? value : JSON.stringify(value);
}

function cliCommand() {
  const mapping = cliCommands[currentTool.id];
  if (!mapping || currentResult.records.length !== 1) return null;
  const [, ...fields] = mapping;
  return ['swissmath', mapping[0], ...fields.map((field) => shellValue(currentResult.input[field]))].join(' ');
}

function shareHash() {
  if (!currentResult || currentResult.records.length !== 1) return null;
  const parameters = new URLSearchParams(currentResult.input);
  const hash = `#${encodeURIComponent(currentTool.id)}?${parameters}`;
  return hash.length <= 1500 ? hash : null;
}

function updateActionAvailability() {
  document.querySelector('#copy-command').disabled = !cliCommand();
  document.querySelector('#share-result').disabled = !shareHash();
}

document.querySelector('#copy-result').addEventListener('click', async () => {
  if (!currentResult) return;
  await navigator.clipboard.writeText(plainResult());
  showToast('Risultato copiato.');
});

document.querySelector('#copy-json').addEventListener('click', async () => {
  if (!currentResult) return;
  await navigator.clipboard.writeText(JSON.stringify(exportPayload(), null, 2));
  showToast('JSON copiato.');
});

document.querySelector('#download-json').addEventListener('click', () => {
  if (!currentResult) return;
  download(`swissmath-${currentTool.id}.json`, JSON.stringify(exportPayload(), null, 2), 'application/json');
  showToast('JSON pronto per il download.');
});

document.querySelector('#download-csv').addEventListener('click', () => {
  if (!currentResult) return;
  download(`swissmath-${currentTool.id}.csv`, resultCsv(), 'text/csv;charset=utf-8');
  showToast('CSV pronto per il download.');
});

document.querySelector('#copy-command').addEventListener('click', async () => {
  const command = cliCommand();
  if (!command) return;
  await navigator.clipboard.writeText(command);
  showToast('Comando CLI copiato.');
});

document.querySelector('#share-result').addEventListener('click', async () => {
  const hash = shareHash();
  if (!hash) {
    showToast('Il batch è troppo grande per un link: usa JSON o CSV.', true);
    return;
  }
  window.location.hash = hash;
  await navigator.clipboard.writeText(window.location.href);
  showToast('Link copiato. Il calcolo non verrà eseguito automaticamente.');
});

document.querySelector('#save-result').addEventListener('click', () => {
  if (!currentResult) return;
  const text = `SwissMath Web v0.2 · Core v0.6\n${currentResult.title}\nTempo: ${formatElapsed(currentResult.elapsed)}\n\n${plainResult()}\n`;
  download(`swissmath-${currentTool.id}.txt`, text, 'text/plain;charset=utf-8');
  showToast('Risultato salvato.');
});

document.querySelector('#print-result').addEventListener('click', () => {
  resultArea.classList.add('print-target');
  document.body.classList.add('printing');
  const cleanup = () => { resultArea.classList.remove('print-target'); document.body.classList.remove('printing'); };
  window.addEventListener('afterprint', cleanup, { once: true });
  window.print();
});

function restoreShareState() {
  const raw = window.location.hash.slice(1);
  if (!raw) return false;
  const [encodedTool, query = ''] = raw.split('?');
  let toolId;
  try {
    toolId = decodeURIComponent(encodedTool);
  } catch {
    return false;
  }
  const category = Object.keys(catalog).find((key) => catalog[key].tools.some((tool) => tool.id === toolId));
  if (!category) return false;
  selectCategory(category);
  currentTool = catalog[category].tools.find((tool) => tool.id === toolId);
  toolSelect.value = toolId;
  renderTool();
  const parameters = new URLSearchParams(query);
  currentTool.fields.forEach((field) => {
    const value = parameters.get(field.name);
    const control = form.elements.namedItem(field.name);
    if (value !== null && control) control.value = value;
  });
  showToast('Input ripristinato dal link. Premi Calcola per eseguire.');
  return true;
}

if (!restoreShareState()) selectCategory(currentCategory);
