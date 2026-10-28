// Console d'administration éditeur — vue d'ensemble.
// Remplace templates/dashboard/index.html. Données réelles uniquement :
// GET /api/dashboard/stats/ (compteurs + tenants récents).

import { useCallback, useEffect, useState, type ComponentType } from 'react'
import { Link } from 'react-router-dom'
import { AlertCircle, AlertTriangle, Building2, Inbox, KeyRound, Loader2, MonitorSmartphone, RotateCw } from 'lucide-react'
import { getDashboardStats, normalizeApiError } from '../api/client'
import type { DashboardStats } from '../types/index'
import StatusBadge from '../components/StatusBadge'

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
      <Loader2 size={18} className="ds-spin" />
      <span>Chargement du tableau de bord...</span>
      <style>{`.ds-spin { animation: ds-spin 0.8s linear infinite; } @keyframes ds-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
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

interface StatCardProps {
  icon: ComponentType<{ size?: number; color?: string }>
  label: string
  value: number
  subLabel: string
  accentColor: string
  background?: string
}

function StatCard({ icon: Icon, label, value, subLabel, accentColor, background }: StatCardProps) {
  return (
    <div
      style={{
        background: background ?? '#FFFFFF',
        border: '1px solid var(--border)',
        borderTop: `3px solid ${accentColor}`,
        borderRadius: 'var(--radius)',
        padding: '22px',
        boxShadow: 'var(--shadow)',
        flex: 1,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '10px' }}>
        <Icon size={15} color={accentColor} />
        {label}
      </div>
      <div style={{ fontSize: '34px', fontWeight: 700, color: accentColor, lineHeight: 1, marginBottom: '6px' }}>{value}</div>
      <div style={{ color: 'var(--text-muted)', fontSize: '12px' }}>{subLabel}</div>
    </div>
  )
}

export default function DashboardPage() {
  const [data, setData] = useState<DashboardStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const stats = await getDashboardStats()
      setData(stats)
    } catch (err) {
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
      <div style={{ marginBottom: '28px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>Tableau de bord</h1>
        <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
          Vue d'ensemble de la plateforme SaaS Souverain
        </p>
      </div>

      {loading && <LoadingState />}
      {!loading && error && <ErrorState message={error} onRetry={() => void load()} />}

      {!loading && !error && data && (
        <>
          <div style={{ display: 'flex', gap: '20px', marginBottom: '28px', flexWrap: 'wrap' }}>
            <StatCard
              icon={Building2}
              label="Tenants enregistrés"
              value={data.stats.total_tenants}
              subLabel="entreprises clientes"
              accentColor="var(--crimson)"
            />
            <StatCard
              icon={KeyRound}
              label="Licences actives"
              value={data.stats.active_licenses}
              subLabel="abonnements en cours"
              accentColor="var(--gold)"
            />
            <StatCard
              icon={MonitorSmartphone}
              label="Machines enregistrées"
              value={data.stats.total_devices}
              subLabel="postes installés"
              accentColor="#2E7D32"
            />
            {data.stats.tenants_over_limit > 0 && (
              <StatCard
                icon={AlertTriangle}
                label="Hors conformité"
                value={data.stats.tenants_over_limit}
                subLabel="tenant(s) dépassant leur licence"
                accentColor="var(--crimson)"
                background="#FFF5F5"
              />
            )}
          </div>

          <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
            <div style={{ padding: '16px 20px', borderBottom: '1px solid var(--border)', fontSize: '15px', fontWeight: 600, color: 'var(--text-primary)' }}>
              Tenants récents
            </div>
            {data.recent_tenants.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '40px 20px', color: 'var(--text-muted)' }}>
                <Inbox size={28} style={{ marginBottom: '8px', opacity: 0.5 }} />
                <p>Aucun tenant enregistré</p>
              </div>
            ) : (
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
                <thead style={{ background: '#F5EFE8' }}>
                  <tr>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Entreprise</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Email</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Employés</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Statut</th>
                  </tr>
                </thead>
                <tbody>
                  {data.recent_tenants.map((tenant) => (
                    <tr key={tenant.id} style={{ borderBottom: '1px solid var(--border)' }}>
                      <td style={{ padding: '14px 16px' }}>
                        <Link to={`/tenants/${tenant.id}`} style={{ color: 'var(--crimson)', fontWeight: 600, textDecoration: 'none' }}>
                          {tenant.name}
                        </Link>
                      </td>
                      <td style={{ padding: '14px 16px', color: 'var(--text-secondary)' }}>{tenant.email}</td>
                      <td style={{ padding: '14px 16px' }}>{tenant.employee_count}</td>
                      <td style={{ padding: '14px 16px' }}><StatusBadge active={tenant.is_active} /></td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </>
      )}
    </div>
  )
}
