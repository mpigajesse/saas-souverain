import type React from 'react'
import { useEffect, useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { getTenant, getLicenses, getDevices } from '../api/client'
import StatusBadge from '../components/StatusBadge'
import type { Tenant, License, Device } from '../types/index'

function truncate(str: string, max: number): string {
  return str.length > max ? str.slice(0, max) + '…' : str
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('fr-FR')
}

function getInitials(name: string): string {
  return name.slice(0, 2).toUpperCase()
}

type LicensePlan = 'starter' | 'pro' | 'enterprise'

function getPlanBadgeStyle(plan: LicensePlan): React.CSSProperties {
  if (plan === 'pro') {
    return {
      background: '#FFF3CD',
      color: '#856404',
      padding: '2px 10px',
      borderRadius: '12px',
      fontSize: '12px',
      fontWeight: 600,
      display: 'inline-block',
    }
  }
  if (plan === 'enterprise') {
    return {
      background: '#F8E8E8',
      color: 'var(--crimson)',
      padding: '2px 10px',
      borderRadius: '12px',
      fontSize: '12px',
      fontWeight: 600,
      display: 'inline-block',
    }
  }
  return {
    background: '#F0ECE8',
    color: 'var(--text-muted)',
    padding: '2px 10px',
    borderRadius: '12px',
    fontSize: '12px',
    fontWeight: 600,
    display: 'inline-block',
  }
}

export default function TenantDetail() {
  const { id } = useParams<{ id: string }>()
  const [tenant, setTenant] = useState<Tenant | null>(null)
  const [licenses, setLicenses] = useState<License[]>([])
  const [devices, setDevices] = useState<Device[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!id) return
    let cancelled = false

    async function load() {
      try {
        const [tenantData, licensesData, devicesData] = await Promise.all([
          getTenant(id!),
          getLicenses(id),
          getDevices(id),
        ])
        if (!cancelled) {
          setTenant(tenantData)
          setLicenses(licensesData)
          setDevices(devicesData)
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
  }, [id])

  const breadcrumbStyle: React.CSSProperties = {
    marginBottom: '24px',
    fontSize: '14px',
    color: 'var(--text-secondary)',
  }

  const breadcrumbLinkStyle: React.CSSProperties = {
    color: 'var(--crimson)',
    fontWeight: 500,
  }

  const infoCardStyle: React.CSSProperties = {
    background: '#FFFFFF',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: '24px',
    boxShadow: 'var(--shadow)',
    marginBottom: '28px',
  }

  const avatarStyle: React.CSSProperties = {
    width: '48px',
    height: '48px',
    borderRadius: '50%',
    background: 'var(--crimson)',
    color: '#FFFFFF',
    fontSize: '20px',
    fontWeight: 700,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
  }

  const tenantNameStyle: React.CSSProperties = {
    fontSize: '20px',
    fontWeight: 700,
    color: 'var(--text-primary)',
  }

  const infoGridStyle: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '16px',
    marginTop: '20px',
  }

  const infoLabelStyle: React.CSSProperties = {
    color: 'var(--text-secondary)',
    fontSize: '12px',
    marginBottom: '2px',
  }

  const infoValueStyle: React.CSSProperties = {
    color: 'var(--text-primary)',
    fontWeight: 500,
  }

  const sectionTitleStyle: React.CSSProperties = {
    borderLeft: '3px solid var(--crimson)',
    paddingLeft: '12px',
    fontSize: '16px',
    fontWeight: 600,
    color: 'var(--text-primary)',
    marginBottom: '16px',
  }

  const sectionStyle: React.CSSProperties = {
    marginBottom: '28px',
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

  const tdStyle: React.CSSProperties = {
    padding: '14px 16px',
    borderBottom: '1px solid var(--border)',
    verticalAlign: 'middle',
    color: 'var(--text-primary)',
  }

  const monoTdStyle: React.CSSProperties = {
    ...tdStyle,
    fontFamily: 'monospace',
    fontSize: '12px',
    color: 'var(--text-secondary)',
  }

  const emptyTdStyle: React.CSSProperties = {
    ...tdStyle,
    textAlign: 'center',
    color: 'var(--text-muted)',
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

  if (!tenant) {
    return <div style={errorStyle}>Tenant introuvable.</div>
  }

  return (
    <div>
      <div style={breadcrumbStyle}>
        <Link to="/tenants" style={breadcrumbLinkStyle}>← Tenants</Link>
        <span> / {tenant.name}</span>
      </div>

      <div style={infoCardStyle}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '8px' }}>
          <div style={avatarStyle}>{getInitials(tenant.name)}</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <span style={tenantNameStyle}>{tenant.name}</span>
            <StatusBadge active={tenant.is_active} />
          </div>
        </div>

        <div style={infoGridStyle}>
          <div>
            <div style={infoLabelStyle}>Email</div>
            <div style={infoValueStyle}>{tenant.email}</div>
          </div>
          <div>
            <div style={infoLabelStyle}>Téléphone</div>
            <div style={infoValueStyle}>{tenant.phone}</div>
          </div>
          <div>
            <div style={infoLabelStyle}>Adresse</div>
            <div style={infoValueStyle}>{tenant.address}</div>
          </div>
          <div>
            <div style={infoLabelStyle}>Employés</div>
            <div style={infoValueStyle}>{tenant.employee_count}</div>
          </div>
        </div>
      </div>

      <div style={sectionStyle}>
        <h2 style={sectionTitleStyle}>Licences</h2>
        <div style={tableWrapperStyle}>
          <table style={tableStyle}>
            <thead style={theadStyle}>
              <tr>
                <th style={thStyle}>Plan</th>
                <th style={thStyle}>Sièges</th>
                <th style={thStyle}>Date début</th>
                <th style={thStyle}>Date fin</th>
                <th style={thStyle}>Statut</th>
              </tr>
            </thead>
            <tbody>
              {licenses.map((lic) => (
                <tr key={lic.id}>
                  <td style={tdStyle}>
                    <span style={getPlanBadgeStyle(lic.plan)}>{lic.plan}</span>
                  </td>
                  <td style={tdStyle}>{lic.seats}</td>
                  <td style={tdStyle}>{formatDate(lic.starts_at)}</td>
                  <td style={tdStyle}>{lic.expires_at ? formatDate(lic.expires_at) : '∞'}</td>
                  <td style={tdStyle}><StatusBadge active={lic.is_active} /></td>
                </tr>
              ))}
              {licenses.length === 0 && (
                <tr>
                  <td colSpan={5} style={emptyTdStyle}>Aucune licence.</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      <div style={sectionStyle}>
        <h2 style={sectionTitleStyle}>Machines</h2>
        <div style={tableWrapperStyle}>
          <table style={tableStyle}>
            <thead style={theadStyle}>
              <tr>
                <th style={thStyle}>Hostname</th>
                <th style={thStyle}>OS</th>
                <th style={thStyle}>Installation ID</th>
                <th style={thStyle}>Dernière connexion</th>
                <th style={thStyle}>Licence</th>
              </tr>
            </thead>
            <tbody>
              {devices.map((dev) => (
                <tr key={dev.id}>
                  <td style={tdStyle}>{dev.hostname}</td>
                  <td style={tdStyle}>{dev.os}</td>
                  <td style={monoTdStyle}>{truncate(dev.installation_id, 8)}…</td>
                  <td style={tdStyle}>{formatDate(dev.last_seen)}</td>
                  <td style={tdStyle}>
                    <StatusBadge
                      active={dev.is_within_license}
                      label={dev.is_within_license ? 'OK' : 'Hors licence'}
                    />
                  </td>
                </tr>
              ))}
              {devices.length === 0 && (
                <tr>
                  <td colSpan={5} style={emptyTdStyle}>Aucune machine enregistrée.</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
