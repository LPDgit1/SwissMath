import init, {
  wasm_analyze_integer,
  wasm_calculate_modular,
  wasm_calculate_residues,
  wasm_run_sieve,
  wasm_run_tool,
  wasm_solve_linear,
} from './pkg/swissmath_web.js';

const f = (name, label, value, help = '', type = 'text') => ({ name, label, value, help, type });
const s = (name, label, value, options, help = '') => ({ name, label, value, options, help, type: 'select' });
const t = (id, name, description, example, fields, command = 'tool') => ({ id, name, description, example, fields, command });

const catalog = {
  arithmetic: {
    title: 'Arithmetic',
    tools: [
      t('gcd', 'Greatest common divisor', 'Computes the GCD of two non-negative integers.', 'Example: gcd(40902, 24140) = 34', [f('a', 'Integer a', '40902'), f('b', 'Integer b', '24140')]),
      t('lcm', 'Least common multiple', 'Computes the LCM with overflow-safe ordering.', 'Example: lcm(21, 6) = 42', [f('a', 'Integer a', '21'), f('b', 'Integer b', '6')]),
      t('xgcd', 'Extended Euclidean algorithm', 'Finds g, x, and y such that ax + by = g.', 'Example: 240x + 46y = 2', [f('a', 'Non-negative integer a', '240'), f('b', 'Non-negative integer b', '46')]),
      t('powmod', 'Modular exponentiation', 'Computes aⁿ mod m without constructing the full power.', 'Example: 7¹²⁸ mod 13', [f('a', 'Base a', '7'), f('exponent', 'Exponent n', '128'), f('modulus', 'Modulus m', '13')]),
      t('invmod', 'Modular inverse', 'Finds a⁻¹ mod m when gcd(a,m)=1.', 'Example: 7⁻¹ mod 26 = 15', [f('a', 'Value a', '7'), f('modulus', 'Modulus m', '26')]),
      t('crt', 'Chinese remainder theorem', 'Combines a congruence system, including non-coprime moduli.', 'One congruence per line: residue, modulus', [f('congruences', 'Congruences', '2, 3\n3, 5\n2, 7', 'Rows: residue, modulus', 'textarea')]),
      t('iroot', 'Integer root', 'Computes the floor of an n-th root and reports exactness.', 'Example: integer cube root of 80 = 4', [f('n', 'Integer n', '80'), f('degree', 'Root degree', '3')]),
      t('perfect-power', 'Perfect power', 'Recognizes the canonical representation aᵏ with a>1 and k>1.', 'Example: 64 = 2⁶', [f('n', 'Integer n', '64')]),
      t('base-convert', 'Base conversion', 'Converts integers between bases 2 and 36.', 'Example: ff in base 16 → 11111111 in base 2', [f('value', 'Value', 'ff'), f('from_base', 'Source base', '16'), f('to_base', 'Target base', '2')]),
      t('modular', 'Modular calculator', 'Computes sum, difference, product, power, and inverse together.', 'Operations in ℤ/mℤ', [f('modulus', 'Modulus', '7'), f('a', 'Value a', '3'), f('b', 'Value b', '5'), f('exponent', 'Exponent', '4')], 'modular'),
      t('residue-set', 'Residue sets', 'Combines two materialized sets in the same modulus.', 'Comma-separated lists', [f('modulus', 'Modulus', '12'), f('left', 'Set A', '0,2,4,6'), f('right', 'Set B', '2,3,6,9'), f('operation', 'Operation', 'intersection', 'intersection, union, or difference')], 'residues'),
    ],
  },
  'number-theory': {
    title: 'Number theory',
    tools: [
      t('isprime', 'u64 primality test', 'Returns a deterministic result over the full u64 domain.', 'Prime, Composite, or Neither', [f('n', 'Integer n', '1000000007')]),
      t('nextprime', 'Next prime', 'Finds the smallest prime strictly greater than n.', 'Example: after 1000 comes 1009', [f('n', 'Integer n', '1000')]),
      t('previousprime', 'Previous prime', 'Finds the largest prime strictly smaller than n.', 'No result exists for n≤2', [f('n', 'Integer n', '1000')]),
      t('factor', 'Integer factorization', 'Factors a u64 using trial division and Pollard–Brent.', 'Example: 360 = 2³·3²·5', [f('n', 'Integer n', '360')]),
      t('divisors', 'Divisors', 'Computes the count, sum, and optionally the divisor list.', 'Very large lists are not materialized', [f('n', 'Integer n', '360')]),
      t('totient', 'Euler totient', 'Computes φ(n) while reusing one factorization.', 'Example: φ(360) = 96', [f('n', 'Integer n', '360')]),
      t('mobius', 'Möbius function', 'Computes μ(n) from the prime factorization.', 'Possible values: −1, 0, 1', [f('n', 'Integer n', '30')]),
      t('radical', 'Radical', 'Computes the product of the distinct prime divisors.', 'Example: rad(360) = 30', [f('n', 'Integer n', '360')]),
      t('squarefree', 'Squarefree test', 'Checks whether no prime square divides n.', 'Exact Boolean result', [f('n', 'Integer n', '30')]),
      t('divisor-count', 'Divisor count', 'Computes τ(n) from the factorization.', 'Example: τ(360) = 24', [f('n', 'Integer n', '360')]),
      t('divisor-sum', 'Divisor sum', 'Computes σ(n) exactly in u128.', 'Example: σ(360) = 1170', [f('n', 'Integer n', '360')]),
      t('valuation', 'p-adic valuation', 'Computes vₚ(n), requiring p to be prime.', 'For n=0 returns ∞', [f('n', 'Integer n', '81'), f('p', 'Prime p', '3')]),
      t('jacobi', 'Jacobi symbol', 'Computes the symbol without factoring the modulus.', 'The modulus must be positive and odd', [f('a', 'Value a', '5'), f('modulus', 'Modulus n', '11')]),
      t('sqrtmod', 'Modular square roots', 'Finds square roots in the exact Core-supported domains.', 'Example: x² ≡ 10 mod 13 → 6, 7', [f('a', 'Value a', '10'), f('modulus', 'Modulus n', '13')]),
      t('multiplicative-order', 'Multiplicative order', 'Finds the smallest k>0 with aᵏ≡1 mod n.', 'Requires gcd(a,n)=1', [f('a', 'Value a', '2'), f('modulus', 'Modulus n', '9')]),
      t('primitive-root', 'Find primitive root', 'Finds the smallest positive generator of the multiplicative group of a prime field.', 'Example: the smallest primitive root modulo 17 is 3', [f('prime', 'Prime p', '17')]),
      t('is-primitive-root', 'Check primitive root', 'Checks whether g generates every nonzero residue modulo the prime p.', 'Example: 3 is a primitive root modulo 17', [f('prime', 'Prime p', '17'), f('g', 'Candidate g', '3')]),
      t('discrete-log', 'Discrete logarithm', 'Finds x such that gˣ ≡ h (mod p) inside the subgroup generated by g.', 'Returns an exact solution, no solution, or a bounded-search limit.', [f('prime', 'Prime p', '97'), f('g', 'Base g', '5'), f('h', 'Target h', '83')]),
      t('integer-analysis', 'Complete integer analysis', 'Classifies primality and, in u64, exposes factors, φ, and λ.', 'Labels distinguish exact proof from probable prime', [f('n', 'Decimal integer', '360')], 'integer'),
      t('linear-congruence', 'Linear congruence', 'Solves ax≡b mod m and explains the solution class.', 'Example: 6x ≡ 8 mod 14', [f('a', 'Coefficient a', '6'), f('b', 'Term b', '8'), f('modulus', 'Modulus m', '14')], 'linear'),
      t('modular-sieve', 'Modular sieve', 'Filters an interval by excluding periodic residues.', 'Example: exclude 0 mod 2 between 1 and 100', [f('start', 'Start', '1'), f('end', 'End', '100'), f('modulus', 'Filter modulus', '2'), f('residues', 'Excluded residues', '0'), f('preview', 'Maximum preview', '25')], 'sieve'),
    ],
  },
  fractions: {
    title: 'Fractions and reconstruction',
    tools: [
      t('contfrac', 'Continued fraction', 'Converts a fraction or decimal exactly into its quotients.', 'Example: 355/113 → [3; 7, 16]', [f('value', 'Value', '355/113')]),
      t('rationalize', 'Rationalize', 'Finds the closest fraction under a maximum denominator.', 'π with denominator ≤10000 → 355/113', [f('value', 'Decimal value', '3.141592653589793'), f('max_denominator', 'Maximum denominator', '10000')]),
      t('rational-reconstruct', 'Rational reconstruction', 'Reconstructs a/b from a residue while checking bounds and modular identity.', 'Returns an error when the constraints do not determine a valid solution', [f('residue', 'Residue r', '7'), f('modulus', 'Modulus m', '101'), f('bound', 'Bound |a|, |b|', '10')]),
    ],
  },
  polynomials: {
    title: 'Polynomials',
    tools: [
      t('poly-eval', 'Polynomial evaluation', 'Evaluates exact coefficients using Horner’s method.', 'Coefficients in ascending order: c₀,c₁,…', [f('coefficients', 'Coefficients', '1, 2, 3'), f('x', 'Value x', '4')]),
      t('poly-gcd', 'Polynomial GCD', 'Computes the monic GCD of two polynomials over ℚ.', 'Coefficients in ascending order', [f('left', 'Polynomial A', '-2, 1, 1'), f('right', 'Polynomial B', '-3, 2, 1')]),
      t('interpolate', 'Exact interpolation', 'Reconstructs the polynomial through points with distinct abscissas.', 'One point per line: x, y', [f('points', 'Points', '0, 1\n1, 6\n2, 17', '', 'textarea')]),
      t('finite-differences', 'Finite differences', 'Builds the table and detects exact polynomial progressions.', 'Example: 1,4,9,16,25 has degree 2', [f('sequence', 'Sequence', '1, 4, 9, 16, 25', '', 'textarea')]),
    ],
  },
  'linear-algebra': {
    title: 'Exact linear algebra',
    tools: [
      t('det', 'Determinant', 'Computes the integer determinant using Bareiss elimination.', 'Rows separated by newlines or semicolons', [f('matrix', 'Matrix', '2, 4\n6, 8', '', 'textarea')]),
      t('rank', 'Exact rank', 'Computes rank without floating-point conversion.', 'Accepts rectangular matrices', [f('matrix', 'Matrix', '1, 2, 3\n2, 4, 6', '', 'textarea')]),
      t('solve', 'Linear system', 'Distinguishes unique, no, and infinitely many solutions.', 'Matrix A and vector b are entered separately', [f('matrix', 'Matrix A', '2, 1\n1, -1', '', 'textarea'), f('rhs', 'Vector b', '5, 1')]),
      t('rref', 'Reduced row-echelon form', 'Computes the exact RREF over ℚ.', 'Fractions remain normalized', [f('matrix', 'Matrix', '1, 2, 1\n2, 4, 3', '', 'textarea')]),
      t('nullspace', 'Nullspace', 'Returns an exact rational basis of the nullspace.', 'Checks A·v=0 for every vector', [f('matrix', 'Matrix', '1, 2, 3\n2, 4, 6', '', 'textarea')]),
      t('hnf', 'Hermite normal form', 'Computes an HNF for integer matrices using unimodular row operations.', 'Domain: integer matrices', [f('matrix', 'Matrix', '2, 4\n6, 8', '', 'textarea')]),
      t('snf', 'Smith invariants', 'Computes the non-zero diagonal factors of the Smith normal form.', 'Each invariant divides the next', [f('matrix', 'Matrix', '2, 4\n6, 8', '', 'textarea')]),
    ],
  },
  'finite-fields': {
    title: 'Finite fields',
    tools: [
      t('fp-matrix-add', 'Matrix addition over Fp', 'Adds two matrices entry by entry over the selected prime field.', 'Paste one row per line; entries are reduced modulo p.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix A', '1, 2\n3, 4', '', 'textarea'), f('other', 'Matrix B', '4, 3\n2, 1', '', 'textarea')]),
      t('fp-matrix-sub', 'Matrix subtraction over Fp', 'Subtracts two matrices entry by entry over the selected prime field.', 'Both matrices must have the same dimensions.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix A', '1, 2\n3, 4', '', 'textarea'), f('other', 'Matrix B', '4, 3\n2, 1', '', 'textarea')]),
      t('fp-matrix-mul', 'Matrix multiplication over Fp', 'Multiplies compatible dense matrices over Fp.', 'A columns must equal B rows.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix A', '1, 2\n3, 4', '', 'textarea'), f('other', 'Matrix B', '4, 3\n2, 1', '', 'textarea')]),
      t('fp-matrix-vector', 'Matrix-vector product over Fp', 'Multiplies a dense matrix by a column vector.', 'Vector length must equal the number of matrix columns.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix A', '1, 2\n3, 4', '', 'textarea'), f('vector', 'Vector', '1, 3')]),
      t('fp-matrix-det', 'Determinant over Fp', 'Computes the exact determinant of a square matrix over Fp.', 'For [[1,2],[3,4]] over F5 the determinant is 3.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix', '1, 2\n3, 4', '', 'textarea')]),
      t('fp-matrix-rank', 'Rank over Fp', 'Computes matrix rank using finite-field elimination.', 'Rectangular matrices are supported.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix', '1, 2, 3\n2, 4, 1', '', 'textarea')]),
      t('fp-matrix-rref', 'RREF over Fp', 'Computes the reduced row-echelon form and pivot columns.', 'All output entries are canonical residues.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix', '1, 2, 3\n2, 4, 1', '', 'textarea')]),
      t('fp-matrix-solve', 'Linear system over Fp', 'Classifies a system as unique, inconsistent, or affine.', 'Enter A and the right-hand-side vector separately.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix A', '1, 2\n3, 4', '', 'textarea'), f('rhs', 'Vector b', '1, 0')]),
      t('fp-matrix-inverse', 'Matrix inverse over Fp', 'Inverts a nonsingular square matrix and rejects singular input.', 'The result can be multiplied by A to recover the identity.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix', '1, 2\n3, 4', '', 'textarea')]),
      t('fp-matrix-kernel', 'Kernel over Fp', 'Returns a basis of the nullspace over the prime field.', 'Basis vectors satisfy A·v=0.', [f('prime', 'Prime p', '5'), f('matrix', 'Matrix', '1, 2, 3\n2, 4, 1', '', 'textarea')]),
      t('fp-poly-add', 'Polynomial addition over Fp', 'Adds canonical coefficient vectors over Fp.', 'Coefficients are in ascending order: c0,c1,…', [f('prime', 'Prime p', '5'), f('polynomial', 'Polynomial A', '1, 2, 0, 1'), f('other', 'Polynomial B', '4, 1')]),
      t('fp-poly-sub', 'Polynomial subtraction over Fp', 'Subtracts canonical coefficient vectors over Fp.', 'Trailing zero coefficients are removed.', [f('prime', 'Prime p', '5'), f('polynomial', 'Polynomial A', '1, 2, 0, 1'), f('other', 'Polynomial B', '4, 1')]),
      t('fp-poly-mul', 'Polynomial multiplication over Fp', 'Multiplies dense polynomials over Fp.', 'Coefficients are reduced modulo p.', [f('prime', 'Prime p', '5'), f('polynomial', 'Polynomial A', '1, 2, 0, 1'), f('other', 'Polynomial B', '4, 1')]),
      t('fp-poly-divrem', 'Polynomial division over Fp', 'Returns quotient and canonical remainder.', 'The remainder degree is smaller than the divisor degree.', [f('prime', 'Prime p', '5'), f('polynomial', 'Dividend', '1, 0, 0, 1'), f('other', 'Divisor', '4, 1')]),
      t('fp-poly-gcd', 'Polynomial GCD over Fp', 'Computes the monic greatest common divisor.', 'The zero polynomial is represented by an empty or all-zero list.', [f('prime', 'Prime p', '5'), f('polynomial', 'Polynomial A', '1, 0, 4, 1'), f('other', 'Polynomial B', '4, 0, 1')]),
      t('fp-poly-xgcd', 'Extended polynomial GCD over Fp', 'Returns a monic GCD and Bézout coefficients.', 'sA+tB=gcd exactly over Fp.', [f('prime', 'Prime p', '5'), f('polynomial', 'Polynomial A', '1, 0, 4, 1'), f('other', 'Polynomial B', '4, 0, 1')]),
      t('fp-poly-derivative', 'Formal derivative over Fp', 'Computes the formal derivative with characteristic-p cancellation.', 'Over F5 the derivative of x^5 is zero.', [f('prime', 'Prime p', '5'), f('polynomial', 'Polynomial', '0, 0, 0, 0, 0, 1')]),
      t('fp-poly-evaluate', 'Polynomial evaluation over Fp', 'Evaluates with Horner’s method over the selected field.', 'Coefficients and x are reduced modulo p.', [f('prime', 'Prime p', '5'), f('polynomial', 'Polynomial', '1, 2, 0, 1'), f('x', 'Value x', '2')]),
      t('fp-poly-powmod', 'Polynomial modular power over Fp', 'Computes A^n modulo a nonzero polynomial using binary exponentiation.', 'The output degree stays below the modulus degree.', [f('prime', 'Prime p', '5'), f('polynomial', 'Base polynomial', '1, 1'), f('exponent', 'Exponent n', '20'), f('modulus', 'Modulus polynomial', '1, 0, 1')]),
    ],
  },
  combinatorics: {
    title: 'Modular combinatorics',
    tools: [
      t('modular-combinatorics', 'Modular combinatorics', 'Computes factorial and binomial residues or p-adic valuations without constructing gigantic integers.', 'Uses Legendre, Kummer, Lucas, and Wilson reductions over a prime field.', [
        s('operation', 'Operation', 'binomial-mod', [
          ['binomial-mod', 'Binomial C(n,k) mod p'],
          ['factorial-mod', 'Factorial n! mod p'],
          ['factorial-valuation', 'Factorial valuation vₚ(n!)'],
          ['binomial-valuation', 'Binomial valuation vₚ(C(n,k))'],
        ]),
        f('prime', 'Prime p', '7'),
        f('n', 'Integer n', '10'),
        f('k', 'Integer k', '3'),
      ]),
    ],
  },
  discovery: {
    title: 'Discovery',
    tools: [
      t('guess', 'Sequence guess', 'Tries a few economical, exact hypotheses on a finite sequence.', 'Constant → arithmetic → geometric → polynomial → recurrence', [f('sequence', 'Sequence', '1, 4, 9, 16, 25, 36', '', 'textarea')]),
      t('integer-relation', 'Integer relation', 'Uses PSLQ to seek small integer coefficients for a candidate relation.', 'A candidate, not a proof from approximate inputs', [f('values', 'Values', '1, 1.4142135623730951, 2.8284271247461903', '', 'textarea'), f('tolerance', 'Tolerance', '1e-12'), f('coefficient_limit', 'Coefficient limit', '100')]),
      t('recurrence', 'Recurrence finder', 'Infers a Berlekamp–Massey recurrence and validates all integer terms.', 'Fibonacci → a(n)=a(n−1)+a(n−2)', [f('sequence', 'Sequence', '0, 1, 1, 2, 3, 5, 8, 13', '', 'textarea')]),
      t('recurrence-nth', 'Nth term of a recurrence', 'Computes aₙ modulo p from supplied initial terms and recurrence coefficients.', 'Convention: aₙ=c₁aₙ₋₁+…+cₖaₙ₋ₖ; n may be as large as 10¹⁸.', [f('prime', 'Prime p', '1000000007'), f('initial', 'Initial terms', '0, 1', '', 'textarea'), f('coefficients', 'Recurrence coefficients', '1, 1', '', 'textarea'), f('n', 'Index n', '1000000000000000000')]),
      t('recurrence-infer', 'Infer recurrence and calculate term', 'Infers the minimal recurrence fitting the supplied prefix, then conditionally extrapolates aₙ.', 'The inferred recurrence models the supplied terms; it is not proof of an unknown generating process.', [f('prime', 'Prime p', '101'), f('sequence', 'Supplied sequence prefix', '0, 1, 1, 2, 3, 5, 8, 13', '', 'textarea'), f('n', 'Index n', '100')]),
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
  'fp-matrix-add': ['matrix', '=add', 'prime', 'matrix', 'other'],
  'fp-matrix-sub': ['matrix', '=sub', 'prime', 'matrix', 'other'],
  'fp-matrix-mul': ['matrix', '=mul', 'prime', 'matrix', 'other'],
  'fp-matrix-vector': ['matrix', '=matvec', 'prime', 'matrix', 'vector'],
  'fp-matrix-det': ['matrix', '=det', 'prime', 'matrix'],
  'fp-matrix-rank': ['matrix', '=rank', 'prime', 'matrix'],
  'fp-matrix-rref': ['matrix', '=rref', 'prime', 'matrix'],
  'fp-matrix-solve': ['matrix', '=solve', 'prime', 'matrix', 'rhs'],
  'fp-matrix-inverse': ['matrix', '=inverse', 'prime', 'matrix'],
  'fp-matrix-kernel': ['matrix', '=kernel', 'prime', 'matrix'],
  'fp-poly-add': ['polynomial', '=add', 'prime', 'polynomial', 'other'],
  'fp-poly-sub': ['polynomial', '=sub', 'prime', 'polynomial', 'other'],
  'fp-poly-mul': ['polynomial', '=mul', 'prime', 'polynomial', 'other'],
  'fp-poly-divrem': ['polynomial', '=divrem', 'prime', 'polynomial', 'other'],
  'fp-poly-gcd': ['polynomial', '=gcd', 'prime', 'polynomial', 'other'],
  'fp-poly-xgcd': ['polynomial', '=xgcd', 'prime', 'polynomial', 'other'],
  'fp-poly-derivative': ['polynomial', '=derivative', 'prime', 'polynomial'],
  'fp-poly-evaluate': ['polynomial', '=evaluate', 'prime', 'polynomial', 'x'],
  'fp-poly-powmod': ['polynomial', '=powmod', 'prime', 'polynomial', 'exponent', 'modulus'],
  'recurrence-nth': ['recurrence', '=nth', 'prime', 'initial', 'coefficients', 'n'],
  'recurrence-infer': ['recurrence', '=infer', 'prime', 'n', 'sequence'],
  'primitive-root': ['group', '=primitive-root', 'prime'],
  'is-primitive-root': ['group', '=is-primitive-root', 'prime', 'g'],
  'discrete-log': ['group', '=dlog', 'prime', 'g', 'h'],
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
  wasmLoading.textContent = 'The local mathematics engine is unavailable. Start SwissMath-Web-Portable.exe instead of opening index.html directly.';
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
const matrixResult = document.querySelector('#matrix-result');

function translateText(value) {
  return typeof value === 'string' ? value : value;
}

function translateStructured(value) {
  if (typeof value === 'string') return translateText(value);
  if (Array.isArray(value)) return value.map(translateStructured);
  if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, translateStructured(item)]));
  return value;
}

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
    const input = document.createElement(field.type === 'select' ? 'select' : (multiline ? 'textarea' : 'input'));
    input.name = field.name;
    if (field.type === 'select') {
      input.replaceChildren(...field.options.map(([value, text]) => {
        const option = document.createElement('option');
        option.value = value;
        option.textContent = text;
        return option;
      }));
    }
    input.value = field.value;
    input.required = true;
    if (multiline) input.rows = field.type === 'textarea' ? 4 : 3;
    const help = document.createElement('small');
    help.textContent = field.help || (multiline ? (field.name === 'matrix' ? 'One row per line.' : 'One value per line.') : '');
    label.append(caption, input, help);
    return label;
  }));
  resultArea.classList.add('hidden');
  resultActions.classList.add('hidden');
  batchResults.classList.add('hidden');
  batchResultsBody.replaceChildren();
  matrixResult.classList.add('hidden');
  matrixResult.replaceChildren();
  primary.classList.remove('hidden');
  details.classList.remove('hidden');
  currentResult = null;
  if (currentTool.id === 'modular-combinatorics') {
    form.elements.namedItem('operation').addEventListener('change', updateCombinatoricsFields);
    updateCombinatoricsFields();
  }
}

