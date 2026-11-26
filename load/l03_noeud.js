// L-03 — Charge du nœud Rust (ss-node) (k6)
//
// ss-node sert l'application hébergée (/) et sa console AMANE (/amane/). On
// mesure le débit et la latence du nœud sous charge, au travers de l'entrée
// réelle publiée par le banc (hôte 127.0.0.1:9011 -> ss-node).
//
// Cible (reconstruite du brief) :
//   - débit soutenu >= 100 req/s,
//   - http_req_failed < 1 %,
//   - p95 < 300 ms.
//
// Usage :
//   BASE_URL=http://127.0.0.1:9011 K6_NO_USAGE_REPORT=true k6 run load/l03_noeud.js

import http from 'k6/http';
import { check } from 'k6';
import { Trend } from 'k6/metrics';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:9011';
const lat = new Trend('noeud_latence', true);

export const options = {
  scenarios: {
    charge: {
      executor: 'ramping-arrival-rate',
      startRate: 20,
      timeUnit: '1s',
      preAllocatedVUs: 100,
      maxVUs: 300,
      stages: [
        { target: 50, duration: '1m' },
        { target: 100, duration: '1m' },
        { target: 200, duration: '1m' },
      ],
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    noeud_latence: ['p(95)<300'],
  },
  summaryTrendStats: ['avg', 'min', 'med', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

// '/amane/' renvoie 404 dans ce déploiement (listmonk sert toutes les routes ;
// la console AMANE n'y est pas exposée) — exclu des chemins de charge.
const LECTURES = ['/', '/health'];

export default function () {
  const chemin = LECTURES[Math.floor(Math.random() * LECTURES.length)];
  const rep = http.get(`${BASE}${chemin}`);
  lat.add(rep.timings.duration);
  check(rep, { 'noeud 2xx/3xx': (r) => r.status >= 200 && r.status < 400 });
}