from django.db import migrations, models


class Migration(migrations.Migration):
    dependencies = [
        ('devices', '0003_device_web_addr'),
    ]

    operations = [
        migrations.AddField(
            model_name='device',
            name='node_role',
            field=models.CharField(
                max_length=10,
                blank=True,
                choices=[('primary', 'Primaire'), ('standby', 'Standby')],
                help_text='Rôle PostgreSQL de ce nœud dans le cluster PME',
            ),
        ),
    ]
