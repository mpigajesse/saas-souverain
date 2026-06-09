import type React from 'react'
import { useEffect, useState } from 'react'
import { getTenants, getLicenses } from '../api/client'
import type { Tenant, License } from '../types/index'

interface StatCardProps {
  label: string
  value: number | string
  subLabel: string
  accentColor: string
}

function StatCard({ label, value, subLabel, accentColor }: StatCardProps) {
  const cardStyle: React.CSSProperties = {
    background: '#FFFFFF',
    border: '1px solid var(--border)',
    borderTop: `3px solid ${accentColor}`,
    borderRadius: 'var(--radius)',
    padding: '24px',
    boxShadow: 'var(--shadow)',
    flex: 1,
  }

  const labelStyle: React.CSSProperties = {
    color: 'var(--text-secondary)',
    fontSize: '13px',
    marginBottom: '8px',
  }

  const valueStyle: React.CSSProperties = {
    fontSize: '36px',
    fontWeight: 700,
    color: accentColor,
    lineHeight: 1,
    marginBottom: '6px',
  }

  const subLabelStyle: React.CSSProperties = {
    color: 'var(--text-muted)',
    fontSize: '12px',
  }

  return (
    <div style={cardStyle}>
      <div style={labelStyle}>{label}</div>
      <div style={valueStyle}>{value}</div>
      <div style={subLabelStyle}>{subLabel}</div>
    </div>
  )
}

export default function DashboardPage() {
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [licenses, setLicenses] = useState<License[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false

    async function load() {
      try {
        const [tenantsData, licensesData] = await Promise.all([
          getTenants(),
          getLicenses(),
        ])
        if (!cancelled) {
          setTenants(tenantsData)
          setLicenses(licensesData)
        }
      } catch (err: unknown) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Erreur de chargement')
        }
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    void load()
    return () => { cancelled = true }
  }, [])

  const pageHeaderStyle: React.CSSProperties = {
    marginBottom: '28px',
  }

  const titleStyle: React.CSSProperties = {
    fontSize: '22px',
    fontWeight: 700,
    color: 'var(--text-primary)',
  }

  const subtitleStyle: React.CSSProperties = {
    color: 'var(--text-secondary)',
    marginTop: '4px',
  }

  const cardsStyle: React.CSSProperties = {
    display: 'flex',
    gap: '20px',
  }

  const errorStyle: React.CSSProperties = {
    background: '#FEE8E8',
    border: '1px solid var(--crimson)',
    padding: '12px',
    borderRadius: 'var(--radius)',
    color: 'var(--crimson)',
  }

  if (loading) {
    return <p style={{ color: 'var(--text-secondary)' }}>Chargement…</p>
  }

  if (error) {
    return <div style={errorStyle}>Erreur : {error}</div>
  }

  const activeLicenses = licenses.filter((l) => l.is_active).length

  return (
    <div>
      <div style={pageHeaderStyle}>
        <h1 style={titleStyle}>Tableau de bord</h1>
        <p style={subtitleStyle}>Vue d'ensemble de la plateforme</p>
      </div>

      <div style={cardsStyle}>
        <StatCard
          label="Tenants enregistrés"
          value={tenants.length}
          subLabel="entreprises clientes"
          accentColor="var(--crimson)"
        />
        <StatCard
          label="Licences actives"
          value={activeLicenses}
          subLabel="licences en cours"
          accentColor="var(--gold)"
        />
      </div>
    </div>
  )
}
