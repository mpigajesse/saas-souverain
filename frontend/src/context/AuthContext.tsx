// Authentification réelle par session Django + cookie CSRF (voir CONTRAT-API.md).
// Plus aucune authentification simulée : l'état vient exclusivement de
// GET /api/auth/me/ et des réponses de /api/auth/login/, /register/, /logout/.

import { useCallback, useEffect, useState, type ReactNode } from 'react'
import {
  ensureCsrf,
  getCurrentUser,
  login as apiLogin,
  logout as apiLogout,
  normalizeApiError,
  register as apiRegister,
} from '../api/client'
import type { ApiError, LoginPayload, RegisterPayload, User } from '../types/index'
import { AuthContext } from './auth-context'

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Détermine l'état de session au montage : chargement -> connecté/anonyme.
  useEffect(() => {
    let cancelled = false

    async function bootstrap() {
      try {
        await ensureCsrf()
      } catch {
        // Le cookie CSRF n'a pas pu être posé (backend injoignable) : on tente
        // quand même /auth/me/ ci-dessous, qui échouera de la même façon et
        // laissera l'utilisateur dans l'état anonyme plutôt que de bloquer.
      }
      try {
        const current = await getCurrentUser()
        if (!cancelled) setUser(current)
      } catch {
        if (!cancelled) setUser(null)
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    bootstrap()
    return () => {
      cancelled = true
    }
  }, [])

  const login = useCallback(async (payload: LoginPayload): Promise<User> => {
    setError(null)
    try {
      const current = await apiLogin(payload)
      setUser(current)
      return current
    } catch (err) {
      const apiError: ApiError = normalizeApiError(err)
      setError(apiError.message)
      throw apiError
    }
  }, [])

  const register = useCallback(async (payload: RegisterPayload): Promise<User> => {
    setError(null)
    try {
      const current = await apiRegister(payload)
      setUser(current)
      return current
    } catch (err) {
      const apiError: ApiError = normalizeApiError(err)
      setError(apiError.message)
      throw apiError
    }
  }, [])

  const logout = useCallback(async (): Promise<void> => {
    try {
      await apiLogout()
    } finally {
      // On désauthentifie côté client même si l'appel réseau échoue : rester
      // « connecté » dans une UI qui n'arrive plus à parler au serveur serait pire.
      setUser(null)
    }
  }, [])

  const clearError = useCallback(() => setError(null), [])

  return (
    <AuthContext.Provider
      value={{
        user,
        role: user?.role ?? null,
        loading,
        error,
        isAuthenticated: user !== null,
        login,
        register,
        logout,
        clearError,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}

