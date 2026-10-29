from django.db.models import ProtectedError
from django.utils import timezone
from rest_framework import status, viewsets
from rest_framework.decorators import action
from rest_framework.response import Response

from api.permissions import IsStaffUser

from .models import Application
from .registre import RegistreInjoignable, image_presente
from .serializers import ApplicationSerializer


class ApplicationViewSet(viewsets.ModelViewSet):
    """Catalogue des applications hébergées — `/api/applications/` — voir
    CONTRAT-API.md section 2.8.

    Réservé aux comptes éditeur (staff) : l'admission d'un SaaS externe dans
    les clusters des PME est un acte de l'éditeur AMANE.
    """

    permission_classes = [IsStaffUser]
    queryset = Application.objects.all()
    serializer_class = ApplicationSerializer

    def destroy(self, request, *args, **kwargs):
        try:
            return super().destroy(request, *args, **kwargs)
        except ProtectedError:
            return Response(
                {'detail': "Application rattachée à des licences : retirez-la plutôt que de la supprimer."},
                status=status.HTTP_409_CONFLICT,
            )

    @action(detail=True, methods=['post'])
    def valider(self, request, pk=None):
        """Brouillon → validée, si l'image est publiée dans le registre."""
        app = self.get_object()
        if app.statut != Application.BROUILLON:
            return Response({'detail': f"Seul un brouillon se valide (statut : {app.statut})."},
                            status=status.HTTP_409_CONFLICT)
        try:
            presente = image_presente(app.image)
        except RegistreInjoignable:
            return Response({'detail': "Registre injoignable : impossible de vérifier l'image."},
                            status=status.HTTP_503_SERVICE_UNAVAILABLE)
        if not presente:
            return Response({'detail': f"Image absente du registre : {app.image}. Publiez-la d'abord."},
                            status=status.HTTP_409_CONFLICT)
        app.statut = Application.VALIDEE
        app.validee_le = timezone.now()
        app.save(update_fields=['statut', 'validee_le', 'updated_at'])
        return Response(self.get_serializer(app).data)

    @action(detail=True, methods=['post'])
    def retirer(self, request, pk=None):
        """Retrait : les piles générées ensuite ne contiennent plus l'application."""
        app = self.get_object()
        app.statut = Application.RETIREE
        app.save(update_fields=['statut', 'updated_at'])
        return Response(self.get_serializer(app).data)
