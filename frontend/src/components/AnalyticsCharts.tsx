// Graphiques analytiques éditeur — aucune valeur inventée : chaque série est
// dérivée des objets réellement reçus en props (tenants / licenses via l'API).
// Voir CLAUDE.md de frontend/ : « toute série qui ne se déduit pas des données
// de l'API disparaît ».

import type React from 'react'
import { Inbox, TrendingUp } from 'lucide-react'
import type { License, Tenant } from '../types/index'

const cardStyle: React.CSSProperties = {
  background: '#FFFFFF',
  borderRadius: 'var(--radius)',
  border: '1px solid var(--border)',
  padding: '20px',
  boxShadow: 'var(--shadow)',
  flex: 1,
}

// ---------------------------------------------------------------------------
// GrowthChart — nombre cumulé de tenants inscrits, mois par mois, calculé à
// partir des vrais `created_at` reçus en props. Aucune valeur n'est fabriquée :
// si l'API ne renvoie aucun tenant, le graphique affiche un état vide.
// ---------------------------------------------------------------------------

interface GrowthChartProps {
  tenants: Tenant[]
}

const MONTHS_SHOWN = 6

function buildMonthlyGrowth(tenants: Tenant[]): { label: string; count: number }[] {
  const now = new Date()
  const months: Date[] = []
  for (let i = MONTHS_SHOWN - 1; i >= 0; i--) {
    months.push(new Date(now.getFullYear(), now.getMonth() - i, 1))
  }

  return months.map((month) => {
    // Fin de mois exclusive : tout tenant créé avant le 1er du mois suivant compte.
    const cutoff = new Date(month.getFullYear(), month.getMonth() + 1, 1)
    const count = tenants.filter((t) => new Date(t.created_at) < cutoff).length
    const label = month.toLocaleDateString('fr-FR', { month: 'short' }).replace('.', '')
    return { label, count }
  })
}

export function GrowthChart({ tenants }: GrowthChartProps) {
  const chartData = buildMonthlyGrowth(tenants)
  const maxVal = Math.max(...chartData.map((d) => d.count), 1)

  return (
    <div style={cardStyle}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
        <div>
          <h3 style={{ fontSize: '15px', fontWeight: 700, color: 'var(--text-primary)', margin: 0 }}>
            Croissance des PME abonnées (tenants)
          </h3>
          <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
            Nombre cumulé de tenants inscrits, calculé depuis leur date de création
          </span>
        </div>
        <span
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: '4px',
            fontSize: '12px',
            fontWeight: 700,
            color: '#2E7D32',
            background: '#E8F5E9',
            padding: '4px 10px',
            borderRadius: '10px',
          }}
        >
          <TrendingUp size={13} /> {tenants.length} tenant{tenants.length > 1 ? 's' : ''} au total
        </span>
      </div>

      {tenants.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '32px 0', color: 'var(--text-muted)' }}>
          <Inbox size={24} style={{ marginBottom: '8px', opacity: 0.5 }} />
          <p style={{ margin: 0, fontSize: '13px' }}>Aucun tenant enregistré pour le moment.</p>
        </div>
      ) : (
        <div style={{ height: '180px', display: 'flex', alignItems: 'flex-end', gap: '16px', paddingTop: '20px' }}>
          {chartData.map((item) => {
            const heightPercent = (item.count / maxVal) * 100
            return (
              <div
                key={item.label}
                style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', height: '100%', justifyContent: 'flex-end' }}
              >
                <span style={{ fontSize: '12px', fontWeight: 700, color: 'var(--crimson)', marginBottom: '6px' }}>
                  {item.count}
                </span>
                <div
                  style={{
                    width: '100%',
                    height: `${Math.max(heightPercent, item.count > 0 ? 8 : 2)}%`,
                    background: 'linear-gradient(180deg, var(--crimson) 0%, #6E0E16 100%)',
                    borderRadius: '4px 4px 0 0',
                    transition: 'height 0.4s ease-in-out',
                  }}
                />
                <span style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '8px', textTransform: 'capitalize' }}>
                  {item.label}
                </span>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// PlanDistributionChart — répartition réelle des licences par offre (`plan`).
// ---------------------------------------------------------------------------

interface PlanDistributionChartProps {
  licenses: License[]
}

export function PlanDistributionChart({ licenses }: PlanDistributionChartProps) {
  const total = licenses.length || 1

  const enterpriseCount = licenses.filter((l) => l.plan === 'enterprise').length
  const proCount = licenses.filter((l) => l.plan === 'pro').length
  const starterCount = licenses.filter((l) => l.plan === 'starter').length

  const plans = [
    { label: 'Enterprise', count: enterpriseCount, percent: Math.round((enterpriseCount / total) * 100), color: 'var(--crimson)' },
    { label: 'Pro', count: proCount, percent: Math.round((proCount / total) * 100), color: 'var(--gold)' },
    { label: 'Starter', count: starterCount, percent: Math.round((starterCount / total) * 100), color: '#616161' },
  ]

  return (
    <div style={cardStyle}>
      <h3 style={{ fontSize: '15px', fontWeight: 700, color: 'var(--text-primary)', marginBottom: '4px' }}>
        Répartition par offre de licence
      </h3>
      <span style={{ fontSize: '12px', color: 'var(--text-muted)', display: 'block', marginBottom: '20px' }}>
        Calculé depuis les {licenses.length} licence{licenses.length > 1 ? 's' : ''} émise{licenses.length > 1 ? 's' : ''}
      </span>

      {licenses.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '32px 0', color: 'var(--text-muted)' }}>
          <Inbox size={24} style={{ marginBottom: '8px', opacity: 0.5 }} />
          <p style={{ margin: 0, fontSize: '13px' }}>Aucune licence émise pour le moment.</p>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {plans.map((p) => (
            <div key={p.label}>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', fontWeight: 600, marginBottom: '6px' }}>
                <span>{p.label} ({p.count})</span>
                <span style={{ color: p.color }}>{p.percent}%</span>
              </div>
              <div style={{ height: '10px', background: '#EEEEEE', borderRadius: '5px', overflow: 'hidden' }}>
                <div
                  style={{
                    width: `${p.percent}%`,
                    height: '100%',
                    background: p.color,
                    borderRadius: '5px',
                    transition: 'width 0.4s ease-in-out',
                  }}
                />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
