import init, {
  wasm_analyze_integer,
  wasm_calculate_crt,
  wasm_calculate_modular,
  wasm_calculate_multiplicative_order,
  wasm_calculate_quadratic_symbols,
  wasm_calculate_residues,
  wasm_find_modular_roots,
  wasm_run_sieve,
  wasm_solve_linear,
  wasm_solve_system,
} from './pkg/swissmath_web.js';

const wasmLoading = document.querySelector('#wasm-loading');
const wasmControls = [...document.querySelectorAll('button, input, textarea, select')];
wasmControls.forEach((control) => { control.disabled = true; });
let wasmReady = false;
let wasmInitError = null;

try {
  await init();
  wasmReady = true;
  wasmControls.forEach((control) => { control.disabled = false; });
  wasmLoading?.remove();
} catch (error) {
  wasmInitError = error;
  if (wasmLoading) {
    wasmLoading.textContent = 'Il motore matematico locale non è disponibile. Ricarica la pagina.';
    wasmLoading.classList.add('error');
  }
}

const wasmCommands = {
  analyze_integer: wasm_analyze_integer,
  calculate_crt: wasm_calculate_crt,
  calculate_modular: wasm_calculate_modular,
  calculate_multiplicative_order: wasm_calculate_multiplicative_order,
  calculate_quadratic_symbols: wasm_calculate_quadratic_symbols,
  calculate_residues: wasm_calculate_residues,
  find_modular_roots: wasm_find_modular_roots,
  run_sieve: wasm_run_sieve,
  solve_linear: wasm_solve_linear,
  solve_system: wasm_solve_system,
};

function invoke(command, payload) {
  if (!wasmReady) {
    throw new Error(`Motore WASM non pronto: ${wasmInitError?.message || 'inizializzazione non completata'}.`);
  }
  const wasmCommand = wasmCommands[command];
  if (!wasmCommand) throw new Error(`Comando SwissMath non riconosciuto: ${command}.`);
  const envelope = JSON.parse(wasmCommand(JSON.stringify(payload)));
  if (!envelope.ok) throw new Error(envelope.error || 'Errore nel calcolo locale.');
  return envelope.value;
}

const screens = document.querySelectorAll('.screen');
const navItems = document.querySelectorAll('.nav-item');
const title = document.querySelector('#page-title');
const toast = document.querySelector('#toast');
const resultActions = document.querySelector('#result-actions');
const resultActionsContext = document.querySelector('#result-actions-context');
const saveResultButton = document.querySelector('#save-result');
const printResultButton = document.querySelector('#print-result');
let currentResult = null;

const titles = {
  modular: 'Calcolo modulare',
  crt: 'Combinazione CRT',
  residues: 'Insiemi di residui',
  integers: 'Numeri interi',
  quadratic: 'Residui quadratici',
  congruences: 'Congruenze lineari',
  sieve: 'Filtro modulare',
};

function showToast(message, isError = false) {
  toast.textContent = message;
  toast.classList.toggle('error', isError);
  toast.classList.add('show');
  window.clearTimeout(showToast.timer);
  showToast.timer = window.setTimeout(() => toast.classList.remove('show'), 3600);
}

function formatElapsed(milliseconds) {
  if (milliseconds < 1000) return `${milliseconds.toFixed(1)} ms`;
  return `${(milliseconds / 1000).toFixed(2)} s`;
}

async function invokeTimed(command, payload) {
  const started = performance.now();
  const result = await requireBackend()(command, payload);
  return { result, elapsedMs: performance.now() - started };
}

function showCompletion(message, elapsedMs, isError = false) {
  showToast(`${message} · tempo: ${formatElapsed(elapsedMs)}`, isError);
}

function refreshResultActions() {
  const activeScreen = document.querySelector('.screen.active');
  const visible = currentResult && activeScreen?.id === `screen-${currentResult.screenId}`;
  resultActions.classList.toggle('hidden', !visible);
  if (visible) resultActionsContext.textContent = `${currentResult.title} · ${formatElapsed(currentResult.elapsedMs)}`;
}

