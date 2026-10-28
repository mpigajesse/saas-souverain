// Relais zero-knowledge — remplace templates/dashboard/relay.html.
// Le relais peut être injoignable : c'est un état normal, affiché calmement.

import { useCallback, useEffect, useState } from 'react'
import { AlertCircle, Boxes, Clock, Database, Lock, Loader2, RotateCw, Server, Wifi, WifiOff } from 'lucide-react'
import { getRelayStatus, normalizeApiError } from '../api/client'
import type { RelayStatus } from '../types/index'

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
      <Loader2 size={18} className="rl-spin" />
      <span>Chargement de l'état du relais...</span>
      <style>{`.rl-spin { animation: rl-spin 0.8s linear infinite; } @keyframes rl-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
    </div>
  )
}

function ErrorState({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        gap: '12px',
        background: '#FDECEC',
        border: '1px solid #F3B7B7',
        color: 'var(--danger)',
        borderRadius: 'var(--radius)',
        padding: '18px 20px',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <AlertCircle size={18} />
        <span>{message}</span>
      </div>
      <button
        onClick={onRetry}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: '#FFFFFF',
          border: '1px solid var(--danger)',
          color: 'var(--danger)',
          borderRadius: '6px',
          padding: '6px 12px',
          fontSize: '12px',
          fontWeight: 600,
          cursor: 'pointer',
        }}
      >
        <RotateCw size={14} /> Réessayer
      </button>
    </div>
  )
}

function optionalString(value: unknown): string | null {
  return typeof value === 'string' && value.length > 0 ? value : null
}

function CharItem({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ background: 'rgba(255,255,255,.05)', border: '1px solid rgba(255,255,255,.1)', borderRadius: '10px', padding: '10px 16px', minWidth: '120px' }}>
      <div style={{ fontSize: '0.67rem', textTransform: 'uppercase', letterSpacing: '.08em', color: 'rgba(255,255,255,.38)', marginBottom: '4px' }}>{label}</div>
      <div style={{ fontSize: '0.88rem', fontWeight: 700, color: '#fff', fontFamily: 'monospace' }}>{value}</div>
    </div>
  )
}

export default function RelayPage() {
  const [data, setData] = useState<RelayStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const status = await getRelayStatus()
      setData(status)
    } catch (err) {
      // Ici, seule une vraie panne de communication avec le SaaS éditeur (pas le relais) est une erreur.
      setError(normalizeApiError(err).message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    // Chargement initial differe d'une micro-tache : appeler load() dans la
    // phase synchrone de l'effet declenche des rendus en cascade (React 19,
    // regle react-hooks/set-state-in-effect).
    queueMicrotask(() => {
      void load()
    })
  }, [load])

  return (
    <div>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>Relais zero-knowledge</h1>
        <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
          Service stateless de rendez-vous multi-sites — il ne voit jamais les données métier en clair
        </p>
      </div>

      {loading && <LoadingState />}
      {!loading && error && <ErrorState message={error} onRetry={() => void load()} />}

      {!loading && !error && data && (
        <>
          {/* Bandeau d'état */}
          <div
            style={{
              background: 'linear-gradient(135deg, #0f172a 0%, #1e1b4b 60%, #312e81 100%)',
              borderRadius: '16px',
              padding: '26px 30px',
              marginBottom: '28px',
              color: '#fff',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', flexWrap: 'wrap', gap: '16px', marginBottom: '20px' }}>
              <div>
                <div style={{ fontSize: '1.3rem', fontWeight: 800, marginBottom: '4px' }}>Relais zero-knowledge</div>
                <div style={{ fontFamily: 'monospace', fontSize: '0.78rem', color: 'rgba(255,255,255,.4)' }}>{data.relay_url}</div>
              </div>
              {data.health.reachable ? (
                <span
                  style={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: '7px',
                    borderRadius: '20px',
                    padding: '6px 16px',
                    fontSize: '0.8rem',
                    fontWeight: 700,
                    background: 'rgba(16,185,129,.15)',
                    color: '#6EE7B7',
                    border: '1px solid rgba(16,185,129,.3)',
                  }}
                >
                  <Wifi size={14} /> En ligne{data.health.version !== '—' ? ` · v${data.health.version}` : ''}
                </span>
              ) : (
                <span
                  style={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: '7px',
                    borderRadius: '20px',
                    padding: '6px 16px',
                    fontSize: '0.8rem',
                    fontWeight: 700,
                    background: 'rgba(148,163,184,.15)',
                    color: '#CBD5E1',
                    border: '1px solid rgba(148,163,184,.3)',
                  }}
                >
                  <WifiOff size={14} /> Hors ligne
                </span>
              )}
            </div>

            {!data.health.reachable && (
              <div
                style={{
                  background: 'rgba(148,163,184,.1)',
                  border: '1px solid rgba(148,163,184,.25)',
                  borderRadius: '8px',
                  padding: '12px 16px',
                  color: '#E2E8F0',
                  fontSize: '0.82rem',
                  marginBottom: '18px',
                }}
              >
                <strong>Relais actuellement injoignable.</strong>{' '}
                {data.health.error ? data.health.error : "Aucune donnée de télémétrie disponible pour l'instant."}
              </div>
            )}

            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '10px', borderTop: '1px solid rgba(255,255,255,.08)', paddingTop: '20px' }}>
              <CharItem label="Hostname" value={data.health.hostname} />
              <CharItem label="Port" value={data.health.port} />
              <CharItem label="Uptime" value={data.health.uptime_fmt} />
              <CharItem label="Blobs (tenants)" value={String(data.health.blob_tenants)} />
              <CharItem label="Stockage" value={data.health.blobs_dir} />
            </div>

            {data.health.networks.length > 0 && (
              <div style={{ marginTop: '20px' }}>
                <div style={{ fontSize: '0.68rem', textTransform: 'uppercase', letterSpacing: '.08em', color: 'rgba(255,255,255,.3)', marginBottom: '10px' }}>
                  Interfaces réseau
                </div>
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                  {data.health.networks.map((net, idx) => (
                    <div
                      key={idx}
                      style={{
                        display: 'inline-flex',
                        alignItems: 'center',
                        gap: '8px',
                        borderRadius: '9px',
                        padding: '8px 14px',
                        border: '1px solid rgba(255,255,255,.12)',
                        background: 'rgba(255,255,255,.05)',
                        fontFamily: 'monospace',
                        fontSize: '0.8rem',
                        color: 'rgba(255,255,255,.7)',
                      }}
                    >
                      {net}
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div
              style={{
                display: 'inline-flex',
                alignItems: 'flex-start',
                gap: '8px',
                background: 'rgba(245,158,11,.08)',
                border: '1px solid rgba(245,158,11,.25)',
                borderRadius: '10px',
                padding: '10px 16px',
                fontSize: '0.78rem',
                color: '#FCD34D',
                marginTop: '20px',
                lineHeight: 1.5,
              }}
            >
              <Lock size={15} style={{ flexShrink: 0, marginTop: '1px' }} />
              <span>
                {data.health.zero_knowledge
                  ? 'Zero-knowledge — le relais stocke des blobs chiffrés opaques. Même saisi ou compromis, il ne peut ni lire la DEK ni déchiffrer les données métier PME.'
                  : "Ce relais n'a pas confirmé son fonctionnement zero-knowledge."}
              </span>
            </div>
          </div>

          {/* Statistiques */}
          <div style={{ display: 'flex', gap: '20px', marginBottom: '28px', flexWrap: 'wrap' }}>
            <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderTop: '3px solid var(--crimson)', borderRadius: 'var(--radius)', padding: '20px', boxShadow: 'var(--shadow)', flex: 1, minWidth: '180px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '8px' }}>
                <Wifi size={14} color="var(--crimson)" /> Nœuds PME connectés
              </div>
              <div style={{ fontSize: '30px', fontWeight: 700, color: 'var(--crimson)' }}>{data.total_nodes}</div>
              <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>annonces reçues au relais</div>
            </div>
            <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderTop: '3px solid var(--gold)', borderRadius: 'var(--radius)', padding: '20px', boxShadow: 'var(--shadow)', flex: 1, minWidth: '180px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '8px' }}>
                <Boxes size={14} color="var(--gold)" /> Tenants déclarés
              </div>
              <div style={{ fontSize: '30px', fontWeight: 700, color: 'var(--gold)' }}>{data.tenants.length}</div>
              <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>clusters enregistrés</div>
            </div>
            <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: 'var(--radius)', padding: '20px', boxShadow: 'var(--shadow)', flex: 1, minWidth: '180px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '8px' }}>
                <Database size={14} /> Blobs stockés
              </div>
              <div style={{ fontSize: '30px', fontWeight: 700, color: 'var(--text-primary)' }}>{data.health.blob_tenants}</div>
              <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>tenants avec données chiffrées</div>
            </div>
          </div>

          {/* Topologie par tenant */}
          {data.tenants.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '48px 20px', color: 'var(--text-muted)', background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: 'var(--radius)' }}>
              <Server size={28} style={{ marginBottom: '8px', opacity: 0.5 }} />
              <p>Aucun tenant n'a encore annoncé de nœud au relais</p>
            </div>
          ) : (
            data.tenants.map((entry) => (
              <div key={entry.tenant.id} style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', marginBottom: '16px', overflow: 'hidden' }}>
                <div style={{ padding: '14px 18px', borderBottom: '1px solid var(--border)', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <span style={{ fontWeight: 700, color: 'var(--text-primary)' }}>{entry.tenant.name}</span>
                    <span style={{ fontFamily: 'monospace', fontSize: '0.72rem', color: 'var(--text-muted)', background: '#F3F4F6', padding: '2px 8px', borderRadius: '4px' }}>
                      {entry.tenant.id}
                    </span>
                  </div>
                  <span
                    style={{
                      background: entry.nodes.length > 0 ? '#D1FAE5' : '#F3F4F6',
                      color: entry.nodes.length > 0 ? '#065F46' : 'var(--text-muted)',
                      borderRadius: '12px',
                      padding: '3px 12px',
                      fontSize: '0.78rem',
                      fontWeight: 600,
                    }}
                  >
                    {entry.nodes.length} nœud{entry.nodes.length !== 1 ? 's' : ''}
                  </span>
                </div>
                {entry.nodes.length === 0 ? (
                  <div style={{ color: 'var(--text-muted)', fontSize: '0.875rem', textAlign: 'center', padding: '20px' }}>
                    Aucun nœud annoncé pour ce tenant.
                  </div>
                ) : (
                  <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                    <thead style={{ background: '#F5EFE8' }}>
                      <tr>
                        <th style={{ padding: '10px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Nœud</th>
                        <th style={{ padding: '10px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>
                          <Clock size={11} style={{ verticalAlign: 'middle', marginRight: '4px' }} />Dernier contact
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      {entry.nodes.map((node) => (
                        <tr key={node.node_id}>
                          <td style={{ padding: '12px 16px', fontFamily: 'monospace', fontSize: '0.82rem' }}>{node.short_id}…</td>
                          <td style={{ padding: '12px 16px', color: 'var(--text-muted)', fontSize: '0.82rem' }}>
                            {optionalString(node.time_ago) ?? formatFallback(node.last_seen)}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            ))
          )}
        </>
      )}
    </div>
  )
}

function formatFallback(iso: string): string {
  if (!iso) return '—'
  return new Date(iso).toLocaleString('fr-FR')
}
