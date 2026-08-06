// Fixture: the §9.2 module-contract substrate. Params carry all three
// ground-provenance classes (literal default / no default / computed);
// every policy row gets a triggering plus a non-triggering edit case.

@allowed(['westeurope', 'northeurope'])
param location string = 'westeurope'
param accountName string
param skuName string = 'Standard_LRS'
param deployedAt string = utcNow()

var namePrefix = 'ddd'

resource stg 'Microsoft.Storage/storageAccounts@2023-01-01' = {
  name: '${namePrefix}${accountName}'
  location: location
  sku: {
    name: skuName
  }
  kind: 'StorageV2'
}

output storageId string = stg.id
output storageName string = stg.name
