from django.contrib.auth.decorators import login_required
from django.shortcuts import redirect, render

from tenants.models import Tenant
from licenses.models import License
from devices.models import Device


def build_dashboard_context() -> dict:
    """Compteurs + tenants récents. Partagé par la vue web `dashboard` et
    `GET /api/dashboard/stats/`.
    """
    all_tenants = Tenant.objects.prefetch_related('licenses', 'devices').all()
    tenants_over_limit = sum(1 for t in all_tenants if not t.licence_status['ok'])

    stats = {
        'total_tenants': all_tenants.count(),
        'active_licenses': License.objects.filter(is_active=True).count(),
        'total_devices': Device.objects.filter(is_active=True).count(),
        'tenants_over_limit': tenants_over_limit,
    }
    recent_tenants = Tenant.objects.order_by('-created_at')[:5]
    return {'stats': stats, 'recent_tenants': recent_tenants}


@login_required
def dashboard(request):
    # Les utilisateurs PME (non-staff) sont redirigés vers leur espace
    if not request.user.is_staff and hasattr(request.user, 'tenant'):
        return redirect('bienvenue')

    return render(request, 'dashboard/index.html', build_dashboard_context())
