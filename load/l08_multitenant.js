// L-08 — Isolation multi-tenant du relais SOUS CHARGE (k6)
//
// Deux tenants A et B, chacun son jeton. Sous charge concurrente, chaque VU
// n'accède qu'à son propre blob (doit aboutir 2xx) et, périodiquement, tente
// un accès croisé (jeton A sur blob de B et inversement) qui doit être
// refusé (401/403). Une seule réponse 2xx à un accès croisé = violation.
//
// Cible : 0 violation d'isolation sous charge, http_req_failed < 1 %.
//
// Usage :
//   RELAY_URL=... RELAY_TOKEN_A=... RELAY_TOKEN_B=... \
//     TENANT_A=... TENANT_B=... K6_NO_USAGE_REPORT=true k6 run load/l08_multitenant.js

import http from 'k6/http';
import { check } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const BASE = __ENV.RELAY_URL || 'http://127.0.0.1:18099';
const TA = __ENV.TENANT_A || 'aaaaaaaa-1111-4111-8111-aaaaaaaaaaaa';
const TB = __ENV.TENANT_B || 'bbbbbbbb-2222-4222-8222-bbbbbbbbbbbb';
const JA = __ENV.RELAY_TOKEN_A || '';
const JB = __ENV.RELAY_TOKEN_B || '';

const lat = new Trend('relais_latence', true);
const violations = new Rate('violations_isolation');
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
    // Aucune violation d'isolation tolérée.
    violations_isolation: ['rate==0'],
    http_req_failed: ['rate<0.01'],
  },
  summaryTrendStats: ['avg', 'p(95)', 'p(99)', 'max'],
};

const CORPS = 'y'.repeat(16 * 1024);

export default function () {
  const estA = Math.random() < 0.5;
  const t = estA ? TA : TB;
  const j = estA ? JA : JB;
  const autreT = estA ? TB : TA;
  const autreJ = estA ? JB : JA;

  // 1) Accès légitime au blob du tenant courant : doit aboutir 2xx.
  const url = `${BASE}/api/blobs/${t}/charge-l08`;
  const rep = http.put(url, CORPS, { headers: { 'x-relay-token': j } });
  lat.add(rep.timings.duration);
  refus.add(rep.status === 401 || rep.status === 403);
  check(rep, { 'acces legitime 2xx': (r) => r.status >= 200 && r.status < 300 });

  // 2) Accès croisé (jeton du tenant courant sur blob de l'autre) : doit
  //    être refusé. Toute réponse 2xx est une violation d'isolation.
  const urlCroise = `${BASE}/api/blobs/${autreT}/charge-l08`;
  const repC = http.get(urlCroise, { headers: { 'x-relay-token': j } });
  const viole = repC.status >= 200 && repC.status < 300;
  violations.add(viole);
  check(repC, { 'acces croise refuse': (r) => r.status === 401 || r.status === 403 });
}