import { useState, useEffect } from 'react'

interface GrafanaPanelProps {
  title: string
  metricType: 'GRPC_LATENCY' | 'CLUSTER_NODES' | 'LICENSE_VERIFICATIONS'
  refreshRateSeconds?: number
}

export default function GrafanaEmbed({ title, metricType }: GrafanaPanelProps) {
  const [useLiveIframe, setUseLiveIframe] = useState(false)
  
  // Données de points dynamiques animés en temps réel
  const [wavePoints, setWavePoints] = useState<number[]>([40, 65, 30, 85, 45, 70, 55, 90, 60, 75])
  const [liveLatency, setLiveLatency] = useState<number>(2.4)
  const [liveVerifications, setLiveVerifications] = useState<number>(1420)

  // Boucle d'animation en temps réel toutes les 1.5 secondes (Animation vivante qui bouge)
  useEffect(() => {
    const interval = setInterval(() => {
      setWavePoints((prev) => {
        const nextPoints = [...prev.slice(1)]
        // Génère un nouveau point de télémétrie aléatoire mais réaliste
        const newPoint = Math.floor(Math.random() * 50) + 35
        nextPoints.push(newPoint)
        return nextPoints
      })

      // Mettre à jour la latence en direct (ex: 2.1 ms -> 2.6 ms)
      const newLat = parseFloat((2.0 + Math.random() * 0.8).toFixed(2))
      setLiveLatency(newLat)

      // Ingrémenter les vérifications de licences
      setLiveVerifications((v) => v + Math.floor(Math.random() * 3))
    }, 1500)

    return () => clearInterval(interval)
  }, [])

  // Construction de la courbe SVG dynamique fluide
  const svgPathD = wavePoints.reduce((acc, point, idx) => {
    const x = idx * 42
    const y = 110 - point
    return idx === 0 ? `M ${x} ${y}` : `${acc} L ${x} ${y}`
  }, '')

  const svgAreaD = `${svgPathD} L ${9 * 42} 120 L 0 120 Z`

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
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '14px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span
            style={{
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              background: 'var(--crimson)',
              boxShadow: '0 0 8px var(--crimson)',
              animation: 'pulse 1.5s infinite',
            }}
          />
          <h4 style={{ fontSize: '14px', fontWeight: 700, color: 'var(--text-primary)', margin: 0 }}>{title}</h4>
        </div>

        <button
          onClick={() => setUseLiveIframe(!useLiveIframe)}
          style={{
            background: useLiveIframe ? 'var(--crimson)' : 'rgba(196,151,42,0.12)',
            color: useLiveIframe ? '#FFFFFF' : 'var(--crimson-dark)',
            border: '1px solid var(--gold)',
            borderRadius: '4px',
            padding: '4px 8px',
            fontSize: '11px',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          {useLiveIframe ? 'Mode Live Iframe (Port 3000)' : 'Mode Intégré En Direct 🔴'}
        </button>
      </div>

      {useLiveIframe ? (
        <iframe
          src="http://localhost:3000/d-solo/amane-telemetry?panelId=1&refresh=5s"
          width="100%"
          height="160"
          frameBorder="0"
          title={title}
          style={{ borderRadius: '4px' }}
        />
      ) : (
        <div>
          {/* Métrique en direct */}
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', marginBottom: '8px' }}>
            <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>
              {metricType === 'GRPC_LATENCY' && 'Latence gRPC en direct'}
              {metricType === 'CLUSTER_NODES' && 'Disponibilité Raft / etcd'}
              {metricType === 'LICENSE_VERIFICATIONS' && 'Vérifications Ed25519 totales'}
            </span>
            <span style={{ fontSize: '16px', fontWeight: 700, color: 'var(--crimson)', fontFamily: 'monospace' }}>
              {metricType === 'GRPC_LATENCY' && `${liveLatency} ms`}
              {metricType === 'CLUSTER_NODES' && '99.9% (3/3 Nœuds)'}
              {metricType === 'LICENSE_VERIFICATIONS' && `${liveVerifications.toLocaleString()} jetons`}
            </span>
          </div>

          {/* Graphique SVG Animé en Temps Réel respectant la charte visuelle (Crimson + Gold) */}
          <div style={{ background: '#FAF7F4', borderRadius: '6px', padding: '10px 4px 4px 4px', border: '1px solid var(--border)' }}>
            <svg viewBox="0 0 378 120" style={{ width: '100%', height: '120px', overflow: 'visible' }}>
              <defs>
                <linearGradient id={`grad-${metricType}`} x1="0%" y1="0%" x2="0%" y2="100%">
                  <stop offset="0%" stopColor="var(--crimson)" stopOpacity="0.25" />
                  <stop offset="100%" stopColor="var(--gold)" stopOpacity="0.0" />
                </linearGradient>
              </defs>

              {/* Lignes de grille en arrière-plan */}
              <line x1="0" y1="30" x2="378" y2="30" stroke="var(--border)" strokeDasharray="3 3" />
              <line x1="0" y1="60" x2="378" y2="60" stroke="var(--border)" strokeDasharray="3 3" />
              <line x1="0" y1="90" x2="378" y2="90" stroke="var(--border)" strokeDasharray="3 3" />

              {/* Remplissage dégradé sous la courbe */}
              <path d={svgAreaD} fill={`url(#grad-${metricType})`} />

              {/* Courbe animée dynamique bordeaux / crimson */}
              <path
                d={svgPathD}
                fill="none"
                stroke="var(--crimson)"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                style={{ transition: 'd 0.5s ease-in-out' }}
              />

              {/* Point de pulsation en direct sur la fin de la courbe */}
              {wavePoints.length > 0 && (
                <circle
                  cx={378}
                  cy={110 - wavePoints[wavePoints.length - 1]}
                  r="5"
                  fill="var(--gold)"
                  stroke="var(--crimson)"
                  strokeWidth="2"
                />
              )}
            </svg>
          </div>
        </div>
      )}
    </div>
  )
}
