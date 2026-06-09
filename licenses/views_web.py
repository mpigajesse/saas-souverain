from django.contrib import messages
from django.contrib.auth.decorators import login_required
from django.shortcuts import get_object_or_404, redirect, render

from tenants.models import Tenant
from .forms import LicenseForm
from .models import License


@login_required
def license_list(request):
    licenses = License.objects.select_related('tenant').order_by('-created_at')
    return render(request, 'licenses/list.html', {'licenses': licenses})


@login_required
def license_create(request):
    tenant_id = request.GET.get('tenant')
    tenant = Tenant.objects.filter(pk=tenant_id).first() if tenant_id else None
    if request.method == 'POST':
        form = LicenseForm(request.POST, tenant_id=tenant_id)
        if form.is_valid():
            lic = form.save()
            messages.success(request, 'Licence créée avec succès.')
            return redirect('tenant-detail', pk=lic.tenant.pk)
    else:
        form = LicenseForm(tenant_id=tenant_id)
    return render(request, 'licenses/form.html', {
        'form': form,
        'tenant': tenant,
        'action': 'Créer une licence',
    })


@login_required
def license_update(request, pk):
    lic = get_object_or_404(License, pk=pk)
    if request.method == 'POST':
        form = LicenseForm(request.POST, instance=lic)
        if form.is_valid():
            form.save()
            messages.success(request, 'Licence mise à jour.')
            return redirect('tenant-detail', pk=lic.tenant.pk)
    else:
        form = LicenseForm(instance=lic)
    return render(request, 'licenses/form.html', {
        'form': form,
        'license': lic,
        'tenant': lic.tenant,
        'action': 'Modifier la licence',
    })


@login_required
def license_delete(request, pk):
    lic = get_object_or_404(License, pk=pk)
    if request.method == 'POST':
        tenant_pk = lic.tenant.pk
        lic.delete()
        messages.success(request, 'Licence supprimée.')
        return redirect('tenant-detail', pk=tenant_pk)
    return render(request, 'licenses/confirm_delete.html', {'license': lic})
