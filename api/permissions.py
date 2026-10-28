"""Permissions partagées par les vues API — voir CONTRAT-API.md.

Les deux rôles applicatifs (« editor » = staff, « tenant » = compte PME) sont
dérivés de `User.is_staff` et de la relation inverse `User.tenant`, exactement
comme le fait déjà le reste du code Django (`hasattr(request.user, 'tenant')`
dans tenants/views_web.py et tenants/views_espace.py).
"""
from rest_framework.permissions import BasePermission


class IsStaffUser(BasePermission):
    """Réservé à l'espace éditeur. 401 si anonyme, 403 si connecté mais non-staff."""

    message = "Réservé aux comptes éditeur."

    def has_permission(self, request, view) -> bool:
        user = request.user
        return bool(user and user.is_authenticated and user.is_staff)


class IsTenantUser(BasePermission):
    """Réservé à l'espace PME. 401 si anonyme, 403 si connecté mais sans tenant."""

    message = "Réservé aux comptes rattachés à une entreprise."

    def has_permission(self, request, view) -> bool:
        user = request.user
        if not (user and user.is_authenticated):
            return False
        return hasattr(user, 'tenant') and user.tenant is not None
