from django import forms
from django.contrib.auth.models import User

from .models import Tenant


class TenantForm(forms.ModelForm):
    class Meta:
        model = Tenant
        fields = ['name', 'email', 'phone', 'address', 'employee_count', 'is_active']
        widgets = {
            'name': forms.TextInput(attrs={'class': 'form-control', 'placeholder': "Nom de l'entreprise"}),
            'email': forms.EmailInput(attrs={'class': 'form-control', 'placeholder': 'contact@entreprise.com'}),
            'phone': forms.TextInput(attrs={'class': 'form-control', 'placeholder': '+33 1 23 45 67 89'}),
            'address': forms.Textarea(attrs={'class': 'form-control', 'rows': 3, 'placeholder': 'Adresse complète'}),
            'employee_count': forms.NumberInput(attrs={'class': 'form-control', 'min': 1}),
            'is_active': forms.CheckboxInput(attrs={'class': 'form-check-input'}),
        }
        labels = {
            'name': "Nom de l'entreprise",
            'email': 'Email de contact',
            'phone': 'Téléphone',
            'address': 'Adresse',
            'employee_count': "Nombre d'employés",
            'is_active': 'Compte actif',
        }
        help_texts = {
            'employee_count': 'Détermine le nombre de postes autorisés dans la licence.',
        }


class InscriptionForm(forms.Form):
    # Compte entreprise
    name = forms.CharField(max_length=200, label="Nom de l'entreprise")
    email = forms.EmailField(label='Email professionnel')
    phone = forms.CharField(max_length=20, required=False, label='Téléphone')
    address = forms.CharField(widget=forms.Textarea(attrs={'rows': 3}), required=False, label='Adresse')
    employee_count = forms.IntegerField(min_value=1, label="Nombre d'employés (= nombre de postes)")

    # Compte utilisateur
    password = forms.CharField(widget=forms.PasswordInput, label='Mot de passe')
    password_confirm = forms.CharField(widget=forms.PasswordInput, label='Confirmer le mot de passe')

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        for field in self.fields.values():
            field.widget.attrs['class'] = 'form-control'

    def clean_email(self):
        email = self.cleaned_data['email']
        if User.objects.filter(email=email).exists():
            raise forms.ValidationError('Un compte avec cet email existe déjà.')
        if Tenant.objects.filter(email=email).exists():
            raise forms.ValidationError('Un compte tenant avec cet email existe déjà.')
        return email

    def clean(self):
        cleaned = super().clean()
        p1 = cleaned.get('password')
        p2 = cleaned.get('password_confirm')
        if p1 and p2 and p1 != p2:
            self.add_error('password_confirm', 'Les mots de passe ne correspondent pas.')
        return cleaned