function registerResult(screenId, resultId, resultTitle, inputSummary, elapsedMs) {
  const resultElement = document.querySelector(`#${resultId}`);
  const visibleText = resultElement?.innerText?.trim() || resultElement?.textContent?.trim() || '';
  currentResult = {
    screenId,
    resultId,
    title: resultTitle,
    elapsedMs,
    filename: `swissmath-${screenId}-${new Date().toISOString().slice(0, 19).replace(/[T:]/g, '-')}.txt`,
    text: [
      'SwissMath Core',
      resultTitle,
      `Tempo operazione: ${formatElapsed(elapsedMs)}`,
      inputSummary,
      '',
      visibleText,
    ].filter(Boolean).join('\n'),
  };
  refreshResultActions();
}

async function saveCurrentResult() {
  if (!currentResult) return;
  const text = `${currentResult.text}\n`;
  if (typeof window.showSaveFilePicker === 'function') {
    try {
      const handle = await window.showSaveFilePicker({
        suggestedName: currentResult.filename,
        types: [{ description: 'Risultato SwissMath', accept: { 'text/plain': ['.txt'] } }],
      });
      const writable = await handle.createWritable();
      await writable.write(text);
      await writable.close();
      showToast('Risultato salvato.');
      return;
    } catch (error) {
      if (error?.name === 'AbortError') return;
    }
  }

  const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = currentResult.filename;
  document.body.append(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
  showToast('Risultato scaricato nella cartella Download.');
}

function printCurrentResult() {
  if (!currentResult) return;
  const target = document.querySelector(`#${currentResult.resultId}`);
  if (!target) return;
  const cleanup = () => {
    target.classList.remove('print-target');
    document.body.classList.remove('printing');
  };
  target.classList.add('print-target');
  document.body.classList.add('printing');
  window.addEventListener('afterprint', cleanup, { once: true });
  try {
    window.print();
  } catch (error) {
    cleanup();
    showError(error);
  }
}

function switchScreen(name) {
  navItems.forEach((item) => item.classList.toggle('active', item.dataset.screen === name));
  screens.forEach((screen) => screen.classList.toggle('active', screen.id === `screen-${name}`));
  title.textContent = titles[name];
  refreshResultActions();
}

navItems.forEach((item) => item.addEventListener('click', () => switchScreen(item.dataset.screen)));
saveResultButton.addEventListener('click', () => saveCurrentResult().catch(showError));
printResultButton.addEventListener('click', printCurrentResult);

function requireBackend() {
  return invoke;
}

function formValue(form, name) {
  return new FormData(form).get(name).toString().trim();
}

function isUnsignedInteger(value) {
  return /^\d+$/.test(value);
}

function validateUnsigned(value, label) {
  if (!isUnsignedInteger(value)) throw new Error(`${label}: inserisci un intero non negativo.`);
  try {
    return BigInt(value);
  } catch {
    throw new Error(`${label}: valore non valido.`);
  }
}

function parseList(value, label) {
  if (!value.trim()) return [];
  return value.split(',').map((item, index) => {
    const clean = item.trim();
    validateUnsigned(clean, `${label} [${index + 1}]`);
    return clean;
  });
}

function validateGuiModulus(value) {
  const parsed = validateUnsigned(value, 'Modulo');
  if (parsed < 1n) throw new Error('Il modulo deve essere maggiore di zero.');
  if (parsed > 2000000n) throw new Error('Per mantenere la GUI reattiva, il modulo massimo è 2.000.000.');
  return value;
}

function showError(error) {
  showToast(error?.message || String(error), true);
}

document.querySelectorAll('#crt-form input').forEach((input) => {
  input.addEventListener('input', () => {
    const preview = document.querySelector(`[data-preview="${input.name}"]`);
    if (preview) preview.textContent = input.value || '—';
  });
});

document.querySelector('#modular-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  try {
    const modulus = formValue(form, 'modulus');
    validateGuiModulus(modulus);
    const { result, elapsedMs } = await invokeTimed('calculate_modular', {
      modulus,
      a: formValue(form, 'a'),
      b: formValue(form, 'b'),
      exponent: formValue(form, 'exponent'),
    });
    document.querySelector('#result-sum').textContent = result.sum;
    document.querySelector('#result-difference').textContent = result.difference;
    document.querySelector('#result-product').textContent = result.product;
    document.querySelector('#result-power').textContent = result.power;
    document.querySelector('#result-inverse').textContent = result.inverse_a ?? 'non esiste';
    document.querySelector('#inverse-note').textContent = result.inverse_a ? 'inverso moltiplicativo di a' : 'gcd(a, m) ≠ 1';
    document.querySelector('#modular-context').textContent = `mod ${result.modulus} · a = ${result.a} · b = ${result.b}`;
    document.querySelector('#modular-result').classList.remove('hidden');
    registerResult('modular', 'modular-result', 'Calcolo modulare', `mod ${result.modulus} · a = ${result.a} · b = ${result.b}`, elapsedMs);
    showCompletion('Calcolo completato', elapsedMs);
  } catch (error) { showError(error); }
});

