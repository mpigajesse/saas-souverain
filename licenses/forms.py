from django import forms

from .models import License


class LicenseForm(forms.ModelForm):
    class Meta:
        model = License
        fields = ['tenant', 'plan', 'seats', 'starts_at', 'expires_at', 'is_active']
        widgets = {
            'tenant': forms.Select(attrs={'class': 'form-control'}),
            'plan': forms.Select(attrs={'class': 'form-control'}),
            'seats': forms.NumberInput(attrs={'class': 'form-control', 'min': 1}),
            'starts_at': forms.DateInput(attrs={'class': 'form-control', 'type': 'date'}),
            'expires_at': forms.DateInput(attrs={'class': 'form-control', 'type': 'date'}),
            'is_active': forms.CheckboxInput(attrs={'class': 'form-check-input'}),
        }
        labels = {
            'tenant': 'Tenant',
            'plan': 'Plan',
            'seats': 'Nombre de sièges',
            'starts_at': 'Date de début',
            'expires_at': 'Date de fin (laisser vide = illimité)',
            'is_active': 'Licence active',
        }

    def __init__(self, *args, **kwargs):
        tenant_id = kwargs.pop('tenant_id', None)
        super().__init__(*args, **kwargs)
        if tenant_id:
            self.fields['tenant'].initial = tenant_id
            self.fields['tenant'].widget = forms.HiddenInput()

    def clean(self):
        cleaned = super().clean()
        tenant = cleaned.get('tenant')
        seats = cleaned.get('seats')
        if tenant and seats and seats > tenant.employee_count:
            raise forms.ValidationError(
                f"Les sièges ({seats}) ne peuvent pas dépasser le nombre d'employés"
                f" du tenant ({tenant.employee_count})."
            )
        return cleaned
