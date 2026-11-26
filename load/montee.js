// (8) — Montée en charge : jusqu'au point de rupture (k6)
//
// Rampe de débit continue (20 -> 400 req/s sur 6 min) pour trouver le débit
// maximal soutenable avant l'apparition d'échecs ou d'une latence qui
// s'effondre. Le « point de rupture » est le dernier palier sans échec.
//
// Cible : identifier le débit max soutenable (>= 200 req/s attendu) et le
// débit auquel http_req_failed dépasse 1 %.
//
// Usage :
//   BASE_URL=http://127.0.0.1:9011 K6_NO_USAGE_REPORT=true k6 run load/montee.js

import http from 'k6/http';
import { check } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:9011';
const lat = new Trend('montee_latence', true);
const echecs = new Rate('montee_echecs');

export const options = {
  scenarios: {
    montee: {
      executor: 'ramping-arrival-rate',
      startRate: 20,
      timeUnit: '1s',
      preAllocatedVUs: 200,
      maxVUs: 600,
      stages: [
        { target: 50, duration: '1m' },
        { target: 100, duration: '1m' },
        { target: 200, duration: '1m' },
        { target: 300, duration: '1m' },
        { target: 400, duration: '1m' },
        { target: 400, duration: '1m' },
      ],
    },
  },
  // Pas de seuil bloquant : on veut la courbe complète jusqu'à la rupture.
  summaryTrendStats: ['avg', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

export default function () {
  const rep = http.get(`${BASE}/health`);
  lat.add(rep.timings.duration);
  echecs.add(rep.status === 0 || rep.status >= 500);
  check(rep, { 'montee 2xx/3xx': (r) => r.status >= 200 && r.status < 400 });
}