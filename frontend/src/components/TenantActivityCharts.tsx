import { useState, useEffect } from 'react'

interface TenantActivityChartsProps {
  tenantName: string
}

export default function TenantActivityCharts({ tenantName }: TenantActivityChartsProps) {
  // Génération de points d'activité en temps réel pour ce Tenant PME spécifique
  const [activityPoints, setActivityPoints] = useState<number[]>([15, 32, 28, 45, 60, 52, 75, 68, 85, 92])
  const [detectedIssuesCount, setDetectedIssuesCount] = useState<number>(1)
  const [cpuUsage, setCpuUsage] = useState<number>(14.2)

  useEffect(() => {
    const interval = setInterval(() => {
      setActivityPoints((prev) => {
        const next = [...prev.slice(1)]
        next.push(Math.floor(Math.random() * 45) + 50)
        return next
      })

      setCpuUsage(parseFloat((12 + Math.random() * 8).toFixed(1)))
    }, 2000)

    return () => clearInterval(interval)
  }, [])

  const svgPathD = activityPoints.reduce((acc, point, idx) => {
    const x = idx * 40
    const y = 100 - point * 0.8
    return idx === 0 ? `M ${x} ${y}` : `${acc} L ${x} ${y}`
  }, '')

  const svgAreaD = `${svgPathD} L ${9 * 40} 110 L 0 110 Z`

  return (
    <div style={{ marginBottom: '28px' }}>
      <h2 style={{ borderLeft: '3px solid var(--crimson)', paddingLeft: '12px', fontSize: '16px', fontWeight: 600, marginBottom: '16px', color: 'var(--text-primary)' }}>
        Télémétrie & Activité en Temps Réel — {tenantName}
      </h2>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
        {/* GRAPHIQUE 1 : Activité des Mouvements Métier gRPC */}
        <div
          style={{
            background: '#FFFFFF',
            borderRadius: 'var(--radius)',
            border: '1px solid var(--border)',
            borderTop: '3px solid var(--crimson)',
            padding: '20px',
            boxShadow: 'var(--shadow)',
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
            <div>
              <h3 style={{ fontSize: '14px', fontWeight: 700, color: 'var(--text-primary)', margin: 0 }}>
                Volume des Transactions Métier gRPC
              </h3>
              <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>Écritures au journal CBOR chiffré</span>
            </div>
            <span style={{ fontSize: '12px', fontWeight: 700, color: 'var(--crimson)', fontFamily: 'monospace' }}>
              {activityPoints[activityPoints.length - 1]} ops/min
            </span>
          </div>

          <div style={{ background: '#FAF7F4', borderRadius: '6px', padding: '10px 4px 4px 4px', border: '1px solid var(--border)' }}>
            <svg viewBox="0 0 360 110" style={{ width: '100%', height: '110px' }}>
              <defs>
                <linearGradient id="grad-tenant-act" x1="0%" y1="0%" x2="0%" y2="100%">
                  <stop offset="0%" stopColor="var(--crimson)" stopOpacity="0.3" />
                  <stop offset="100%" stopColor="var(--gold)" stopOpacity="0.0" />
                </linearGradient>
              </defs>

              <line x1="0" y1="30" x2="360" y2="30" stroke="var(--border)" strokeDasharray="3 3" />
              <line x1="0" y1="60" x2="360" y2="60" stroke="var(--border)" strokeDasharray="3 3" />

              <path d={svgAreaD} fill="url(#grad-tenant-act)" />
              <path d={svgPathD} fill="none" stroke="var(--crimson)" strokeWidth="2.5" style={{ transition: 'd 0.5s ease' }} />
            </svg>
          </div>
        </div>

        {/* GRAPHIQUE 2 & INDICATEURS : Problèmes Détectés & Métriques Infra */}
        <div
          style={{
            background: '#FFFFFF',
            borderRadius: 'var(--radius)',
            border: '1px solid var(--border)',
            borderTop: '3px solid var(--gold)',
            padding: '20px',
            boxShadow: 'var(--shadow)',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
          }}
        >
          <div>
            <h3 style={{ fontSize: '14px', fontWeight: 700, color: 'var(--text-primary)', marginBottom: '14px' }}>
              Métriques de Santé & Problèmes Détectés
            </h3>

            {/* Cartes Métriques PME */}
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', marginBottom: '16px' }}>
              <div style={{ background: '#F8F9FA', padding: '12px', borderRadius: '6px', border: '1px solid var(--border)' }}>
                <div style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>Charge CPU Cluster</div>
                <div style={{ fontSize: '20px', fontWeight: 700, color: '#2E7D32' }}>{cpuUsage}%</div>
                <div style={{ fontSize: '10px', color: 'var(--text-muted)' }}>Nominal (3 Nœuds)</div>
              </div>

              <div style={{ background: '#FFF3CD', padding: '12px', borderRadius: '6px', border: '1px solid #FFEBAA' }}>
                <div style={{ fontSize: '11px', color: '#856404' }}>Anomalies Bloquées</div>
                <div style={{ fontSize: '20px', fontWeight: 700, color: '#856404' }}>{detectedIssuesCount} alerte</div>
                <div style={{ fontSize: '10px', color: '#856404' }}>Tentative stock négatif</div>
              </div>
            </div>
          </div>

          {/* Journal des Dernières Détections */}
          <div style={{ background: '#FAF7F4', padding: '10px 12px', borderRadius: '6px', border: '1px solid var(--border)', fontSize: '12px' }}>
            <div style={{ fontWeight: 600, color: 'var(--text-primary)', marginBottom: '4px', display: 'flex', alignItems: 'center', gap: '6px' }}>
              <span>🛡️</span> Dernier événement d'intégrité :
            </div>
            <div style={{ color: 'var(--text-secondary)' }}>
              « Invariant SDK Rust : Tentative de sortie de stock négative bloquée avec succès. Aucune corruption de données. »
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
