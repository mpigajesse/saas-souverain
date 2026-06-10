from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('devices', '0002_device_node_addr'),
    ]

    operations = [
        migrations.AddField(
            model_name='device',
            name='web_addr',
            field=models.CharField(
                blank=True,
                max_length=255,
                help_text='IP:port de l\'interface web PME (ex: 192.168.200.130:3000)',
            ),
        ),
    ]
