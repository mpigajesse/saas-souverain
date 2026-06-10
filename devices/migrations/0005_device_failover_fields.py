from django.db import migrations, models


class Migration(migrations.Migration):
    dependencies = [
        ('devices', '0004_device_node_role'),
    ]

    operations = [
        migrations.AddField(
            model_name='device',
            name='last_failover_at',
            field=models.DateTimeField(
                null=True,
                blank=True,
                help_text='Dernier failover automatique détecté sur ce nœud',
            ),
        ),
        migrations.AddField(
            model_name='device',
            name='failover_count',
            field=models.IntegerField(default=0),
        ),
    ]
