from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('devices', '0009_device_chiffrement_disque'),
    ]

    operations = [
        migrations.AddField(
            model_name='device',
            name='recuperation',
            field=models.CharField(blank=True, choices=[('configuree', 'Configurée'), ('reportee', 'Reportée'), ('non_configuree', 'Non configurée'), ('dek_differente', 'DEK différente de celle du relais')], help_text='Récupération de la DEK : configurée, reportée, absente, ou cluster réinstallé à récupérer', max_length=16),
        ),
    ]