document.querySelector('#crt-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  try {
    const { result, elapsedMs } = await invokeTimed('calculate_crt', {
      residueA: formValue(form, 'residueA'),
      modulusA: formValue(form, 'modulusA'),
      residueB: formValue(form, 'residueB'),
      modulusB: formValue(form, 'modulusB'),
    });
    const status = document.querySelector('#crt-status');
    status.textContent = result.compatible ? 'COMPATIBILI' : 'INCOMPATIBILI';
    status.style.color = result.compatible ? 'var(--accent)' : 'var(--danger)';
    document.querySelector('#crt-message').textContent = result.message;
    document.querySelector('#crt-equation').textContent = result.residue === null
      ? 'Nessuna classe combinata rappresentabile'
      : `x ≡ ${result.residue} (mod ${result.modulus})`;
    document.querySelector('#crt-result').classList.remove('hidden');
    registerResult('crt', 'crt-result', 'Combinazione CRT', result.message, elapsedMs);
    showCompletion(result.compatible ? 'CRT calcolato' : 'Le congruenze non sono compatibili', elapsedMs, !result.compatible);
  } catch (error) { showError(error); }
});

document.querySelector('#residues-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  try {
    const modulus = formValue(form, 'modulus');
    validateGuiModulus(modulus);
    const operation = formValue(form, 'operation');
    const { result, elapsedMs } = await invokeTimed('calculate_residues', {
      modulus,
      left: parseList(formValue(form, 'left'), 'A'),
      right: parseList(formValue(form, 'right'), 'B'),
      operation,
    });
    document.querySelector('#residues-context').textContent = `mod ${result.modulus}`;
    document.querySelector('#residue-len').textContent = result.len;
    document.querySelector('#residue-message').textContent = result.message;
    const queryWrap = document.querySelector('#residue-query-wrap');
    const values = document.querySelector('#residue-values');
    if (result.query) {
      queryWrap.classList.remove('hidden');
      document.querySelector('#residue-query').textContent = result.len;
      values.textContent = '';
    } else {
      queryWrap.classList.add('hidden');
      const visible = result.values.length > 500 ? `${result.values.slice(0, 500).join(', ')} … (${result.values.length - 500} altri)` : result.values.join(', ');
      values.textContent = visible;
    }
    document.querySelector('#residues-result').classList.remove('hidden');
    registerResult('residues', 'residues-result', 'Operazione sugli insiemi di residui', `mod ${result.modulus} · ${result.message}`, elapsedMs);
    showCompletion('Operazione completata', elapsedMs);
  } catch (error) { showError(error); }
});

function renderFactorization(factors, unavailable = false) {
  const target = document.querySelector('#integer-factorization');
  target.replaceChildren();
  if (unavailable) {
    target.textContent = 'Non disponibile';
    return;
  }
  if (!factors.length) {
    target.textContent = '1';
    return;
  }
  factors.forEach((factor, index) => {
    if (index > 0) target.append(' × ');
    target.append(factor.prime);
    if (factor.exponent !== '1') {
      const exponent = document.createElement('sup');
      exponent.textContent = factor.exponent;
      target.append(exponent);
    }
  });
}

