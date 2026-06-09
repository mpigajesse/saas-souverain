import type React from 'react'
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { getTenants } from '../api/client'
import StatusBadge from '../components/StatusBadge'
import type { Tenant } from '../types/index'

export default function TenantsPage() {
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [hoveredRow, setHoveredRow] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false

    async function load() {
      try {
        const data = await getTenants()
        if (!cancelled) setTenants(data)
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

  const headerStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '24px',
  }

  const titleStyle: React.CSSProperties = {
    fontSize: '22px',
    fontWeight: 700,
    color: 'var(--text-primary)',
  }

  const buttonStyle: React.CSSProperties = {
    background: 'var(--crimson)',
    color: '#FFFFFF',
    border: 'none',
    padding: '10px 18px',
    borderRadius: 'var(--radius)',
    fontSize: '13px',
    fontWeight: 600,
    cursor: 'pointer',
  }

  const tableWrapperStyle: React.CSSProperties = {
    background: '#FFFFFF',
    borderRadius: 'var(--radius)',
    boxShadow: 'var(--shadow)',
    overflow: 'hidden',
  }

  const tableStyle: React.CSSProperties = {
    width: '100%',
    borderCollapse: 'collapse',
  }

  const theadStyle: React.CSSProperties = {
    background: '#F5EFE8',
  }

  const thStyle: React.CSSProperties = {
    padding: '12px 16px',
    textAlign: 'left',
    color: 'var(--text-secondary)',
    fontSize: '11px',
    fontWeight: 600,
    textTransform: 'uppercase',
    letterSpacing: '0.8px',
    borderBottom: '2px solid var(--gold)',
  }

  const thCenterStyle: React.CSSProperties = {
    ...thStyle,
    textAlign: 'center',
  }

  const tdStyle: React.CSSProperties = {
    padding: '14px 16px',
    borderBottom: '1px solid var(--border)',
    verticalAlign: 'middle',
  }

  const tdCenterStyle: React.CSSProperties = {
    ...tdStyle,
    textAlign: 'center',
  }

  const companyNameStyle: React.CSSProperties = {
    fontWeight: 600,
    color: 'var(--text-primary)',
  }

  const companyEmailStyle: React.CSSProperties = {
    color: 'var(--text-secondary)',
    fontSize: '12px',
    marginTop: '2px',
  }

  const actionLinkStyle: React.CSSProperties = {
    color: 'var(--crimson)',
    fontWeight: 600,
    textDecoration: 'none',
  }

  const errorStyle: React.CSSProperties = {
    background: '#FEE8E8',
    border: '1px solid var(--crimson)',
    padding: '12px',
    borderRadius: 'var(--radius)',
    color: 'var(--crimson)',
  }

  const emptyTdStyle: React.CSSProperties = {
    ...tdStyle,
    textAlign: 'center',
    color: 'var(--text-muted)',
  }

  if (loading) {
    return <p style={{ color: 'var(--text-secondary)' }}>Chargement…</p>
  }

  if (error) {
    return <div style={errorStyle}>Erreur : {error}</div>
  }

  return (
    <div>
      <div style={headerStyle}>
        <h1 style={titleStyle}>Tenants</h1>
        <button style={buttonStyle} type="button">
          ＋ Nouveau tenant
        </button>
      </div>

      <div style={tableWrapperStyle}>
        <table style={tableStyle}>
          <thead style={theadStyle}>
            <tr>
              <th style={thStyle}>Entreprise</th>
              <th style={thCenterStyle}>Employés</th>
              <th style={thStyle}>Statut</th>
              <th style={thStyle}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {tenants.map((t) => (
              <tr
                key={t.id}
                style={{ background: hoveredRow === t.id ? '#FFF8F4' : 'transparent' }}
                onMouseEnter={() => setHoveredRow(t.id)}
                onMouseLeave={() => setHoveredRow(null)}
              >
                <td style={tdStyle}>
                  <div style={companyNameStyle}>{t.name}</div>
                  <div style={companyEmailStyle}>{t.email}</div>
                </td>
                <td style={tdCenterStyle}>{t.employee_count}</td>
                <td style={tdStyle}>
                  <StatusBadge active={t.is_active} />
                </td>
                <td style={tdStyle}>
                  <Link to={`/tenants/${t.id}`} style={actionLinkStyle}>
                    Voir →
                  </Link>
                </td>
              </tr>
            ))}
            {tenants.length === 0 && (
              <tr>
                <td colSpan={4} style={emptyTdStyle}>
                  Aucun tenant trouvé.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  )
}
