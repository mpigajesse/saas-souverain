import axios from 'axios'
import type { Tenant, License, Device } from '../types/index'

const http = axios.create({
  baseURL: 'http://localhost:8000/api',
  headers: {
    'Content-Type': 'application/json',
  },
})

export async function getTenants(): Promise<Tenant[]> {
  const response = await http.get<Tenant[]>('/tenants/')
  return response.data
}

export async function getTenant(id: string): Promise<Tenant> {
  const response = await http.get<Tenant>(`/tenants/${id}/`)
  return response.data
}

export async function getLicenses(tenantId?: string): Promise<License[]> {
  const url = tenantId ? `/licenses/?tenant=${tenantId}` : '/licenses/'
  const response = await http.get<License[]>(url)
  return response.data
}

export async function getDevices(tenantId?: string): Promise<Device[]> {
  const url = tenantId ? `/devices/?tenant=${tenantId}` : '/devices/'
  const response = await http.get<Device[]>(url)
  return response.data
}