function renderIntegerAnalysis(result) {
  const typeLabels = {
    unità: 'Unità',
    primo: 'Primo',
    composto: 'Composto',
    composito: 'Composito',
    probabile_primo: 'Probabile primo',
    primo_esatto: 'Primo — verifica esatta',
    prova_incompleta: 'Prova esatta non completata',
    né_primo_né_composto: 'Né primo né composto',
  };
  document.querySelector('#integer-type').textContent = typeLabels[result.classification] ?? result.classification;
  document.querySelector('#integer-primality').textContent = typeLabels[result.primality] ?? result.primality;
  document.querySelector('#integer-phi').textContent = result.phi ?? 'Non disponibile';
  document.querySelector('#integer-lambda').textContent = result.lambda ?? 'Non disponibile';
  document.querySelector('#integer-analysis-context').textContent = `n = ${result.n}`;
  document.querySelector('#integer-current-n').textContent = result.n;
  const arithmeticUnavailable = !result.exact || result.classification === 'né_primo_né_composto';
  renderFactorization(result.factors, arithmeticUnavailable);
  document.querySelector('#integer-factorization-note').textContent = result.exact
    ? 'Fattorizzazione prima esatta nel dominio u64.'
    : 'Non disponibile per numeri oltre 64 bit: questa release valuta soltanto la primalità.';
  document.querySelector('#integer-analysis-note').textContent = result.note ?? 'Primalità esatta nel dominio u64.';
  document.querySelector('#integer-order-form').classList.toggle('hidden', !result.exact || result.classification === 'né_primo_né_composto');
  document.querySelector('#integer-analysis-result').classList.remove('hidden');
  document.querySelector('#integer-order-result').classList.add('hidden');
}

document.querySelector('#integer-analysis-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  try {
    const n = formValue(form, 'n');
    validateDecimal(n, 'Numero n');
    const { result, elapsedMs } = await invokeTimed('analyze_integer', { n });
    renderIntegerAnalysis(result);
    registerResult('integers', 'integer-analysis-result', 'Analisi del numero', `n = ${result.n}`, elapsedMs);
    showCompletion('Analisi completata', elapsedMs);
  } catch (error) { showError(error); }
});

function validateDecimal(value, label) {
  if (!/^\d+$/.test(value)) throw new Error(`${label}: inserisci un intero decimale non negativo.`);
  try { return BigInt(value); } catch { throw new Error(`${label}: valore non valido.`); }
}

function validateSignedI128(value, label) {
  if (!/^-?\d+$/.test(value)) throw new Error(`${label}: inserisci un intero decimale.`);
  let parsed;
  try { parsed = BigInt(value); } catch { throw new Error(`${label}: valore non valido.`); }
  const min = -(1n << 127n);
  const max = (1n << 127n) - 1n;
  if (parsed < min || parsed > max) throw new Error(`${label}: valore oltre il limite signed i128.`);
  return parsed;
}

function renderQuadraticSymbols(result) {
  document.querySelector('#quadratic-symbol-context').textContent = `a = ${result.a} · n = ${result.modulus}`;
  document.querySelector('#quadratic-jacobi').textContent = result.jacobi;
  document.querySelector('#quadratic-legendre').textContent = result.legendre ?? 'non applicabile';
  document.querySelector('#quadratic-symbol-message').textContent = result.message;
  document.querySelector('#quadratic-symbol-result').classList.remove('hidden');
}

function renderQuadraticRoots(result) {
  document.querySelector('#quadratic-roots-context').textContent = `x² ≡ ${result.a} (mod ${result.modulus})`;
  document.querySelector('#quadratic-root-exists').textContent = result.exists ? 'Sì' : 'No';
  document.querySelector('#quadratic-root-count').textContent = result.root_count;
  document.querySelector('#quadratic-root-message').textContent = result.message;
  const roots = result.roots.length ? result.roots.join(', ') : 'Nessuna radice';
  document.querySelector('#quadratic-roots').textContent = roots;
  document.querySelector('#quadratic-roots-note').textContent = result.roots.length >= 100
    ? 'Anteprima limitata ai primi 100 valori.'
    : '';
  document.querySelector('#quadratic-roots-result').classList.remove('hidden');
}

document.querySelector('#quadratic-symbol-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  try {
    const a = formValue(form, 'a');
    const modulus = formValue(form, 'modulus');
    validateSignedI128(a, 'a');
    validatePositive(modulus, 'Modulo n');
    const { result, elapsedMs } = await invokeTimed('calculate_quadratic_symbols', { a, modulus });
    renderQuadraticSymbols(result);
    registerResult('quadratic', 'quadratic-symbol-result', 'Simboli quadratici', `a = ${result.a} · n = ${result.modulus}`, elapsedMs);
    showCompletion('Simboli calcolati', elapsedMs);
  } catch (error) { showError(error); }
});

