// L-11 — HAProxy sous charge (k6)
//
// HAProxy (amane-haproxy:5000) mène toujours au primaire PostgreSQL ; il
// n'est pas publié. Le trafic applicatif réel le traverse (ss-node -> HAProxy
// -> primaire). On charge donc l'entrée publiée (:9011) à débit élevé et on
// échantillonne en parallèle l'état du backend HAProxy (helper shell).
//
// Cible (reconstruite du brief) :
//   - débit soutenu >= 200 req/s, http_req_failed < 1 %, p95 < 300 ms,
//   - backend HAProxy = 1 serveur sain, aucune erreur 5xx.
//
// Usage :
//   BASE_URL=http://127.0.0.1:9011 K6_NO_USAGE_REPORT=true k6 run load/l11_haproxy.js

import http from 'k6/http';
import { check } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:9011';
const lat = new Trend('haproxy_latence', true);
const cinqxx = new Rate('haproxy_5xx');

export const options = {
  scenarios: {
    charge: {
      executor: 'constant-arrival-rate',
      rate: 200,
      timeUnit: '1s',
      duration: '3m',
      preAllocatedVUs: 200,
      maxVUs: 400,
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    haproxy_5xx: ['rate<0.001'],
    haproxy_latence: ['p(95)<300'],
  },
  summaryTrendStats: ['avg', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

export default function () {
  const rep = http.get(`${BASE}/health`);
  lat.add(rep.timings.duration);
  cinqxx.add(rep.status >= 500);
  check(rep, { 'haproxy amont 2xx/3xx': (r) => r.status >= 200 && r.status < 400 });
}