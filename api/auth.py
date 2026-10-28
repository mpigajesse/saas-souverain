"""Authentification — `/api/auth/` — voir CONTRAT-API.md section 1.

Session Django + cookie CSRF, pas de JWT. `register` reprend exactement la
logique de `tenants.views_web.inscription` (même formulaire, mêmes validations,
même création User + Tenant + licence d'essai).
"""
from django.contrib.auth import authenticate, login as django_login, logout as django_logout
from django.middleware.csrf import get_token
from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import AllowAny, IsAuthenticated
from rest_framework.response import Response
from rest_framework import status

from tenants.forms import InscriptionForm
from tenants.views_web import _create_tenant_account

from .serializers import serialize_user


@api_view(['GET'])
@permission_classes([AllowAny])
def csrf_view(request):
    """Pose le cookie `csrftoken` — à appeler avant tout POST depuis React."""
    get_token(request)
    return Response({'detail': 'ok'})


@api_view(['POST'])
@permission_classes([AllowAny])
def login_view(request):
    username = request.data.get('username')
    password = request.data.get('password')
    user = authenticate(request, username=username, password=password)
    if user is None:
        return Response({'detail': 'Identifiants invalides.'}, status=status.HTTP_400_BAD_REQUEST)
    django_login(request, user)
    return Response(serialize_user(user))


@api_view(['POST'])
@permission_classes([IsAuthenticated])
def logout_view(request):
    django_logout(request)
    return Response(status=status.HTTP_204_NO_CONTENT)


@api_view(['GET'])
@permission_classes([AllowAny])
def me_view(request):
    if not request.user.is_authenticated:
        return Response({'detail': 'Non authentifié.'}, status=status.HTTP_401_UNAUTHORIZED)
    return Response(serialize_user(request.user))


@api_view(['POST'])
@permission_classes([AllowAny])
def register_view(request):
    form = InscriptionForm(request.data)
    if not form.is_valid():
        errors = {field: [str(e) for e in errs] for field, errs in form.errors.items()}
        return Response(errors, status=status.HTTP_400_BAD_REQUEST)

    user, _tenant = _create_tenant_account(form.cleaned_data)
    django_login(request, user)
    return Response(serialize_user(user), status=status.HTTP_201_CREATED)
