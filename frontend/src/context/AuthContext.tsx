import type React from 'react'
import { createContext, useContext, useState, useEffect, type ReactNode } from 'react'

interface UserProfile {
  name: string
  email: string
  company: string
  role: string
}

interface AuthContextType {
  user: UserProfile | null
  isAuthenticated: boolean
  login: (email: string, pass: string) => boolean
  register: (company: string, email: string, pass: string) => boolean
  logout: () => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

const DEFAULT_USER: UserProfile = {
  name: 'Admin Éditeur',
  email: 'admin@elbaraa-consult.com',
  company: 'EL BARAA CONSULT',
  role: 'Administrateur SaaS',
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserProfile | null>(() => {
    const saved = localStorage.getItem('amane_saas_user')
    return saved ? JSON.parse(saved) : DEFAULT_USER
  })

  useEffect(() => {
    if (user) {
      localStorage.setItem('amane_saas_user', JSON.stringify(user))
    } else {
      localStorage.removeItem('amane_saas_user')
    }
  }, [user])

  const login = (email: string, _pass: string) => {
    const newUser: UserProfile = {
      name: email.split('@')[0],
      email,
      company: 'EL BARAA CONSULT',
      role: 'Administrateur SaaS',
    }
    setUser(newUser)
    return true
  }

  const register = (company: string, email: string, _pass: string) => {
    const newUser: UserProfile = {
      name: email.split('@')[0],
      email,
      company,
      role: 'Fondateur Éditeur',
    }
    setUser(newUser)
    return true
  }

  const logout = () => {
    setUser(null)
  }

  return (
    <AuthContext.Provider value={{ user, isAuthenticated: !!user, login, register, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error('useAuth doit être utilisé à l\'intérieur de AuthProvider')
  }
  return context
}
