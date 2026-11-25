// L-01 — Charge API Portail AMANE (k6)
//
// Profil de charge mixte : 80 % lecture / 20 % écriture, paliers 20 / 50 / 100
// req/s, 2 minutes par palier. Seuils : http_req_failed < 1 %, p95 < 200 ms sur
// la lecture. Conforme au plan `plan-tests-charge-v3.md` §L-01.
//
// Points d'entrée réels du portail (pile PME en service, projet `amane-pme`) :
//   - ss-node publie l'interface sur ${WEB_PORT:-9001} (hôte 127.0.0.1:9001).
//   - HAProxy (amane-haproxy:5000) mène au primaire PostgreSQL ; non publié.
//   - L'API Django/DRF de l'éditeur (:8000) n'est PAS déployée dans ce banc.
// Les chemins ci-dessous sont ceux du portail réellement exposé derrière ss-node.
//
// Usage :
//   k6 run load/l01_portail.js
//   BASE_URL=http://127.0.0.1:9001 k6 run load/l01_portail.js
//   K6_NO_USAGE_REPORT=true k6 run ... (évite tout appel sortant)

import http from 'k6/http';
import { check } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:9001';

// Latences par nature de requête (le seuil p95 ne porte que sur la lecture).
const latenceLecture = new Trend('lecture_latence', true);
const latenceEcriture = new Trend('ecriture_latence', true);
const echecsConnexion = new Rate('echecs_connexion');

export const options = {
  scenarios: {
    // paliers 20 -> 50 -> 100 req/s, 2 min chacun (6 min au total)
    paliers: {
      executor: 'ramping-arrival-rate',
      startRate: 20,
      timeUnit: '1s',
      preAllocatedVUs: 100,
      maxVUs: 300,
      stages: [
        { target: 20, duration: '2m' },
        { target: 50, duration: '2m' },
        { target: 100, duration: '2m' },
      ],
    },
  },
  thresholds: {
    // Critère d'acceptation L-01 : moins de 1 % d'échecs.
    http_req_failed: ['rate<0.01'],
    // p95 < 200 ms sur la lecture uniquement.
    lecture_latence: ['p(95)<200'],
  },
  // Sortie verbeuse minimale : on veut les agrégats dans le fichier de résultats.
  summaryTrendStats: ['avg', 'min', 'med', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

// Chemins du portail exposé par ss-node.
const LECTURES = ['/', '/amane/', '/amane/login'];
const ECRITURE = { path: '/amane/api/enrolement', method: 'POST' };

export default function () {
  const estLecture = Math.random() < 0.8; // 80 % lecture / 20 % écriture

  if (estLecture) {
    const chemin = LECTURES[Math.floor(Math.random() * LECTURES.length)];
    const rep = http.get(`${BASE_URL}${chemin}`, { tags: { kind: 'lecture' } });
    latenceLecture.add(rep.timings.duration);
    echecsConnexion.add(rep.status === 0);
    check(rep, { 'lecture 2xx/3xx': (r) => r.status >= 200 && r.status < 400 });
  } else {
    // Écriture : enrôlement d'un nœud (code à usage unique attendu ; 4xx = rejet
    // applicatif normal, 0 = connexion rompue).
    const rep = http.post(`${BASE_URL}${ECRITURE.path}`, {}, { tags: { kind: 'ecriture' } });
    latenceEcriture.add(rep.timings.duration);
    echecsConnexion.add(rep.status === 0);
    check(rep, { 'ecriture non 5xx': (r) => r.status < 500 });
  }
}