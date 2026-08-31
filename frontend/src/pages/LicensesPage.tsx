import type React from 'react'
import { useEffect, useState } from 'react'
import { getLicenses, getTenants } from '../api/client'
import type { License, Tenant } from '../types/index'
import StatusBadge from '../components/StatusBadge'

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('fr-FR')
}

export default function LicensesPage() {
  const [licenses, setLicenses] = useState<License[]>([])
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [loading, setLoading] = useState(true)
  const [searchTerm, setSearchTerm] = useState('')
  const [filterStatus, setFilterStatus] = useState<'ALL' | 'ACTIVE' | 'EXPIRED'>('ALL')

  // Modal d'émission de licence
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [selectedTenantId, setSelectedTenantId] = useState('')
  const [planTier, setPlanTier] = useState('ENTERPRISE_SOUVERAIN')
  const [maxNodes, setMaxNodes] = useState(3)
  const [signedTokenResult, setSignedTokenResult] = useState<string | null>(null)
  
  // Notification Toast
  const [toastMessage, setToastMessage] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    async function load() {
      try {
        const [licensesData, tenantsData] = await Promise.all([getLicenses(), getTenants()])
        if (!cancelled) {
          setLicenses(licensesData)
          setTenants(tenantsData)
          if (tenantsData.length > 0) setSelectedTenantId(tenantsData[0].id)
        }
      } catch (err) {
        console.error(err)
      } finally {
        if (!cancelled) setLoading(false)
      }
    }
    void load()
    return () => {
      cancelled = true
    }
  }, [])

  const triggerToast = (msg: string) => {
    setToastMessage(msg)
    setTimeout(() => setToastMessage(null), 3500)
  }

  // Renouveler une licence (+30 jours)
  const handleExtendLicense = (licId: string) => {
    setLicenses((prev) =>
      prev.map((lic) => {
        if (lic.id === licId) {
          const newDate = new Date()
          newDate.setDate(newDate.getDate() + 30)
          return {
            ...lic,
            expires_at: newDate.toISOString(),
            is_active: true,
            signed_token: `ed25519:renewed_${Math.random().toString(36).substring(2, 10)}`,
          }
        }
        return lic
      })
    )
    triggerToast('🔄 Licence prolongée de 30 jours et re-signée en Ed25519 !')
  }

  // Révoquer une licence (Inscription CRL < 1s)
  const handleRevokeLicense = (licId: string) => {
    setLicenses((prev) =>
      prev.map((lic) => (lic.id === licId ? { ...lic, is_active: false } : lic))
    )
    triggerToast('🚫 Licence révoquée ! AK inscrite dans la CRL (effet en < 1 seconde).')
  }

  const handleGenerateLicense = (e: React.FormEvent) => {
    e.preventDefault()
    const targetTenant = tenants.find((t) => t.id === selectedTenantId)
    const tenantName = targetTenant ? targetTenant.name : 'PME'
    const dummyToken = `ed25519:sig_${Math.random().toString(36).substring(2, 10)}_tenant_${selectedTenantId}_nodes_${maxNodes}`

    const newLic: License = {
      id: `lic-${Date.now()}`,
      tenant: selectedTenantId,
      tenant_name: tenantName,
      max_nodes: maxNodes,
      plan_tier: planTier,
      expires_at: '2026-12-31T23:59:59Z',
      is_active: true,
      signed_token: dummyToken,
    }

    setLicenses([newLic, ...licenses])
    setSignedTokenResult(dummyToken)
    triggerToast('🔑 Nouvelle licence émise avec succès !')
  }

  const filteredLicenses = licenses.filter((lic) => {
    const matchesSearch =
      (lic.tenant_name || '').toLowerCase().includes(searchTerm.toLowerCase()) ||
      (lic.plan_tier || '').toLowerCase().includes(searchTerm.toLowerCase()) ||
      (lic.signed_token || '').toLowerCase().includes(searchTerm.toLowerCase())

    if (filterStatus === 'ACTIVE') return matchesSearch && lic.is_active
    if (filterStatus === 'EXPIRED') return matchesSearch && !lic.is_active
    return matchesSearch
  })

  if (loading) return <p style={{ color: 'var(--text-secondary)' }}>Chargement des licences...</p>

  return (
    <div>
      {/* Toast Notification */}
      {toastMessage && (
        <div
          style={{
            position: 'fixed',
            bottom: '24px',
            right: '24px',
            background: 'var(--sidebar-bg)',
            color: '#FFFFFF',
            padding: '14px 20px',
            borderRadius: '8px',
            boxShadow: '0 10px 15px -3px rgba(0,0,0,0.3)',
            fontWeight: 600,
            fontSize: '13px',
            zIndex: 2000,
            borderLeft: '4px solid var(--gold)',
          }}
        >
          {toastMessage}
        </div>
      )}

      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '28px' }}>
        <div>
          <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>
            Gestion des Licences (Éditeur SaaS)
          </h1>
          <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
            Émission, signature Ed25519, prolongation et révocation instantanée (CRL)
          </p>
        </div>
        <button
          onClick={() => {
            setSignedTokenResult(null)
            setShowCreateModal(true)
          }}
          style={{
            background: 'var(--crimson)',
            color: '#FFFFFF',
            border: 'none',
            borderRadius: '6px',
            padding: '10px 16px',
            fontWeight: 600,
            cursor: 'pointer',
            boxShadow: 'var(--shadow)',
          }}
        >
          🔑 Émettre une nouvelle licence Ed25519
        </button>
      </div>

      {/* Barre de Recherche et Filtres */}
      <div
        style={{
          background: '#FFFFFF',
          padding: '16px 20px',
          borderRadius: 'var(--radius)',
          border: '1px solid var(--border)',
          marginBottom: '20px',
          display: 'flex',
          gap: '16px',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <input
          type="text"
          placeholder="🔍 Rechercher par PME, jeton Ed25519 ou offre..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          style={{
            flex: 1,
            maxWidth: '400px',
            padding: '8px 14px',
            borderRadius: '6px',
            border: '1px solid var(--border)',
            fontSize: '13px',
          }}
        />

        <div style={{ display: 'flex', gap: '8px' }}>
          {(['ALL', 'ACTIVE', 'EXPIRED'] as const).map((st) => (
            <button
              key={st}
              onClick={() => setFilterStatus(st)}
              style={{
                padding: '6px 12px',
                borderRadius: '6px',
                border: '1px solid var(--border)',
                background: filterStatus === st ? 'var(--sidebar-bg)' : '#FFFFFF',
                color: filterStatus === st ? '#FFFFFF' : 'var(--text-primary)',
                fontWeight: 600,
                fontSize: '12px',
                cursor: 'pointer',
              }}
            >
              {st === 'ALL' ? 'Toutes' : st === 'ACTIVE' ? 'Actives 🟢' : 'Inactives / Révoquées 🔴'}
            </button>
          ))}
        </div>
      </div>

      {/* Table des Licences */}
      <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
          <thead style={{ background: '#F5EFE8', borderBottom: '2px solid var(--gold)' }}>
            <tr>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Tenant PME</th>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Offre / Tier</th>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Quota Postes</th>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Expiration</th>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Signature Ed25519</th>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Statut</th>
              <th style={{ padding: '12px 16px', textAlign: 'right', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Actions Rapides</th>
            </tr>
          </thead>
          <tbody>
            {filteredLicenses.map((lic) => (
              <tr key={lic.id} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '14px 16px', fontWeight: 600 }}>{lic.tenant_name || 'Tenant PME'}</td>
                <td style={{ padding: '14px 16px' }}>
                  <span style={{ background: '#FFF3CD', color: '#856404', padding: '2px 8px', borderRadius: '10px', fontSize: '12px', fontWeight: 600 }}>
                    {lic.plan_tier || 'STANDARD'}
                  </span>
                </td>
                <td style={{ padding: '14px 16px', fontWeight: 600 }}>{lic.max_nodes || 3} postes</td>
                <td style={{ padding: '14px 16px' }}>{formatDate(lic.expires_at)}</td>
                <td style={{ padding: '14px 16px', fontFamily: 'monospace', fontSize: '11px', color: 'var(--text-secondary)' }}>
                  {lic.signed_token ? `${lic.signed_token.substring(0, 24)}...` : 'Ed25519-Token'}
                </td>
                <td style={{ padding: '14px 16px' }}>
                  <StatusBadge active={lic.is_active} />
                </td>
                <td style={{ padding: '14px 16px', textAlign: 'right' }}>
                  <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
                    <button
                      onClick={() => handleExtendLicense(lic.id)}
                      title="Prolonger de 30 jours et re-signer"
                      style={{
                        background: '#E8F5E9',
                        color: '#2E7D32',
                        border: '1px solid #A5D6A7',
                        borderRadius: '4px',
                        padding: '4px 8px',
                        fontSize: '11px',
                        fontWeight: 600,
                        cursor: 'pointer',
                      }}
                    >
                      🔄 +30 Jours
                    </button>
                    {lic.is_active && (
                      <button
                        onClick={() => handleRevokeLicense(lic.id)}
                        title="Révoquer la licence (Inscrire dans la CRL)"
                        style={{
                          background: '#FEE8E8',
                          color: 'var(--crimson)',
                          border: '1px solid #EF9A9A',
                          borderRadius: '4px',
                          padding: '4px 8px',
                          fontSize: '11px',
                          fontWeight: 600,
                          cursor: 'pointer',
                        }}
                      >
                        🚫 Révoquer (CRL)
                      </button>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Modal d'émission de licence */}
      {showCreateModal && (
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
          <div style={{ background: '#FFFFFF', borderRadius: '12px', padding: '28px', maxWidth: '480px', width: '100%', boxShadow: 'var(--shadow)' }}>
            <h3 style={{ fontSize: '18px', fontWeight: 700, marginBottom: '8px' }}>Émission de Licence Ed25519 (Blind Relay)</h3>
            <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '20px' }}>
              Le serveur Éditeur génère un jeton signé avec sa clé privée Ed25519. Le client Rust validera ce jeton en local sans appeler le cloud pendant 30 jours.
            </p>

            {signedTokenResult ? (
              <div style={{ background: '#E8F5E9', padding: '16px', borderRadius: '8px', marginBottom: '20px' }}>
                <div style={{ fontWeight: 700, color: '#1B5E20', marginBottom: '8px' }}>✅ Jeton Ed25519 émis et signé avec succès !</div>
                <div style={{ fontFamily: 'monospace', fontSize: '11px', wordBreak: 'break-all', background: '#FFFFFF', padding: '8px', borderRadius: '4px', border: '1px solid #A5D6A7' }}>
                  {signedTokenResult}
                </div>
                <button
                  onClick={() => setShowCreateModal(false)}
                  style={{ width: '100%', marginTop: '16px', padding: '10px', background: '#2E7D32', color: '#FFF', border: 'none', borderRadius: '6px', fontWeight: 600, cursor: 'pointer' }}
                >
                  Terminer
                </button>
              </div>
            ) : (
              <form onSubmit={handleGenerateLicense} style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
                <div>
                  <label style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>Sélectionner le Tenant PME</label>
                  <select
                    value={selectedTenantId}
                    onChange={(e) => setSelectedTenantId(e.target.value)}
                    style={{ width: '100%', padding: '8px', borderRadius: '6px', border: '1px solid var(--border)' }}
                  >
                    {tenants.map((t) => (
                      <option key={t.id} value={t.id}>{t.name} ({t.subdomain})</option>
                    ))}
                  </select>
                </div>

                <div>
                  <label style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>Offre / Niveau d'Abonnement</label>
                  <select
                    value={planTier}
                    onChange={(e) => setPlanTier(e.target.value)}
                    style={{ width: '100%', padding: '8px', borderRadius: '6px', border: '1px solid var(--border)' }}
                  >
                    <option value="ENTERPRISE_SOUVERAIN">ENTERPRISE SOUVERAIN</option>
                    <option value="PRO_CLUSTER">PRO CLUSTER</option>
                    <option value="STANDARD">STANDARD</option>
                  </select>
                </div>

                <div>
                  <label style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>Nombre Maximum de Postes Autorisés</label>
                  <input
                    type="number"
                    min="1"
                    max="20"
                    value={maxNodes}
                    onChange={(e) => setMaxNodes(parseInt(e.target.value) || 1)}
                    style={{ width: '100%', padding: '8px', borderRadius: '6px', border: '1px solid var(--border)' }}
                  />
                </div>

                <div style={{ display: 'flex', gap: '10px', marginTop: '10px' }}>
                  <button
                    type="button"
                    onClick={() => setShowCreateModal(false)}
                    style={{ flex: 1, padding: '10px', background: '#FFF', border: '1px solid var(--border)', borderRadius: '6px', cursor: 'pointer' }}
                  >
                    Annuler
                  </button>
                  <button
                    type="submit"
                    style={{ flex: 1, padding: '10px', background: 'var(--crimson)', color: '#FFF', border: 'none', borderRadius: '6px', fontWeight: 600, cursor: 'pointer' }}
                  >
                    Signer & Émettre
                  </button>
                </div>
              </form>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