function updateCombinatoricsFields() {
  if (currentTool.id !== 'modular-combinatorics') return;
  const operation = form.elements.namedItem('operation').value;
  const k = form.elements.namedItem('k');
  const needsK = operation.startsWith('binomial-');
  k.disabled = !needsK;
  k.required = needsK;
  k.closest('label').classList.toggle('hidden', !needsK);
}

function decode(call, payload) {
  if (!wasmReady) throw new Error('The WASM engine is not ready yet.');
  const envelope = JSON.parse(call(JSON.stringify(payload)));
  if (!envelope.ok) throw new Error(translateText(envelope.error || 'Unspecified mathematical error.'));
  return envelope.value;
}

function execute(tool, input) {
  if (tool.command === 'tool') return decode(wasm_run_tool, { tool: tool.id, input });
  if (tool.command === 'modular') return decode(wasm_calculate_modular, input);
  if (tool.command === 'residues') return decode(wasm_calculate_residues, { ...input, left: input.left.split(',').map((value) => value.trim()), right: input.right.split(',').map((value) => value.trim()) });
  if (tool.command === 'integer') return decode(wasm_analyze_integer, input);
  if (tool.command === 'linear') return decode(wasm_solve_linear, input);
  if (tool.command === 'sieve') return decode(wasm_run_sieve, { start: input.start, end: input.end, preview: input.preview, filters: [{ kind: 'excluded', modulus: input.modulus, residues: input.residues.split(',').map((value) => value.trim()), a: null, b: null }] });
  throw new Error('Unrecognized command.');
}

