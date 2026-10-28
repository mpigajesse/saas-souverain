"""Forme unique de l'objet <User> — voir CONTRAT-API.md section 1.

Volontairement une simple fonction plutôt qu'un `serializers.Serializer` DRF :
`role` et `tenant` ne sont pas des champs du modèle `User`, ils sont dérivés,
et le sous-objet tenant a une forme différente de `TenantSerializer`.
"""


def serialize_user(user) -> dict:
    tenant = getattr(user, 'tenant', None)
    tenant_data = None
    if tenant is not None:
        tenant_data = {
            'id': str(tenant.id),
            'name': tenant.name,
            'registration_token': str(tenant.registration_token),
        }
    return {
        'id': user.id,
        'username': user.username,
        'email': user.email,
        'is_staff': user.is_staff,
        'role': 'editor' if user.is_staff else 'tenant',
        'tenant': tenant_data,
    }
