import fs from 'node:fs';
import path from 'node:path';

const directory = process.argv.slice(2).find(arg => !arg.startsWith('--')) ?? 'docs/evidence/phase-4-acceptance';
const log = fs.readFileSync(path.join(directory, 'final-ui.log'), 'utf8');
const statistics = values => {
  values.sort((a, b) => a - b);
  const rank = p => values[Math.ceil(values.length * p) - 1] ?? null;
  return { samples: values.length, p50_ms: rank(.5), p95_ms: rank(.95), max_ms: values.at(-1) ?? null };
};
const timings = kind => [...log.matchAll(new RegExp(`latency_us:scripted:${kind}:\\d+:(\\d+)`, 'g'))].map(m => Number(m[1]) / 1000);
const native = kind => [...log.matchAll(new RegExp(`latency_us:native:${kind}:(\\d+)`, 'g'))].map(m => Number(m[1]) / 1000);
const memoryPath = path.join(directory, 'final-memory.json');
const memory = fs.existsSync(memoryPath) ? JSON.parse(fs.readFileSync(memoryPath, 'utf8').replace(/^\uFEFF/, '')) : null;
const selection = statistics(timings('selection'));
const pause = statistics(timings('pause'));
const summary = {
  method: 'Action-handler/scripted-dispatch entry through GPUI post-render callback; includes synchronous engine/FFI/snapshot work for scripted actions; excludes OS input transport and compositor presentation',
  workload: 'Four nodes; 10000 retained rows from 160 independent kernel runs, rebased IDs; one additional live kernel owner processes bounded events and save/load',
  selection, pause,
  native_selection: statistics(native('selection')),
  native_pause: statistics(native('pause')),
  complete: log.includes('soak_complete:1800') && /latency_us:scripted:selection:1799:\d+/.test(log),
  failures: log.split(/\r?\n/).filter(line => line.startsWith('soak_failure:')),
  memory: memory && { elapsed_seconds: memory.elapsed_seconds, peak_working_set_bytes: memory.peak_working_set_bytes, failure: memory.failure, all_responding: memory.samples.every(s => s.responding) },
};
summary.pass = summary.complete && !summary.failures.length && selection.samples >= 100 && pause.samples >= 100
  && selection.p95_ms < 100 && pause.p95_ms < 100 && memory !== null && memory.failure === null
  && memory.elapsed_seconds >= 1800 && memory.peak_working_set_bytes > 0 && memory.peak_working_set_bytes <= 1024 ** 3
  && summary.memory.all_responding;
fs.writeFileSync(path.join(directory, 'metrics.json'), JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
if (process.argv.includes('--require-pass') && !summary.pass) process.exitCode = 1;