document.querySelector('#quadratic-roots-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  try {
    const a = formValue(form, 'a');
    const modulus = formValue(form, 'modulus');
    validateSignedI128(a, 'a');
    validatePositive(modulus, 'Modulo n');
    const { result, elapsedMs } = await invokeTimed('find_modular_roots', { a, modulus });
    renderQuadraticRoots(result);
    registerResult('quadratic', 'quadratic-roots-result', 'Radici modulari', `x² ≡ ${result.a} (mod ${result.modulus})`, elapsedMs);
    showCompletion(result.exists ? 'Radici trovate' : 'Nessuna radice', elapsedMs, !result.exists);
  } catch (error) { showError(error); }
});

document.querySelector('#integer-order-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  try {
    const a = formValue(form, 'a');
    const modulus = document.querySelector('#integer-current-n').textContent.trim();
    validateUint64(a, 'a');
    validatePositive(modulus, 'Modulo n');
    const { result, elapsedMs } = await invokeTimed('calculate_multiplicative_order', { a, modulus });
    document.querySelector('#integer-order-message').textContent = result.message;
    document.querySelector('#integer-order-equation').textContent = result.exists
      ? `ord_${result.modulus}(${result.a}) = ${result.order}`
      : 'Ordine moltiplicativo non esistente';
    document.querySelector('#integer-order-result').classList.remove('hidden');
    registerResult('integers', 'integer-order-result', 'Ordine moltiplicativo', `a = ${result.a} · n = ${result.modulus}`, elapsedMs);
    showCompletion(result.exists ? 'Ordine calcolato' : 'Ordine non disponibile', elapsedMs, !result.exists);
  } catch (error) { showError(error); }
});

function validateUint64(value, label) {
  const parsed = validateUnsigned(value, label);
  const max = 18446744073709551615n;
  if (parsed > max) throw new Error(`${label}: valore oltre il limite u64.`);
  return parsed;
}

function validatePositive(value, label) {
  const parsed = validateUint64(value, label);
  if (parsed < 1n) throw new Error(`${label}: deve essere maggiore di zero.`);
  return parsed;
}

function renderLinearResult(result) {
  const equation = document.querySelector('#linear-equation');
  const facts = document.querySelector('#linear-facts');
  const residues = document.querySelector('#linear-residues');
  const count = document.querySelector('#linear-count');
  document.querySelector('#linear-message').textContent = result.message;
  count.textContent = `${result.solution_count} soluzioni modulo ${result.original_modulus}`;
  if (result.solution_kind === 'none') {
    equation.textContent = 'Nessuna soluzione';
  } else if (result.solution_kind === 'all') {
    equation.textContent = 'Ogni intero è soluzione';
  } else {
    equation.textContent = `x ≡ ${result.residue} (mod ${result.solution_modulus})`;
  }

  const explanation = [
    `a e b normalizzati: ${result.normalized_a} · x ≡ ${result.normalized_b} (mod ${result.original_modulus})`,
    `gcd(${result.normalized_a}, ${result.original_modulus}) = ${result.gcd}`,
  ];
  if (result.solution_kind === 'none') {
    explanation.push(`${result.gcd} non divide ${result.normalized_b}: il sistema è impossibile.`);
  } else if (result.solution_kind === 'class') {
    explanation.push(`Dividi per ${result.gcd}: ${result.reduced_a} · x ≡ ${result.reduced_b} (mod ${result.reduced_modulus})`);
    explanation.push(`Inverso: ${result.reduced_a}⁻¹ ≡ ${result.inverse} (mod ${result.reduced_modulus})`);
    explanation.push(`Classe finale: x ≡ ${result.residue} (mod ${result.solution_modulus})`);
  } else {
    explanation.push('La congruenza è tautologica nel modulo indicato.');
  }
  facts.replaceChildren(...explanation.map((line) => {
    const item = document.createElement('div');
    item.textContent = line;
    return item;
  }));

  residues.textContent = result.residues.length
    ? `Residui modulo ${result.original_modulus}: ${result.residues.join(', ')}`
    : (BigInt(result.solution_count) > 1000n ? 'Elenco dei residui omesso: è troppo grande per la GUI.' : 'Nessun residuo da visualizzare.');
  document.querySelector('#linear-result').classList.remove('hidden');
}

