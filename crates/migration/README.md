# Running Migrator CLI

Run migration and small seeder
```
sea-orm-cli migrate up -d src/migration  
```

Generate entities

```bash
sea-orm-cli generate entity -o src/entities -l --with-serde both
```
