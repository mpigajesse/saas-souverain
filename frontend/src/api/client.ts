import axios from 'axios'
import type { Tenant, License, Device } from '../types/index'

const http = axios.create({
  baseURL: 'http://localhost:8000/api',
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 2500, // Timeout rapide pour basculer gracieusement si Django n'est pas démarré
})

// Données de démonstration et de secours (Fallback Mock) si le serveur Django n'est pas lancé
const MOCK_TENANTS: Tenant[] = [
  {
    id: 'tenant-001',
    name: 'EL BARAA CONSULT',
    subdomain: 'elbaraa-consult',
    created_at: '2026-08-01T10:00:00Z',
    is_active: true,
  },
  {
    id: 'tenant-002',
    name: 'ATLAS PHARMA SOUVERAIN',
    subdomain: 'atlas-pharma',
    created_at: '2026-08-10T14:30:00Z',
    is_active: true,
  },
]

const MOCK_LICENSES: License[] = [
  {
    id: 'lic-001',
    tenant: 'tenant-001',
    tenant_name: 'EL BARAA CONSULT',
    max_nodes: 3,
    plan_tier: 'ENTERPRISE_SOUVERAIN',
    expires_at: '2026-09-30T23:59:59Z',
    is_active: true,
    signed_token: 'ed25519:eyJ0ZW5hbnRfaWQiOiJlbGJhcmFhIiwibWF4X25vZGVzIjozfQ==',
  },
  {
    id: 'lic-002',
    tenant: 'tenant-002',
    tenant_name: 'ATLAS PHARMA SOUVERAIN',
    max_nodes: 5,
    plan_tier: 'STANDARD',
    expires_at: '2026-10-15T23:59:59Z',
    is_active: true,
    signed_token: 'ed25519:eyJ0ZW5hbnRfaWQiOiJhdGxhcyIsIm1heF9ub2RlcyI6NX0=',
  },
]

const MOCK_DEVICES: Device[] = [
  {
    id: 'dev-001',
    tenant: 'tenant-001',
    device_id: 'NODE-ACTIVE-01',
    hostname: 'PC-Direction-Actif',
    node_role: 'ACTIVE',
    ip_address: '10.10.0.1',
    is_online: true,
    last_seen: '2026-08-26T16:05:00Z',
  },
  {
    id: 'dev-002',
    tenant: 'tenant-001',
    device_id: 'NODE-PASSIVE-02',
    hostname: 'PC-Comptabilite-Passif',
    node_role: 'PASSIVE',
    ip_address: '10.10.0.2',
    is_online: true,
    last_seen: '2026-08-26T16:04:30Z',
  },
  {
    id: 'dev-003',
    tenant: 'tenant-001',
    device_id: 'NODE-PASSIVE-03',
    hostname: 'PC-Magasin-Passif',
    node_role: 'PASSIVE',
    ip_address: '10.10.0.3',
    is_online: true,
    last_seen: '2026-08-26T16:04:10Z',
  },
]

export async function getTenants(): Promise<Tenant[]> {
  try {
    const response = await http.get<Tenant[]>('/tenants/')
    return response.data
  } catch (err) {
    console.warn('Backend Django non détecté sur http://localhost:8000. Utilisation des données de secours (Fallback Mock).', err)
    return MOCK_TENANTS
  }
}

export async function getTenant(id: string): Promise<Tenant> {
  try {
    const response = await http.get<Tenant>(`/tenants/${id}/`)
    return response.data
  } catch (err) {
    console.warn(`Backend Django hors-ligne pour tenant ${id}, utilisation des données de secours.`, err)
    return MOCK_TENANTS.find((t) => t.id === id) || MOCK_TENANTS[0]
  }
}

export async function getLicenses(tenantId?: string): Promise<License[]> {
  try {
    const url = tenantId ? `/licenses/?tenant=${tenantId}` : '/licenses/'
    const response = await http.get<License[]>(url)
    return response.data
  } catch (err) {
    console.warn('Backend Django hors-ligne pour licenses, utilisation des données de secours.', err)
    if (tenantId) {
      return MOCK_LICENSES.filter((l) => l.tenant === tenantId)
    }
    return MOCK_LICENSES
  }
}

export async function getDevices(tenantId?: string): Promise<Device[]> {
  try {
    const url = tenantId ? `/devices/?tenant=${tenantId}` : '/devices/'
    const response = await http.get<Device[]>(url)
    return response.data
  } catch (err) {
    console.warn('Backend Django hors-ligne pour devices, utilisation des données de secours.', err)
    if (tenantId) {
      return MOCK_DEVICES.filter((d) => d.tenant === tenantId)
    }
    return MOCK_DEVICES
  }
}
