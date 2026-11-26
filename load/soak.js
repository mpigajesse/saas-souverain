// (7) — Soak : charge modérée prolongée (k6)
//
// Détecte les dégradations lentes (fuite mémoire, saturation de pool,
// latence qui dérive) sous une charge modérée tenue dans la durée.
// Profil : 30 req/s constants pendant 10 min contre l'entrée du banc (:9011).
//
// Cible : http_req_failed < 1 %, p95 < 300 ms, pas de dérive de latence
// entre le début et la fin (comparaison des moitiés dans le rapport).
//
// Usage :
//   BASE_URL=http://127.0.0.1:9011 K6_NO_USAGE_REPORT=true k6 run load/soak.js

import http from 'k6/http';
import { check } from 'k6';
import { Trend } from 'k6/metrics';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:9011';
const lat = new Trend('soak_latence', true);

export const options = {
  scenarios: {
    soak: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: __ENV.SOAK_DUREE || '10m',
      preAllocatedVUs: 50,
      maxVUs: 150,
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    soak_latence: ['p(95)<300'],
  },
  summaryTrendStats: ['avg', 'med', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

export default function () {
  const rep = http.get(`${BASE}/health`);
  lat.add(rep.timings.duration);
  check(rep, { 'soak 2xx/3xx': (r) => r.status >= 200 && r.status < 400 });
}