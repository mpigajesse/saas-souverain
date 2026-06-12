from django.contrib.auth.decorators import login_required
from django.shortcuts import render


@login_required
def architecture(request):
    return render(request, 'dashboard/architecture.html')


@login_required
def schema_soutenance(request):
    return render(request, 'dashboard/schema_soutenance.html')
