// Artifact integrity only: these checks do not assess a person's understanding.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const output = path.join(root, 'cognitive-coverage');
const read = name => fs.readFileSync(path.join(output, name), 'utf8');
const manifest = JSON.parse(read('cognitive-coverage.json'));
const guide = read('learning-guide.html');
const sections = [...guide.matchAll(/<section id="([^"]+)"/g)].map(m => m[1]);
const navigation = [...guide.matchAll(/<a href="#([^"]+)"/g)].map(m => m[1]);
assert.equal(new Set(sections).size, sections.length, 'Duplicate section ID');
// Only the first nav block is the table of contents; quiz explanations also link back.
const toc = guide.match(/<nav[\s\S]*?<\/nav>/)[0];
const anchors = [...toc.matchAll(/href="#([^"]+)"/g)].map(m => m[1]);
assert.deepEqual([...anchors].sort(), [...sections].sort(), 'TOC/sections differ');
for (const anchor of navigation) assert(sections.includes(anchor), `Missing anchor ${anchor}`);
for (const axis of ['files', 'concepts', 'flows']) {
    for (const item of manifest[axis]) {
        assert(manifest.statusLabels[axis].includes(item.status), `Bad ${axis} status`);
        assert(sections.includes(item.guideSection), `Missing ${axis} teaching section`);
    }
}
for (const item of manifest.files) {
    const bytes = fs.readFileSync(path.join(root, item.path));
    assert.equal(crypto.createHash('sha256').update(bytes).digest('hex'), item.sourceHash,
        `Source changed: ${item.path}; refresh the affected learning artifact`);
}
const quiz = [...guide.matchAll(/data-quiz="(q\d+)" data-difficulty="([^"]+)" data-depth="([^"]+)"/g)];
assert.equal(quiz.length, 18);
assert.equal(Object.keys(manifest.quizMapping).length, quiz.length);
for (const difficulty of ['beginner', 'intermediate', 'advanced']) {
    for (const depth of ['overview', 'standard', 'deep-dive']) {
        assert.equal(quiz.filter(q => q[2] === difficulty && q[3] === depth).length, 2);
    }
}
for (const q of quiz) {
    const mapping = manifest.quizMapping[q[1]];
    assert.equal(mapping.difficulty, q[2]);
    assert.equal(mapping.depth, q[3]);
    for (const file of mapping.files) assert(manifest.files.some(f => f.path === file));
    for (const concept of mapping.concepts) assert(manifest.concepts.some(c => c.id === concept));
    for (const flow of mapping.flows) assert(manifest.flows.some(f => f.id === flow));
}
for (const name of ['learning-guide.html', 'cognitive-coverage.html', 'cognitive-coverage-open.html']) {
    const body = read(name);
    assert(body.startsWith('<!DOCTYPE html>') && body.endsWith('</html>'), `Incomplete ${name}`);
    assert(!/<(?:script|link)[^>]+(?:src|href)="https?:/i.test(body), `Remote dependency ${name}`);
}
console.log(JSON.stringify({ result: 'pass', files: manifest.files.length,
    concepts: manifest.concepts.length, flows: manifest.flows.length, quiz: quiz.length,
    note: 'Static integrity only; browser interaction is tested separately.' }, null, 2));