document.querySelector('#linear-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  try {
    const a = formValue(form, 'a');
    const b = formValue(form, 'b');
    const modulus = formValue(form, 'modulus');
    validateUint64(a, 'a');
    validateUint64(b, 'b');
    validatePositive(modulus, 'Modulo');
    const { result, elapsedMs } = await invokeTimed('solve_linear', { a, b, modulus });
    renderLinearResult(result);
    registerResult('congruences', 'linear-result', 'Congruenza lineare', `${a} · x ≡ ${b} (mod ${modulus})`, elapsedMs);
    showCompletion('Congruenza risolta', elapsedMs);
  } catch (error) { showError(error); }
});

const systemRows = document.querySelector('#system-rows');

function addSystemRow(values = { a: '14', b: '8', modulus: '30' }) {
  const row = document.createElement('div');
  row.className = 'system-row';
  row.innerHTML = `
    <label class="field"><span>a</span><input data-field="a" type="text" inputmode="numeric" value="${values.a}" /></label>
    <label class="field"><span>b</span><input data-field="b" type="text" inputmode="numeric" value="${values.b}" /></label>
    <label class="field"><span>modulo</span><input data-field="modulus" type="text" inputmode="numeric" value="${values.modulus}" /></label>
    <button class="remove-button" data-remove-row type="button" aria-label="Rimuovi riga">×</button>`;
  systemRows.append(row);
}

function collectSystemRows() {
  return [...systemRows.querySelectorAll('.system-row')].map((row, index) => {
    const a = row.querySelector('[data-field="a"]').value.trim();
    const b = row.querySelector('[data-field="b"]').value.trim();
    const modulus = row.querySelector('[data-field="modulus"]').value.trim();
    validateUint64(a, `a nella riga ${index + 1}`);
    validateUint64(b, `b nella riga ${index + 1}`);
    validatePositive(modulus, `Modulo nella riga ${index + 1}`);
    return { a, b, modulus };
  });
}

function renderSystemResult(result) {
  const equation = document.querySelector('#system-equation');
  if (result.solution_kind === 'none') equation.textContent = 'Nessuna soluzione comune';
  else if (result.solution_kind === 'all') equation.textContent = 'Ogni intero è soluzione';
  else equation.textContent = `x ≡ ${result.residue} (mod ${result.modulus})`;
  document.querySelector('#system-message').textContent = result.message;
  const summary = document.querySelector('#system-row-summary');
  summary.replaceChildren(...result.rows.map((row, index) => {
    const item = document.createElement('div');
    const status = row.solution_kind === 'none' ? 'nessuna soluzione' : row.solution_kind === 'all' ? 'tautologia' : `x ≡ ${row.residue} (mod ${row.solution_modulus})`;
    item.textContent = `Riga ${index + 1}: ${status}`;
    return item;
  }));
  document.querySelector('#system-result').classList.remove('hidden');
}

systemRows.addEventListener('click', (event) => {
  if (event.target.matches('[data-remove-row]')) {
    const rows = systemRows.querySelectorAll('.system-row');
    if (rows.length > 1) event.target.closest('.system-row').remove();
  }
});
document.querySelector('#system-add').addEventListener('click', () => addSystemRow({ a: '1', b: '0', modulus: '2' }));
document.querySelector('#system-submit').addEventListener('click', async () => {
  try {
    const rows = collectSystemRows();
    const { result, elapsedMs } = await invokeTimed('solve_system', { rows });
    renderSystemResult(result);
    registerResult('congruences', 'system-result', 'Sistema di congruenze', `${rows.length} righe`, elapsedMs);
    showCompletion('Sistema risolto', elapsedMs);
  } catch (error) { showError(error); }
});
addSystemRow();

const sieveRows = document.querySelector('#sieve-rows');

function updateSieveRow(row) {
  const isLinear = row.querySelector('[data-field="kind"]').value === 'linear';
  row.querySelector('.sieve-residues-wrap').classList.toggle('hidden', isLinear);
  row.querySelector('.sieve-linear-fields').classList.toggle('hidden', !isLinear);
}

