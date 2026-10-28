// Intégration Grafana — pas de télémétrie fabriquée : ce composant se contente
// d'intégrer un panneau Grafana réel (iframe) et de proposer un lien de repli
// s'il ne peut pas s'afficher (X-Frame-Options, instance injoignable, etc.).
//
// L'URL de l'instance Grafana est configurable via la variable d'environnement
// VITE_GRAFANA_URL (ex: https://grafana.exemple.fr). À défaut, on retombe sur
// l'instance locale de développement http://localhost:3000 — sans garantie
// qu'elle soit joignable en dehors d'un poste de dev.
import { useState } from 'react'
import { Activity, ExternalLink } from 'lucide-react'

interface GrafanaPanelProps {
  title: string
  /** Type de métrique affichée par le panneau ciblé (informatif, sert au libellé). */
  metricType: 'GRPC_LATENCY' | 'CLUSTER_NODES' | 'LICENSE_VERIFICATIONS'
  /** Intervalle de rafraîchissement du panneau Grafana, en secondes. */
  refreshRateSeconds?: number
}

const GRAFANA_BASE_URL: string =
  (import.meta.env.VITE_GRAFANA_URL as string | undefined) || 'http://localhost:3000'

const METRIC_LABELS: Record<GrafanaPanelProps['metricType'], string> = {
  GRPC_LATENCY: 'Latence réseau',
  CLUSTER_NODES: 'Disponibilité des nœuds',
  LICENSE_VERIFICATIONS: 'Vérifications de licence',
}

export default function GrafanaEmbed({ title, metricType, refreshRateSeconds = 30 }: GrafanaPanelProps) {
  const [iframeFailed, setIframeFailed] = useState(false)
  const src = `${GRAFANA_BASE_URL}/?kiosk&refresh=${refreshRateSeconds}s`

  return (
    <div
      style={{
        background: '#FFFFFF',
        borderRadius: 'var(--radius)',
        border: '1px solid var(--border)',
        borderTop: '3px solid var(--gold)',
        padding: '18px',
        color: 'var(--text-primary)',
        boxShadow: 'var(--shadow)',
        flex: 1,
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Activity size={15} color="var(--gold)" />
          <h4 style={{ fontSize: '14px', fontWeight: 700, color: 'var(--text-primary)', margin: 0 }}>{title}</h4>
        </div>
        <a
          href={GRAFANA_BASE_URL}
          target="_blank"
          rel="noopener noreferrer"
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: '4px',
            fontSize: '11px',
            fontWeight: 600,
            color: 'var(--crimson-dark)',
            textDecoration: 'none',
          }}
        >
          Ouvrir dans Grafana <ExternalLink size={12} />
        </a>
      </div>

      <div style={{ fontSize: '11px', color: 'var(--text-secondary)', marginBottom: '8px' }}>
        {METRIC_LABELS[metricType]}
      </div>

      {iframeFailed ? (
        <div
          style={{
            background: '#FAF7F4',
            border: '1px solid var(--border)',
            borderRadius: '6px',
            padding: '20px 12px',
            textAlign: 'center',
            fontSize: '12px',
            color: 'var(--text-muted)',
          }}
        >
          Le panneau Grafana n'a pas pu s'afficher ici. Utilisez le lien « Ouvrir dans Grafana » ci-dessus.
        </div>
      ) : (
        <iframe
          src={src}
          width="100%"
          height="200"
          style={{ border: 'none', borderRadius: '4px' }}
          title={title}
          onError={() => setIframeFailed(true)}
        />
      )}
    </div>
  )
}
