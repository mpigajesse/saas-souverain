export interface Tenant {
  id: string
  name: string
  email: string
  phone: string
  address: string
  employee_count: number
  is_active: boolean
  created_at: string
}

export interface License {
  id: string
  tenant: string
  plan: 'starter' | 'pro' | 'enterprise'
  seats: number
  starts_at: string
  expires_at: string | null
  is_active: boolean
  created_at: string
}

export interface Device {
  id: string
  tenant: string
  installation_id: string
  mac_address: string
  hostname: string
  os: string
  registered_at: string
  last_seen: string
  is_active: boolean
  is_within_license: boolean
}
