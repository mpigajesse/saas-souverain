// L-02 — Charge du relais zero-knowledge (k6)
//
// Le relais stocke des blobs opaques (zero-knowledge) : il ne voit que des
// octets chiffrés par le nœud. On mesure son débit et sa latence sous charge
// mixte PUT/GET, avec un jeton de tenant valide (HMAC, en-tête x-relay-token).
//
// Cible (reconstruite du brief, plan-tests-charge-v3.md absent du dépôt) :
//   - débit soutenu >= 50 req/s,
//   - http_req_failed < 1 %,
//   - p95 < 200 ms.
//
// Usage :
//   RELAY_URL=http://127.0.0.1:18099 RELAY_TOKEN=<hex> TENANT_ID=<uuid> \
//     K6_NO_USAGE_REPORT=true k6 run load/l02_relais.js
//
// Le jeton se calcule avec docs/tests/artifacts-l02-12/jeton.py.

import http from 'k6/http';
import { check } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const BASE = __ENV.RELAY_URL || 'http://127.0.0.1:18099';
const TOKEN = __ENV.RELAY_TOKEN || '';
const TENANT = __ENV.TENANT_ID || 'aaaaaaaa-1111-4111-8111-aaaaaaaaaaaa';
const KEY = __ENV.BLOB_KEY || 'charge-l02';

// Corps de blob opaque (simule un blob chiffré ; le relais n'en connait pas
// le contenu). 32 Kio, taille représentative d'un petit objet.
const CORPS = 'x'.repeat(32 * 1024);

const lat = new Trend('relais_latence', true);
const debit = new Trend('relais_debit', true);
const refus = new Rate('relais_refus');

export const options = {
  scenarios: {
    charge: {
      executor: 'ramping-arrival-rate',
      startRate: 10,
      timeUnit: '1s',
      preAllocatedVUs: 50,
      maxVUs: 200,
      stages: [
        { target: 20, duration: '1m' },
        { target: 50, duration: '1m' },
        { target: 100, duration: '1m' },
      ],
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    relais_latence: ['p(95)<200'],
  },
  summaryTrendStats: ['avg', 'min', 'med', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

const url = `${BASE}/api/blobs/${TENANT}/${KEY}`;
const params = { headers: { 'x-relay-token': TOKEN } };

export default function () {
  // 50 % PUT / 50 % GET, sur le même blob (le relais doit tenir les deux).
  if (Math.random() < 0.5) {
    const rep = http.put(url, CORPS, params);
    lat.add(rep.timings.duration);
    refus.add(rep.status === 401 || rep.status === 403);
    check(rep, { 'PUT 2xx': (r) => r.status >= 200 && r.status < 300 });
  } else {
    const rep = http.get(url, params);
    lat.add(rep.timings.duration);
    refus.add(rep.status === 401 || rep.status === 403);
    check(rep, { 'GET 2xx': (r) => r.status >= 200 && r.status < 300 });
  }
}