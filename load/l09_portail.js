// L-09 — Charge du portail éditeur multi-tenant (k6)
//
// ATTENTION : le portail Django/DRF réel (:8000, multi-tenant) n'est PAS
// déployé sur le banc. Le banc n'expose qu'un portail factice mono-tenant
// (tools/bench/portail-mock.py, m1:5173) qui imite /api/devices/cluster-status
// et /api/devices/register/. Ce script mesure donc le DÉBIT du portail factice,
// mais l'isolation multi-tenant n'est pas vérifiable ici (cause racine du
// « non tenu » L-09).
//
// Cible (reconstruite du brief) :
//   - débit >= 100 req/s, http_req_failed < 1 %, p95 < 300 ms,
//   - isolation multi-tenant (NON mesurable : portail mono-tenant).
//
// Usage :
//   BASE_URL=http://10.77.0.11:5173 K6_NO_USAGE_REPORT=true k6 run load/l09_portail.js

import http from 'k6/http';
import { check } from 'k6';
import { Trend } from 'k6/metrics';

const BASE = __ENV.BASE_URL || 'http://10.77.0.11:5173';
const lat = new Trend('portail_latence', true);

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
    portail_latence: ['p(95)<300'],
  },
  summaryTrendStats: ['avg', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

export default function () {
  // 80 % lecture (cluster-status) / 20 % écriture (register).
  if (Math.random() < 0.8) {
    const rep = http.get(`${BASE}/api/devices/cluster-status/`);
    lat.add(rep.timings.duration);
    check(rep, { 'cluster-status 2xx': (r) => r.status >= 200 && r.status < 300 });
  } else {
    const rep = http.post(`${BASE}/api/devices/register/`,
      JSON.stringify({ node_addr: '10.77.0.99:9001' }),
      { headers: { 'Content-Type': 'application/json' } });
    lat.add(rep.timings.duration);
    check(rep, { 'register 2xx': (r) => r.status >= 200 && r.status < 300 });
  }
}