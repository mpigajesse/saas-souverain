import type React from 'react'
import { useEffect, useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { getTenant, getLicenses, getDevices } from '../api/client'
import StatusBadge from '../components/StatusBadge'
import type { Tenant, License, Device } from '../types/index'

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('fr-FR')
}

function getInitials(name: string): string {
  return name.slice(0, 2).toUpperCase()
}

export default function TenantDetail() {
  const { id } = useParams<{ id: string }>()
  const [tenant, setTenant] = useState<Tenant | null>(null)
  const [licenses, setLicenses] = useState<License[]>([])
  const [devices, setDevices] = useState<Device[]>([])
  const [loading, setLoading] = useState(true)
  
  // Modal QR Code V2
  const [showPairingModal, setShowPairingModal] = useState(false)
  const [pairingTimer, setPairingTimer] = useState(300)

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
        console.error(err)
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    void load()
    return () => {
      cancelled = true
    }
  }, [id])

  useEffect(() => {
    let interval: NodeJS.Timeout
    if (showPairingModal && pairingTimer > 0) {
      interval = setInterval(() => {
        setPairingTimer((prev) => prev - 1)
      }, 1000)
    }
    return () => clearInterval(interval)
  }, [showPairingModal, pairingTimer])

  const formatTimer = (sec: number) => {
    const mins = Math.floor(sec / 60)
    const secs = sec % 60
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
  }

  // Impression synthétique du dossier client PME
  const handlePrintContract = () => {
    window.print()
  }

  if (loading) return <p style={{ color: 'var(--text-secondary)' }}>Chargement du dossier client...</p>
  if (!tenant) return <div style={{ color: 'var(--crimson)' }}>Tenant introuvable.</div>

  return (
    <div>
      {/* Bandeau Supérieur & Bouton Impression */}
      <div className="no-print" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '24px' }}>
        <div style={{ fontSize: '14px', color: 'var(--text-secondary)' }}>
          <Link to="/tenants" style={{ color: 'var(--crimson)', fontWeight: 500, textDecoration: 'none' }}>← Tenants</Link>
          <span> / {tenant.name}</span>
        </div>
        <button
          onClick={handlePrintContract}
          className="no-print"
          style={{
            background: '#FFFFFF',
            border: '1px solid var(--border)',
            padding: '8px 14px',
            borderRadius: '6px',
            fontWeight: 600,
            fontSize: '13px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
          }}
        >
          🖨️ Imprimer la Fiche Client
        </button>
      </div>

      {/* Fiche Entreprise */}
      <div
        style={{
          background: '#FFFFFF',
          border: '1px solid var(--border)',
          borderRadius: 'var(--radius)',
          padding: '24px',
          boxShadow: 'var(--shadow)',
          marginBottom: '28px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '8px' }}>
          <div
            style={{
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
            }}
          >
            {getInitials(tenant.name)}
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <span style={{ fontSize: '20px', fontWeight: 700, color: 'var(--text-primary)' }}>{tenant.name}</span>
            <StatusBadge active={tenant.is_active} />
          </div>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '16px', marginTop: '20px' }}>
          <div>
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Sous-domaine Métier</div>
            <div style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{tenant.subdomain}.amane.local</div>
          </div>
          <div>
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Identifiant Réseau PME</div>
            <div style={{ color: 'var(--text-primary)', fontFamily: 'monospace', fontWeight: 600 }}>ID-{tenant.id}</div>
          </div>
          <div>
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Date d'Abonnement</div>
            <div style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{formatDate(tenant.created_at)}</div>
          </div>
        </div>
      </div>

      {/* Graphiques Télémétrie & Activité Spécifique du Tenant PME */}
      <TenantActivityCharts tenantName={tenant.name} />

      {/* Licences */}
      <div style={{ marginBottom: '28px' }}>
        <h2 style={{ borderLeft: '3px solid var(--crimson)', paddingLeft: '12px', fontSize: '16px', fontWeight: 600, marginBottom: '16px' }}>
          Licences Cryptographiques Ed25519
        </h2>
        <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead style={{ background: '#F5EFE8' }}>
              <tr>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Offre</th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Postes Max</th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Date Expiration</th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Statut</th>
              </tr>
            </thead>
            <tbody>
              {licenses.map((lic) => (
                <tr key={lic.id}>
                  <td style={{ padding: '14px 16px', fontWeight: 600 }}>{lic.plan_tier || 'ENTERPRISE'}</td>
                  <td style={{ padding: '14px 16px' }}>{lic.max_nodes || 3} postes</td>
                  <td style={{ padding: '14px 16px' }}>{formatDate(lic.expires_at)}</td>
                  <td style={{ padding: '14px 16px' }}><StatusBadge active={lic.is_active} /></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Machines / Nœuds du Cluster */}
      <div style={{ marginBottom: '28px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
          <h2 style={{ borderLeft: '3px solid var(--crimson)', paddingLeft: '12px', fontSize: '16px', fontWeight: 600, margin: 0 }}>
            Parc de Machines Enrôlées
          </h2>
          <button
            onClick={() => {
              setPairingTimer(300)
              setShowPairingModal(true)
            }}
            className="no-print"
            style={{
              background: 'var(--crimson)',
              color: '#FFFFFF',
              border: 'none',
              borderRadius: '6px',
              padding: '8px 14px',
              fontWeight: 600,
              fontSize: '13px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              boxShadow: 'var(--shadow)',
            }}
          >
            📱 Appairer un nouveau poste (QR Code V2)
          </button>
        </div>

        <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead style={{ background: '#F5EFE8' }}>
              <tr>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Hostname</th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Rôle</th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>IP VPN Mesh</th>
                <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Dernière vue</th>
              </tr>
            </thead>
            <tbody>
              {devices.map((dev) => (
                <tr key={dev.id}>
                  <td style={{ padding: '14px 16px', fontWeight: 600 }}>{dev.hostname}</td>
                  <td style={{ padding: '14px 16px' }}>
                    <span
                      style={{
                        fontSize: '11px',
                        fontWeight: 700,
                        padding: '2px 8px',
                        borderRadius: '10px',
                        background: dev.node_role === 'ACTIVE' ? '#E8F5E9' : '#EEEEEE',
                        color: dev.node_role === 'ACTIVE' ? '#2E7D32' : '#616161',
                      }}
                    >
                      {dev.node_role}
                    </span>
                  </td>
                  <td style={{ padding: '14px 16px', fontFamily: 'monospace' }}>{dev.ip_address}</td>
                  <td style={{ padding: '14px 16px', fontSize: '12px', color: 'var(--text-muted)' }}>{formatDate(dev.last_seen)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Modal QR Code d'Appairage V2 */}
      {showPairingModal && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(0,0,0,0.6)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
        >
          <div
            style={{
              background: '#FFFFFF',
              borderRadius: '12px',
              padding: '28px',
              maxWidth: '440px',
              width: '100%',
              boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
              textAlign: 'center',
            }}
          >
            <h3 style={{ fontSize: '18px', fontWeight: 700, color: 'var(--text-primary)', marginBottom: '8px' }}>
              Appairage QR Code V2 (Sealed Box X25519)
            </h3>
            <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '20px' }}>
              Faites scanner ce QR Code sur le 2ème PC pour transmettre la clé d'accès (AK) de manière 100% chiffrée.
            </p>

            <div
              style={{
                background: '#F8F9FA',
                padding: '20px',
                borderRadius: '8px',
                display: 'inline-block',
                border: '2px dashed var(--gold)',
                marginBottom: '16px',
              }}
            >
              <div style={{ fontSize: '72px', lineHeight: 1 }}>📱🔳</div>
              <div style={{ fontFamily: 'monospace', fontSize: '11px', color: 'var(--text-muted)', marginTop: '8px' }}>
                INV-8f92a10c • Sealed Box X25519
              </div>
            </div>

            <div
              style={{
                background: pairingTimer < 60 ? '#FEE8E8' : '#FFF3CD',
                color: pairingTimer < 60 ? 'var(--crimson)' : '#856404',
                padding: '10px',
                borderRadius: '6px',
                fontWeight: 700,
                fontSize: '14px',
                marginBottom: '20px',
              }}
            >
              ⏱️ Temps restant (Usage unique) : {formatTimer(pairingTimer)}
            </div>

            <button
              onClick={() => setShowPairingModal(false)}
              style={{
                width: '100%',
                padding: '10px',
                background: 'var(--sidebar-bg)',
                color: '#FFFFFF',
                border: 'none',
                borderRadius: '6px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Fermer la fenêtre d'appairage
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
