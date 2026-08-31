import { useEffect, useState } from 'react'
import { getTenants, getLicenses, getDevices } from '../api/client'
import type { Tenant, License, Device } from '../types/index'
import StatusBadge from '../components/StatusBadge'
import { GrowthChart, PlanDistributionChart } from '../components/AnalyticsCharts'
import GrafanaEmbed from '../components/GrafanaEmbed'

interface StatCardProps {
  label: string
  value: number | string
  subLabel: string
  accentColor: string
}

function StatCard({ label, value, subLabel, accentColor }: StatCardProps) {
  return (
    <div
      style={{
        background: '#FFFFFF',
        border: '1px solid var(--border)',
        borderTop: `3px solid ${accentColor}`,
        borderRadius: 'var(--radius)',
        padding: '24px',
        boxShadow: 'var(--shadow)',
        flex: 1,
      }}
    >
      <div style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '8px' }}>{label}</div>
      <div style={{ fontSize: '36px', fontWeight: 700, color: accentColor, lineHeight: 1, marginBottom: '6px' }}>{value}</div>
      <div style={{ color: 'var(--text-muted)', fontSize: '12px' }}>{subLabel}</div>
    </div>
  )
}

export default function DashboardPage() {
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [licenses, setLicenses] = useState<License[]>([])
  const [devices, setDevices] = useState<Device[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let cancelled = false

    async function load() {
      try {
        const [tenantsData, licensesData, devicesData] = await Promise.all([
          getTenants(),
          getLicenses(),
          getDevices(),
        ])
        if (!cancelled) {
          setTenants(tenantsData)
          setLicenses(licensesData)
          setDevices(devicesData)
        }
      } catch (err: unknown) {
        console.error('Erreur dashboard:', err)
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    void load()
    return () => {
      cancelled = true
    }
  }, [])

  if (loading) {
    return <p style={{ color: 'var(--text-secondary)' }}>Chargement des données analytiques Éditeur…</p>
  }

  const activeLicenses = licenses.filter((l) => l.is_active).length

  return (
    <div>
      <div style={{ marginBottom: '28px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>
          Console d'Administration Éditeur SaaS
        </h1>
        <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
          Télémétrie, graphiques Grafana et analytiques en temps réel de vos abonnés PME
        </p>
      </div>

      {/* Cartes Métriques Éditeur */}
      <div style={{ display: 'flex', gap: '20px', marginBottom: '28px' }}>
        <StatCard
          label="Entreprises Abonnées (Tenants)"
          value={tenants.length}
          subLabel="PME enregistrées en base de données"
          accentColor="var(--crimson)"
        />
        <StatCard
          label="Licences Cryptographiques"
          value={activeLicenses}
          subLabel="Signées en Ed25519 (Blind Relay)"
          accentColor="var(--gold)"
        />
        <StatCard
          label="Postes PME Enrôlés"
          value={devices.length}
          subLabel="Appairés via Sealed Box X25519"
          accentColor="#2E7D32"
        />
      </div>

      {/* SECTION TÉLÉMÉTRIE & PANNEAUX GRAFANA EMBEDDED */}
      <div style={{ marginBottom: '28px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px' }}>
          <h2 style={{ borderLeft: '3px solid var(--gold)', paddingLeft: '12px', fontSize: '16px', fontWeight: 700, margin: 0, color: 'var(--text-primary)' }}>
            Observabilité Grafana & Télémétrie gRPC en Temps Réel
          </h2>
          <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
            Propulsé par Prometheus & Grafana (Port 3000)
          </span>
        </div>

        <div style={{ display: 'flex', gap: '20px' }}>
          <GrafanaEmbed title="Latence Réseau gRPC (ms)" metricType="GRPC_LATENCY" refreshRateSeconds={10} />
          <GrafanaEmbed title="Disponibilité Quorum etcd / Patroni" metricType="CLUSTER_NODES" refreshRateSeconds={15} />
          <GrafanaEmbed title="Vérifications Licences Ed25519" metricType="LICENSE_VERIFICATIONS" refreshRateSeconds={30} />
        </div>
      </div>

      {/* GRAPHIQUES ANALYTIQUES COMMERCIAUX RÉELS (Liés à la Base de Données) */}
      <div style={{ display: 'flex', gap: '24px', marginBottom: '28px' }}>
        <GrowthChart tenants={tenants} />
        <PlanDistributionChart licenses={licenses} />
      </div>

      {/* Tables des Tenants et des Licences */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '24px' }}>
        {/* Table Tenants */}
        <div
          style={{
            background: '#FFFFFF',
            border: '1px solid var(--border)',
            borderRadius: 'var(--radius)',
            padding: '20px',
            boxShadow: 'var(--shadow)',
          }}
        >
          <h2 style={{ fontSize: '15px', fontWeight: 600, marginBottom: '14px', color: 'var(--text-primary)' }}>
            Dernières PME Enregistrées
          </h2>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
            {tenants.map((tenant) => (
              <div
                key={tenant.id}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '12px',
                  background: '#F8F9FA',
                  borderRadius: '6px',
                }}
              >
                <div>
                  <div style={{ fontSize: '14px', fontWeight: 600, color: 'var(--text-primary)' }}>
                    {tenant.name}
                  </div>
                  <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
                    {tenant.subdomain}.amane.local
                  </div>
                </div>
                <StatusBadge active={tenant.is_active} />
              </div>
            ))}
          </div>
        </div>

        {/* Table Licences Ed25519 */}
        <div
          style={{
            background: '#FFFFFF',
            border: '1px solid var(--border)',
            borderRadius: 'var(--radius)',
            padding: '20px',
            boxShadow: 'var(--shadow)',
          }}
        >
          <h2 style={{ fontSize: '15px', fontWeight: 600, marginBottom: '14px', color: 'var(--text-primary)' }}>
            Licences Cryptographiques Signées
          </h2>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
            {licenses.map((lic) => (
              <div
                key={lic.id}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '12px',
                  background: '#F8F9FA',
                  borderRadius: '6px',
                }}
              >
                <div>
                  <div style={{ fontSize: '13px', fontWeight: 600, color: 'var(--text-primary)' }}>
                    {lic.tenant_name || 'Tenant PME'} — {lic.plan_tier || 'ENTERPRISE'}
                  </div>
                  <div style={{ fontSize: '11px', color: 'var(--text-muted)', fontFamily: 'monospace' }}>
                    Quota : {lic.max_nodes || 3} postes • Jeton Ed25519 Valide
                  </div>
                </div>
                <StatusBadge active={lic.is_active} />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