function addSieveRow(values = { kind: 'allowed', modulus: '5', residues: '1,4', a: '', b: '' }) {
  const row = document.createElement('div');
  row.className = 'sieve-row';
  row.innerHTML = `
    <div class="sieve-row-head"><label class="field"><span>Tipo</span><select data-field="kind"><option value="allowed">Residui ammessi</option><option value="excluded">Residui esclusi</option><option value="linear">Congruenza lineare</option></select></label><label class="field"><span>Modulo</span><input data-field="modulus" type="text" inputmode="numeric" value="${values.modulus}" /></label><button class="remove-button" data-remove-sieve type="button" aria-label="Rimuovi filtro">×</button></div>
    <label class="field sieve-residues-wrap"><span>Residui, separati da virgola</span><input data-field="residues" type="text" value="${values.residues}" placeholder="es. 0, 1, 6" /></label>
    <div class="sieve-linear-fields field-grid two hidden"><label class="field"><span>Coefficiente a</span><input data-field="a" type="text" inputmode="numeric" value="${values.a}" /></label><label class="field"><span>Termine b</span><input data-field="b" type="text" inputmode="numeric" value="${values.b}" /></label></div>`;
  sieveRows.append(row);
  const select = row.querySelector('[data-field="kind"]');
  select.value = values.kind;
  select.addEventListener('change', () => updateSieveRow(row));
  updateSieveRow(row);
}

sieveRows.addEventListener('click', (event) => {
  if (event.target.matches('[data-remove-sieve]')) {
    const rows = sieveRows.querySelectorAll('.sieve-row');
    if (rows.length > 1) event.target.closest('.sieve-row').remove();
  }
});
document.querySelector('#sieve-add').addEventListener('click', () => addSieveRow({ kind: 'allowed', modulus: '7', residues: '0,1,6', a: '', b: '' }));

function collectSieveFilters() {
  return [...sieveRows.querySelectorAll('.sieve-row')].map((row, index) => {
    const kind = row.querySelector('[data-field="kind"]').value;
    const modulus = row.querySelector('[data-field="modulus"]').value.trim();
    validatePositive(modulus, `Modulo del filtro ${index + 1}`);
    if (kind === 'linear') {
      const a = row.querySelector('[data-field="a"]').value.trim();
      const b = row.querySelector('[data-field="b"]').value.trim();
      validateUint64(a, `a del filtro ${index + 1}`);
      validateUint64(b, `b del filtro ${index + 1}`);
      return { kind, modulus, residues: [], a, b };
    }
    validateGuiModulus(modulus);
    return { kind, modulus, residues: parseList(row.querySelector('[data-field="residues"]').value, `Filtro ${index + 1}`), a: null, b: null };
  });
}

document.querySelector('#sieve-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  try {
    const start = formValue(form, 'start');
    const end = formValue(form, 'end');
    const preview = formValue(form, 'preview');
    const startNumber = validateUint64(start, 'Da');
    const endNumber = validateUint64(end, 'A');
    validateUint64(preview, 'Anteprima');
    if (startNumber > endNumber) throw new Error('L’intervallo deve soddisfare Da ≤ A.');
    if (BigInt(preview) > 1000n) throw new Error('L’anteprima può contenere al massimo 1.000 valori.');
    const filters = collectSieveFilters();
    const { result, elapsedMs } = await invokeTimed('run_sieve', { start, end, preview, filters });
    document.querySelector('#sieve-message').textContent = result.message;
    document.querySelector('#sieve-count').textContent = result.survivor_count;
    document.querySelector('#sieve-percentage').textContent = result.survivor_percentage;
    document.querySelector('#sieve-filter-count').textContent = result.normalized_filter_count;
    document.querySelector('#sieve-range').textContent = `${result.start} … ${result.end}`;
    document.querySelector('#sieve-anchor').textContent = result.anchor_modulus
      ? `Ancora: mod ${result.anchor_modulus} · ${result.anchor_allowed_count} residui ammessi`
      : 'Nessuna ancora: intervallo non vincolato o filtro vuoto.';
    document.querySelector('#sieve-preview').textContent = result.preview.length ? result.preview.join(', ') : 'Nessun valore da visualizzare.';
    document.querySelector('#sieve-result').classList.remove('hidden');
    registerResult('sieve', 'sieve-result', 'Filtro modulare', `Intervallo ${result.start} … ${result.end}`, elapsedMs);
    showCompletion('Ricerca completata', elapsedMs);
  } catch (error) { showError(error); }
});
addSieveRow();