function displayValue(value) {
  const translated = translateStructured(value);
  if (typeof translated === 'string' || typeof translated === 'number' || typeof translated === 'boolean') return String(translated);
  return JSON.stringify(translated, null, 2);
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

function exactnessLabel(value) {
  return { exact: 'Exact', inferred_recurrence: 'Inferred recurrence', bounded_incomplete: 'Bounded incomplete', probable: 'Probable', qualified: 'Qualified', proof_incomplete: 'Proof incomplete', '—': '—' }[value] ?? value;
}

function batchInputs(input) {
  if (!batchToolIds.has(currentTool.id) || typeof input.n !== 'string' || !input.n.includes('\n')) return null;
  const values = input.n.split(/\r?\n/).map((value) => value.trim()).filter(Boolean);
  return values.length > 1 ? values : null;
}

function renderBatch(rows) {
  batchResultsBody.replaceChildren(...rows.map((row) => {
    const tr = document.createElement('tr');
    const values = [row.input, row.result, row.status === 'ok' ? 'OK' : 'Error', exactnessLabel(row.exactness)];
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

function matrixViewsForResult(result) {
  const column = (values) => Array.isArray(values) ? values.map((value) => [value]) : null;
  if (currentTool.id.startsWith('fp-matrix-') && Array.isArray(result.matrix)) return [{ label: 'Matrix over Fp', rows: result.matrix }];
  if (currentTool.id === 'fp-matrix-vector' && Array.isArray(result.vector)) return [{ label: 'Result vector', rows: column(result.vector) }];
  if (currentTool.id === 'fp-matrix-kernel' && Array.isArray(result.basis)) return [{ label: 'Kernel basis', rows: result.basis }];
  if (currentTool.id === 'fp-matrix-solve' && result.kind === 'unique' && Array.isArray(result.solution)) return [{ label: 'Solution vector', rows: column(result.solution) }];
  if (currentTool.id === 'fp-matrix-solve' && result.kind === 'infinite') return [
    ...(Array.isArray(result.particular) ? [{ label: 'Particular solution', rows: column(result.particular) }] : []),
    ...(Array.isArray(result.kernel_basis) ? [{ label: 'Kernel basis', rows: result.kernel_basis }] : []),
  ];
  if (currentTool.id === 'rref' && Array.isArray(result.matrix)) return [{ label: 'Reduced row-echelon form', rows: result.matrix }];
  if (currentTool.id === 'nullspace' && Array.isArray(result.basis)) return [{ label: 'Nullspace basis', rows: result.basis }];
  if (currentTool.id === 'hnf' && Array.isArray(result.matrix)) return [{ label: 'Hermite normal form', rows: result.matrix }];
  if (currentTool.id === 'snf' && Array.isArray(result.invariants)) return [{ label: 'Non-zero Smith invariants', rows: [result.invariants] }];
  if (currentTool.id === 'solve' && result.kind === 'unique' && Array.isArray(result.solution)) return [{ label: 'Solution vector', rows: column(result.solution) }];
  if (currentTool.id === 'solve' && result.kind === 'infinite') {
    return [
      ...(Array.isArray(result.particular) ? [{ label: 'Particular solution', rows: column(result.particular) }] : []),
      ...(Array.isArray(result.nullspace_basis) ? [{ label: 'Nullspace basis', rows: result.nullspace_basis }] : []),
    ];
  }
  return [];
}

function renderMatrixTable(rows) {
  if (!Array.isArray(rows) || !rows.length) return null;
  const table = document.createElement('table');
  table.setAttribute('role', 'grid');
  const body = document.createElement('tbody');
  rows.forEach((row, rowIndex) => {
    const tr = document.createElement('tr');
    const header = document.createElement('th');
    header.scope = 'row';
    header.textContent = String(rowIndex + 1);
    tr.append(header);
    (Array.isArray(row) ? row : [row]).forEach((value) => {
      const cell = document.createElement('td');
      cell.textContent = displayValue(value);
      tr.append(cell);
    });
    body.append(tr);
  });
  table.append(body);
  return table;
}

function renderMatrixViews(views) {
  matrixResult.classList.add('hidden');
  matrixResult.replaceChildren();
  views.forEach(({ label, rows }) => {
    const view = document.createElement('div');
    view.className = 'matrix-view';
    const heading = document.createElement('div');
    heading.className = 'matrix-view-title';
    heading.textContent = label;
    const table = renderMatrixTable(rows);
    if (table) view.append(heading, table);
    matrixResult.append(view);
  });
  if (!views.some(({ rows }) => Array.isArray(rows) && rows.length)) return false;
  matrixResult.classList.remove('hidden');
  return true;
}

function formatDetailValue(value) {
  if (Array.isArray(value)) return value.map(formatDetailValue).join(', ');
  if (value && typeof value === 'object') return Object.entries(value).map(([key, item]) => `${key}: ${formatDetailValue(item)}`).join('; ');
  return displayValue(value);
}

function renderScalar(result) {
  const main = mainResult(result);
  const hasMatrix = renderMatrixViews(matrixViewsForResult(result));
  primary.textContent = displayValue(main);
  const hiddenMatrixKeys = new Set(['result', 'matrix', 'basis', 'nullspace_basis', 'solution', 'particular', 'invariants']);
  const supporting = Object.fromEntries(Object.entries(result).filter(([key]) => !hiddenMatrixKeys.has(key)));
  details.textContent = Object.entries(supporting).map(([key, value]) => `${key}: ${formatDetailValue(value)}`).join('\n');
  primary.classList.toggle('hidden', Boolean(hasMatrix));
  details.classList.remove('hidden');
  if (hasMatrix && !details.textContent) details.classList.add('hidden');
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
    showToast(`Operation completed · time: ${formatElapsed(elapsed)}`);
  } catch (error) {
    const elapsed = performance.now() - started;
    resultArea.classList.add('hidden');
    resultActions.classList.add('hidden');
    batchResults.classList.add('hidden');
    showToast(`${error.message || error} · time: ${formatElapsed(elapsed)}`, true);
  }
});

toolSelect.addEventListener('change', () => {
  currentTool = catalog[currentCategory].tools.find((item) => item.id === toolSelect.value);
  renderTool();
});
categoryButtons.forEach((button) => button.addEventListener('click', () => selectCategory(button.dataset.category)));

function exportPayload() {
  return {
    application: 'SwissMath Web', web_version: '0.5', core_version: '0.9',
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
  if (currentTool.id === 'modular-combinatorics' && currentResult.records.length === 1) {
    const input = currentResult.input;
    const values = ['swissmath', 'comb', input.operation, input.prime, input.n];
    if (input.operation.startsWith('binomial-')) values.push(input.k);
    return values.map(shellValue).join(' ');
  }
  const mapping = cliCommands[currentTool.id];
  if (!mapping || currentResult.records.length !== 1) return null;
  const [, ...fields] = mapping;
  return ['swissmath', mapping[0], ...fields.map((field) => (
    field.startsWith('=') ? field.slice(1) : shellValue(currentResult.input[field])
  ))].join(' ');
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
  showToast('Result copied.');
});

document.querySelector('#copy-json').addEventListener('click', async () => {
  if (!currentResult) return;
  await navigator.clipboard.writeText(JSON.stringify(exportPayload(), null, 2));
  showToast('JSON copied.');
});

document.querySelector('#download-json').addEventListener('click', () => {
  if (!currentResult) return;
  download(`swissmath-${currentTool.id}.json`, JSON.stringify(exportPayload(), null, 2), 'application/json');
  showToast('JSON ready for download.');
});

document.querySelector('#download-csv').addEventListener('click', () => {
  if (!currentResult) return;
  download(`swissmath-${currentTool.id}.csv`, resultCsv(), 'text/csv;charset=utf-8');
  showToast('CSV ready for download.');
});

document.querySelector('#copy-command').addEventListener('click', async () => {
  const command = cliCommand();
  if (!command) return;
  await navigator.clipboard.writeText(command);
  showToast('CLI command copied.');
});

document.querySelector('#share-result').addEventListener('click', async () => {
  const hash = shareHash();
  if (!hash) {
    showToast('The batch is too large for a link: use JSON or CSV.', true);
    return;
  }
  window.location.hash = hash;
  await navigator.clipboard.writeText(window.location.href);
  showToast('Link copied. The calculation will not run automatically.');
});

document.querySelector('#save-result').addEventListener('click', () => {
  if (!currentResult) return;
  const text = `SwissMath Web v0.5 · Core v0.9\n${currentResult.title}\nTime: ${formatElapsed(currentResult.elapsed)}\n\n${plainResult()}\n`;
  download(`swissmath-${currentTool.id}.txt`, text, 'text/plain;charset=utf-8');
  showToast('Result saved.');
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
  updateCombinatoricsFields();
  showToast('Input restored from the link. Press Calculate to run it.');
  return true;
}

if (!restoreShareState()) selectCategory(currentCategory);
