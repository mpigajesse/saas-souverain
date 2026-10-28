// Hook d'acces au contexte d'authentification.

import { useContext } from 'react'
import { AuthContext, type AuthContextType } from './auth-context'

export function useAuth(): AuthContextType {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error("useAuth doit etre utilise a l'interieur de AuthProvider")
  }
  return context
}
